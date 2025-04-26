use std::{collections::HashMap, panic::Location};

use crate::{atom_tree::{AtomRoot, AtomTree}, atom_tree_to_graph::Label, compilation::Compilation, syntax::{CodeSyntax, CollectionSyntax, CompositeTypeSyntax, ExpressionSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypedIdentifierSyntax}, token::{Atom, AtomSub, AtomType}};

pub struct AtomTreeTranslator<'a> {
    pub collections: Vec<CollectionSyntax>,
    pub atom_tree: AtomRoot,
    pub compilation: &'a  mut Compilation,
    pub composites: Vec<CompositeTypeSyntax>
}

impl<'a> AtomTreeTranslator<'a> {
    pub fn new(comp: &'a mut Compilation, collections: Vec<CollectionSyntax>, composites: Vec<CompositeTypeSyntax>) -> Self {
        Self {
            compilation: comp,
            collections,
            composites,
            atom_tree: AtomRoot::default()
        }
    }
    pub fn convert(mut self, problems: Vec<SubstructureSyntax>, solutions: HashMap<String, SubCallSyntax>) -> AtomRoot {

        for problem in problems {
            let mut input = vec![];
            let solution = match solutions.get(&problem.name) {
                Some(s) if s.application.is_some() => {
                    let application = s.application.as_ref().unwrap();
                    match application {
                        ExpressionSyntax::Literal(l) => {
                            vec![l.to_owned().into()]
                        }
                        ExpressionSyntax::Tuple(t) => {
                            t.iter().map(|syntax| {
                                if let ExpressionSyntax::Literal(l) = syntax {
                                    l.to_owned().into()
                                } else {
                                    self.compilation.add_error("Expected tuple of literals", None);
                                    Label::Null
                                }
                            }).collect()
                        }
                        _ => {
                            self.compilation.add_warning("Internal error compiling problem with solution", None);
                            vec![Label::Null; problem.args.len()]
                        }
                    }
                }
                
                _ => vec![Label::Null; problem.args.len()],
            };
            for (arg, sln) in problem.args.iter().zip(solution.iter()) {
                let in_var = self.atom_tree.define_new_var(AtomTree::SeedLabel(*sln));
                input.push(ValueCollection::SingleVar(in_var));
            }
            self.compile_substructure(&problem, input);
        }

        self.atom_tree
    }

    pub fn map_args(&mut self, mut input_args: Vec<ValueCollection>, map_args: &Vec<TypedIdentifierSyntax>, map: &mut HashMap<String, ValueCollection>) {
        assert_eq!(map_args.len(), input_args.len());
    
        for i in (0..map_args.len()).rev() {
            let var = input_args.remove(i).write_as_var(self);
            map.insert(map_args[i].name.to_owned(), var);
        }
    }

    pub fn force(&mut self, value: ValueCollection, t: AtomType) {
        match value {
            ValueCollection::Single(value) => {
                self.atom_tree.define_restriction(value, t);
            }
            ValueCollection::SingleVar(value) => {
                self.atom_tree.define_restriction(AtomTree::Variable { id: value }, t);
            }
            ValueCollection::Tuple(values) => {
                for value in values {
                    self.force(value, t);
                }
            }
            _ => {
                self.compilation.add_error("Can only force tuple types or single types", None);
            }
        }
    }

    pub fn compile_substructure(&mut self, substructure: &SubstructureSyntax, inputs: Vec<ValueCollection>) -> Option<ValueCollection> {
        let mut variables = HashMap::new();
        self.map_args(inputs, &substructure.args, &mut variables);
        for statement in &substructure.code {
            match statement {
                CodeSyntax::Let { variable, value } => {
                    let value = self.compile_value(&value, &mut variables)?;    
                    variables.insert(variable.to_owned(), value.write_as_var(self));
                }
                CodeSyntax::Force { value, type_syntax } => {
                    let value = self.compile_value(&value, &mut variables)?;
                    let force_type = *type_syntax.as_atom().expect("Todo: Force other types");
                    self.force(value, force_type);
                }
                CodeSyntax::Sub(sub) => {
                    self.compile_sub_call(sub, &variables);
                }
                _ => {todo!()}
            }
        }
        if let Some(res) = &substructure.result {
            self.compile_value(res, &variables)
        } else {
            Some(ValueCollection::Tuple(vec![]))
        }
    }


