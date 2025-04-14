use std::{collections::HashMap, panic::Location};

use crate::{atom_tree::{AtomRoot, AtomTree}, compilation::Compilation, syntax::{ArgumentSyntax, CodeSyntax, CollectionSyntax, NodeValueSyntax, SubCallSyntax, SubLocation, SubstructureSyntax}, token::{Atom, AtomSub}};

pub struct AtomTreeTranslator {
    pub collections: Vec<CollectionSyntax>,
    pub atom_tree: AtomRoot,
    pub compilation: Compilation
}

impl AtomTreeTranslator {
    pub fn new(comp: Compilation, collections: Vec<CollectionSyntax>) -> Self {
        Self {
            compilation: comp,
            collections,
            atom_tree: AtomRoot::default()
        }
    }
    pub fn convert(mut self, problems: Vec<SubstructureSyntax>) -> AtomRoot {

        for problem in problems {
            let mut input = vec![];
            for _ in &problem.args {
                let in_var = self.atom_tree.define_new_var(AtomTree::Null);
                input.push(AtomTree::Variable { id: in_var });
            }
            self.compile_substructure(&problem, input);
        }

        self.atom_tree
    }

    pub fn map_args(&mut self, mut input_args: Vec<AtomTree>, map_args: &Vec<ArgumentSyntax>, map: &mut HashMap<String, usize>) {
        assert_eq!(map_args.len(), input_args.len());
    
        for i in (0..map_args.len()).rev() {
            let var = self.atom_tree.define_new_var(input_args.remove(i));
            map.insert(map_args[i].name.to_owned(), var);
        }
    }

    pub fn compile_substructure(&mut self, substructure: &SubstructureSyntax, inputs: Vec<AtomTree>) -> Option<Vec<AtomTree>> {
        let mut variables = HashMap::new();
        self.map_args(inputs, &substructure.args, &mut variables);
        for statement in &substructure.code {
            match statement {
                CodeSyntax::Let { variable, value } => {
                    if let Some(mut value) =  self.compile_value(&value, &mut variables) {
                        if value.len() != 1 {
                            todo!("Setting variables to tuples")
                        }
                        let reference = self.atom_tree.define_new_var(value.pop().unwrap());
                        variables.insert(variable.to_owned(), reference);

                    }
                }
                CodeSyntax::Force { value, type_syntax } => {
                    if let Some(mut value) =  self.compile_value(&value, &mut variables) {
                        if value.len() != 1 {
                            todo!("Forcing tuples")
                        }
                        
                        self.atom_tree.define_restriction(value.pop().unwrap(), *type_syntax.as_atom().expect("Todo: Force other types"));

                    }
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
            Some(vec![])
        }
    }


    pub fn compile_sub_call(&mut self, sub_call_syntax: &SubCallSyntax, variables: &HashMap<String, usize>) -> Option<Vec<AtomTree>> {
        let mut application = match  &sub_call_syntax.application {
            Some(application) => self.compile_value(application, &variables)?,
            None => vec![]
        };
        match &sub_call_syntax.location {
            SubLocation::Atom(a) => {
                match a {
                    AtomSub::Not => {
                        if application.len() != 1 {
                            self.compilation.add_error("Not-sub must take 1 input", None);
                            return None;
                        }
                        Some(vec![AtomTree::Not(application.pop().unwrap().into())])
                    },
                    AtomSub::Or => {
                        if application.len() != 2 {
                            self.compilation.add_error("Or-sub must take 2 inputs", None);
                            return None;
                        }
                        Some(vec![AtomTree::Or(application.pop().unwrap().into(), application.pop().unwrap().into())])
                    }
                }
            }
            SubLocation::Structure { collection, sub } => {
                let collection = self.collections.iter().find(|&f| &f.name == collection).unwrap();
                let sub = collection.subs.iter().find(|&f| &f.name == sub).unwrap().clone();

                self.compile_substructure(&sub, application)

            }
        
        }
    }

    pub fn compile_value(&mut self, value: &NodeValueSyntax, variables: &HashMap<String, usize>) -> Option<Vec<AtomTree>> {

        match value {
            NodeValueSyntax::Literal(atom) => {
                Some(vec![AtomTree::AtomType { atom: *atom }])
            }
            NodeValueSyntax::Variable(name) => {
                if let Some(id) = variables.get(name) {
                    Some(vec![AtomTree::Variable{id: *id}])
                } else {
                    self.compilation.add_error(&format!("Variable {name} not found in current scope."), None);
                    None
                }
            }
            NodeValueSyntax::Tuple(t) => {
                let mut vals = vec![];
                for v in t {
                    let mut v = self.compile_value(v, variables)?;
                    if v.len() != 1 {
                        todo!("Implement tuple in tuple support/advanced structures")
                    }
                    vals.append(&mut v);
                }
                Some(vals)
            }
            NodeValueSyntax::Sub(sub_call_syntax) => {
                self.compile_sub_call(sub_call_syntax, variables)
            }
        }

    }
}