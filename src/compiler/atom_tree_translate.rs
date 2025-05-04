use std::{collections::HashMap, panic::Location};

use crate::compiler::{atom_tree::{AtomRoot, AtomTree}, atom_tree_to_graph::Label, compilation::Compilation, syntax::{CodeSyntax, CollectionSyntax, CompositeTypeSyntax, ExpressionSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypedIdentifierSyntax}, token::{Atom, AtomSub, AtomType}};

use super::atom_tree::ValueAction;

pub struct AtomTreeTranslator<'a> {
    pub collections: Vec<CollectionSyntax>,
    pub atom_tree: AtomRoot,
    pub compilation: &'a  mut Compilation,
    pub composites: Vec<CompositeTypeSyntax>,
    pub condition_stack: Vec<AtomTree>,
}

impl<'a> AtomTreeTranslator<'a> {
    pub fn find_composite(&self, name: &String) -> Option<&CompositeTypeSyntax> {
        self.composites.iter().find(|composite| &composite.name == name)
    }
    pub fn new(comp: &'a mut Compilation, collections: Vec<CollectionSyntax>, composites: Vec<CompositeTypeSyntax>) -> Self {
        Self {
            compilation: comp,
            collections,
            composites,
            atom_tree: AtomRoot::default(),
            condition_stack: vec![]
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

    ///Mutate value such that it is always valid if forced if any condition is not met
    pub fn mutate_value_to_enforce_at_condition(&self, mut value: AtomTree, t: AtomType) -> AtomTree {
        for condition in &self.condition_stack {
            match t {
                AtomType::True => {
                    value = AtomTree::Or(value.into(), AtomTree::Not(condition.to_owned().into()).into())
                }
                AtomType::False => {
                    value = AtomTree::Not(AtomTree::Or(AtomTree::Not(value.into()).into(), AtomTree::Not(condition.to_owned().into()).into()).into())
                }
            }
        }
        value
    }

    pub fn force(&mut self, value: ValueCollection, t: AtomType) {
        match value {
            ValueCollection::Single(value) => {
                let value = self.mutate_value_to_enforce_at_condition(value, t);
                self.atom_tree.define_restriction(value, t);

            }
            ValueCollection::SingleVar(value) => {

                let value = self.mutate_value_to_enforce_at_condition(AtomTree::Variable { id: value }, t);

                self.atom_tree.define_restriction(value, t);
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
        self.compile_code_block(&substructure.code, &mut variables);
        if let Some(res) = &substructure.result {
            self.compile_expression(res, &variables)
        } else {
            Some(ValueCollection::Tuple(vec![]))
        }
    }

    pub fn compile_code_block(&mut self, block: &Vec<CodeSyntax>, variables: &mut HashMap<String, ValueCollection>) -> Option<()> {
        for statement in block {
            match statement {
                //A conditional code block only changes force statements to always be valid iff the condition is not met
                CodeSyntax::If { condition, condition_true } => {
                    let condition = self.compile_expression(condition, variables)?.write_as_var(self).get_as_atom_tree_if_single_or_error(self.compilation)?;
                    self.condition_stack.push(condition);
                    self.compile_code_block(condition_true, variables);
                    self.condition_stack.pop();
                }
                CodeSyntax::IfElse { condition, condition_true, condition_false } => {
                    let condition = self.compile_expression(condition, variables)?.write_as_var(self).get_as_atom_tree_if_single_or_error(self.compilation)?;
                    self.condition_stack.push(condition);
                    self.compile_code_block(condition_true, variables);
                    let condition = self.condition_stack.pop().expect("Expected condition stack to be non-empty");

                    let inverted_condition = AtomTree::Not(condition.into());
                    self.condition_stack.push(inverted_condition);
                    self.compile_code_block(condition_false, variables);
                    self.condition_stack.pop();
                }
                CodeSyntax::Let { variable, value } => {
                    let value = self.compile_expression(&value, variables)?;    
                    variables.insert(variable.to_owned(), value.write_as_var(self));
                }
                CodeSyntax::Force { value, type_syntax } => {
                    let value = self.compile_expression(&value, variables)?;
                    let force_type = *type_syntax.as_atom().expect("Todo: Force other types");
                    self.force(value, force_type);
                }
                CodeSyntax::Sub(sub) => {
                    self.compile_sub_call(sub, &variables);
                }
                CodeSyntax::Output { expression } => {
                    if !self.compilation.settings().output_code_logs {
                        continue;
                    }
                    let value = self.compile_expression(expression, variables)?;
                    let mut string_buffer = vec![];
                    let mut value_buffer = vec![];
                    let mut cur_str = String::new();
                    self.format(&mut string_buffer, &mut value_buffer, value, &mut cur_str);
                    string_buffer.push(cur_str);
                    let condition = self.true_if_all_conditions_are_met();
                    self.atom_tree.value_actions.push((condition, ValueAction::Output(string_buffer, value_buffer)));
                }
                _ => {todo!()}
            }
        }
        Some(())
    }

    fn true_if_all_conditions_are_met(&self) -> AtomTree {
        let mut cur = AtomTree::AtomType { atom: AtomType::True };
        for condition in &self.condition_stack {
            cur = AtomTree::Not(AtomTree::Or(AtomTree::Not(condition.to_owned().into()).into(), AtomTree::Not(cur.into()).into()).into());
        }
        cur
    }

    fn format(&mut self, string_buffer: &mut Vec<String>, value_buffer: &mut Vec<AtomTree>, value: ValueCollection, current_string: &mut String) {
        match value {
            ValueCollection::Super(SuperValue::Int(i)) => {
                current_string.push_str(&i.to_string());
            }
            ValueCollection::Super(SuperValue::String(s)) => {
                current_string.push_str(&s);
            }
            ValueCollection::Array { items } => {
                current_string.push_str("[");
                for val in items {
                    match val {
                        Some(s) => self.format(string_buffer, value_buffer, s, current_string),
                        None => current_string.push_str(" ")
                    }
                    current_string.push_str(", ");

                }
                current_string.push_str("]");
            }
            ValueCollection::Composite { composite_name, fields } => {
                current_string.push_str(&composite_name);
                current_string.push_str(" { ");
                for (field, value) in fields {
                    current_string.push_str(&field);
                    current_string.push_str(": ");
                    self.format(string_buffer, value_buffer, value, current_string);
                    current_string.push_str(", ");
                }
                current_string.push_str(" } ");

            }
            ValueCollection::Single(tree) => {
                let mut push_string = String::new();
                std::mem::swap(current_string, &mut push_string); 
                string_buffer.push(push_string);
                value_buffer.push(tree);
            }
            ValueCollection::SingleVar(v) => {
                let mut push_string = String::new();
                std::mem::swap(current_string, &mut push_string); 
                string_buffer.push(push_string);
                value_buffer.push(AtomTree::Variable { id: v });
            }
            ValueCollection::Tuple(t) => {
                current_string.push_str("(");
                for val in t {
                    self.format(string_buffer, value_buffer, val, current_string);
                    current_string.push_str(", ");

                }
                current_string.push_str(")");
            }
            ValueCollection::Super(SuperValue::String(s)) => {
                current_string.push_str(&s);
            }
            _ => {
                self.compilation.add_error("Type cannot be formatted for outputting", None);
            }
        }
    }

    pub fn compile_sub_call(&mut self, sub_call_syntax: &SubCallSyntax, variables: &HashMap<String, ValueCollection>) -> Option<ValueCollection> {
        let mut application = match  &sub_call_syntax.application {
            Some(application) => self.compile_expression(application, &variables)?,
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

    pub fn compile_expression(&mut self, value: &ExpressionSyntax, variables: &HashMap<String, ValueCollection>) -> Option<ValueCollection> {

        match value {
            ExpressionSyntax::Array(expressions) => {
                let items = expressions.iter().map(|exp| self.compile_expression(exp, variables)).collect();
                Some(ValueCollection::Array { items })
            }
            ExpressionSyntax::LengthArray { count, base } => {
                let count = self.compile_expression(&count, variables)?;
                let expression = self.compile_expression(&base, variables);
                match count {
                    ValueCollection::Super(SuperValue::Int(count)) => {
                        let items = vec![expression; count];
                        Some(ValueCollection::Array { items })
                    },
                    _ => {self.compilation.add_error("Expected integer", None); None} 
                }
            }
            ExpressionSyntax::Int(i) => {
                Some(ValueCollection::Super(SuperValue::Int(*i)))
            }
            ExpressionSyntax::String(string) => {
                Some(ValueCollection::Super(SuperValue::String(string.to_owned())))
            }
            ExpressionSyntax::Access{base, field} => {
                self.compile_expression(base, variables)?.access_identifier_or_error(field, self.compilation).cloned()
            }
            ExpressionSyntax::AccessIdx{base, idx} => {
                self.compile_expression(base, variables)?.access_index_or_error(idx, self.compilation).cloned()
            }
            ExpressionSyntax::IndexOp { base, index } => {
                let index = self.compile_expression(index, variables)?.get_as_int_or_error(self.compilation)?;;
                if let ValueCollection::Array { items } = self.compile_expression(&base, variables)? {
                    if let Some(e) = items.get(index) {
                        e.to_owned()
                    } else {
                        self.compilation.add_error("Index was out of bounds", None);
                        None
                    }
                } else {
                    self.compilation.add_error("You can only use the index operation on array types", None);
                    None
                }
            }
            ExpressionSyntax::CompositeConstructor { type_name, field_assign } => {
                let composite = match self.find_composite(type_name) {
                    Some(s) => s,
                    None => {
                        self.compilation.add_error(&format!("Couldn't find type {type_name}"), None);
                        return None;
                    }
                };
                let mut missing_fields = composite.fields.clone();
                let mut assigned_fields = HashMap::new();
                for field_assign in field_assign {
                    match missing_fields.iter().enumerate().find(|(_, n)| n.name == field_assign.left) {
                        Some((i, _)) => {
                            missing_fields.remove(i);
                            assigned_fields.insert(field_assign.left.clone(), self.compile_expression(&field_assign.right, variables)?);
                        },
                        None => {
                            self.compilation.add_error(&format!("Field \"{}\" has either already been assigned, or is not in the composite type.", field_assign.left), None);
                        }
                    }
                }
                if !missing_fields.is_empty() {
                    self.compilation.add_error("Some fields of the struct haven't been assigned", None);
                    return None;
                }
                return Some(ValueCollection::Composite { composite_name: type_name.clone(), fields: assigned_fields })
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
                    let mut v = self.compile_expression(v, variables)?;
                    vals.push(v);
                }
                Some(ValueCollection::Tuple(vals))
            }
            ExpressionSyntax::Sub(sub_call_syntax) => {
                self.compile_sub_call(sub_call_syntax, variables)
            }
            _ => todo!("Compiling {:?} hasn't been implemented", value)
        }

    }
}

#[derive(Clone)]
pub enum ValueCollection {
    Array {
        items: Vec<Option<ValueCollection>>
    },
    SingleVar(usize),
    Single(AtomTree),
    Tuple(Vec<Self>),
    Composite {
        composite_name: String,
        fields: HashMap<String, ValueCollection>
    },
    Super(SuperValue)
}
#[derive(Clone)]

pub enum SuperValue {
    String(String),
    Int(usize)
}

impl ValueCollection {
    pub fn get_as_int_or_error(self, compilation: &mut Compilation) -> Option<usize> {
        match self {
            Self::Super(SuperValue::Int(i)) => Some(i),
            _ => {
                compilation.add_error("Expected super integer value", None);
                None
            }
        }
    }
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
            Self::SingleVar(s) => Self::SingleVar(s),
            _ => self
        }
    }
    pub fn access_identifier_or_error(&self, accessor_name: &String, compilation: &mut Compilation) -> Option<&Self> {
        match self {
            Self::Composite { fields, .. } => {
                match fields.get(accessor_name) {
                    Some(s) => Some(s),
                    None => {
                        compilation.add_error(&format!("Cannot access field {} on this type", accessor_name), None);
                        None
        
                    }
                }
            }
            _ => {
                compilation.add_error(&format!("Tried to access fields on a value of this type."), None);
                return None;
            }
        }
    }
    pub fn access_index_or_error(&self, accessor_idx: &usize, compilation: &mut Compilation) -> Option<&Self> {
        match self {
            Self::Tuple(fields) => {
                match fields.get(*accessor_idx) {
                    Some(s) => Some(s),
                    None => {
                        compilation.add_error(&format!("Index {} is out of bounds for tuple with size {}", accessor_idx, fields.len()), None);
                        None
                    }
                }
            }
            Self::Composite { fields, .. } => {
                todo!("Indexing composite types");
            }
            _ => {
                compilation.add_error(&format!("Can't index access a value of this type."), None);
                return None;
            }
        }
    }
}