    pub fn compile_sub_call(&mut self, sub_call_syntax: &SubCallSyntax, variables: &HashMap<String, ValueCollection>) -> Option<ValueCollection> {
        let mut application = match  &sub_call_syntax.application {
            Some(application) => self.compile_value(application, &variables)?,
            None => ValueCollection::Tuple(vec![])
        };
        match &sub_call_syntax.location {
            SubLocation::Atom(a) => {
                match a {
                    AtomSub::Not => {
                        let application = application.get_as_atom_tree_if_single_or_error(self.compilation)?;
                        Some(ValueCollection::Single(AtomTree::Not(application.into())))
                    },
                    AtomSub::Or => {
                        let (application_a, application_b) = match application {
                            ValueCollection::Tuple(mut t) if t.len() == 2 => {
                                (
                                    //Or is commutative so order of inputs doesn't really matter
                                    t.pop()?.get_as_atom_tree_if_single_or_error(self.compilation)?,
                                    t.pop()?.get_as_atom_tree_if_single_or_error(self.compilation)?
                                )
                            },
                            
                            _ => {
                                self.compilation.add_error("Incorrect parameters for or function. Expected 2 boolean parameter.", None);
                                return None;
                            }
                        };
                        Some(ValueCollection::Single(AtomTree::Or(application_a.into(), application_b.into())))
                    }
                }
            }
            SubLocation::Structure { collection, sub } => {
                let collection = self.collections.iter().find(|&f| &f.name == collection).unwrap();
                let sub = collection.subs.iter().find(|&f| &f.name == sub).unwrap().clone();
                let application = if let ValueCollection::Tuple(t) = application {
                    t
                } else {
                    vec![application]
                };
                self.compile_substructure(&sub, application)

            }
        
        }
    }

    pub fn compile_value(&mut self, value: &ExpressionSyntax, variables: &HashMap<String, ValueCollection>) -> Option<ValueCollection> {

        match value {
            ExpressionSyntax::Access{base, field} => {
                self.compile_value(base, variables)?.access_or_error(field, self.compilation).cloned()
            }
            ExpressionSyntax::CompositeConstructor { type_name, field_assign } => {
                todo!()
            }
            ExpressionSyntax::Literal(atom) => {
                Some(ValueCollection::Single(AtomTree::AtomType { atom: *atom }))
            }
            ExpressionSyntax::Variable(name) => {
                if let Some(var) = variables.get(name) {
                    Some(var.to_owned())
                } else {
                    self.compilation.add_error(&format!("Variable {name} not found in current scope."), None);
                    None
                }
            }
            ExpressionSyntax::Tuple(t) => {
                let mut vals = vec![];
                for v in t {
                    let mut v = self.compile_value(v, variables)?;
                    vals.push(v);
                }
                Some(ValueCollection::Tuple(vals))
            }
            ExpressionSyntax::Sub(sub_call_syntax) => {
                self.compile_sub_call(sub_call_syntax, variables)
            }
        }

    }
}

#[derive(Clone)]
pub enum ValueCollection {
    SingleVar(usize),
    Single(AtomTree),
    Tuple(Vec<Self>),
    Composite {
        composite_name: String,
        fields: HashMap<String, ValueCollection>
    }
}

impl ValueCollection {
    pub fn get_as_atom_tree_if_single_or_error(self, compilation: &mut Compilation) -> Option<AtomTree> {
        match self {
            Self::SingleVar(id) => Some(AtomTree::Variable { id }),
            Self::Single(tree) => Some(tree),
            Self::Tuple(mut t) if t.len() == 1 => t.remove(0).get_as_atom_tree_if_single_or_error(compilation),
            _ => {
                compilation.add_error("Expected simple boolean value", None);
                None
            }
        }
    }
    pub fn write_as_var(self, atom_tree_translate: &mut AtomTreeTranslator) -> Self {
        match self {
            Self::Single(atom_tree) => {
                Self::SingleVar(atom_tree_translate.atom_tree.define_new_var(atom_tree))
            },
            Self::Tuple(v) => {
                Self::Tuple(
                    v.into_iter()
                    .map(|value| value.write_as_var(atom_tree_translate))
                    .collect())
            }
            Self::Composite { composite_name, fields } => {
                Self::Composite { composite_name,
                     fields: fields.into_iter()
                            .map(|(k, v)| (k, v.write_as_var(atom_tree_translate)))
                            .collect() 
                    }
            }
            Self::SingleVar(s) => Self::SingleVar(s)
        }
    }
    pub fn access_or_error(&self, accessor_name: &String, compilation: &mut Compilation) -> Option<&Self> {
        match self {
            Self::Single(_) | Self::SingleVar(_) => {
                compilation.add_error(&format!("Cannot access field {} on a simple bool-type value", accessor_name), None);
                None
            },
            Self::Tuple(fields) => {
                todo!("I hate tuples anyways")
                //let index = accessor_name.parse()
            }
            Self::Composite { fields, .. } => {
                fields.get(accessor_name)
            }
        }
    }
}