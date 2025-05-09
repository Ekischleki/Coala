use std::{collections::HashMap, fmt::Pointer};

use crate::compiler::{atom_tree::{AtomRoot, AtomTree}, atom_tree_to_graph::Label, compilation::Compilation, syntax::{CodeSyntax, CollectionSyntax, CompositeTypeSyntax, ExpressionSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypedIdentifierSyntax}, token::{AtomSub, AtomType}};

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
            self.compile_expression(res, &mut variables)
        } else {
            Some(ValueCollection::Tuple(vec![]))
        }
    }
    pub fn select_if_conditions_met(select_if_true: AtomTree, select_if_false: AtomTree, condition: AtomTree) -> AtomTree {
        //Selector, selecting new value if condition is met and old value otherwise
        let selected_at_true = AtomTree::Not(
            AtomTree::Or(
                AtomTree::Not(condition.clone().into()).into(), 
                AtomTree::Not(select_if_true.into()).into()
            ).into()
        );
        let selected_at_false = AtomTree::Not(
            AtomTree::Or(
                condition.into(), 
                AtomTree::Not(select_if_false.into()).into()
            ).into()
        );
        let selected = AtomTree::Or(selected_at_true.into(), selected_at_false.into());
        selected
    }

    pub fn compile_code_block(&mut self, block: &Vec<CodeSyntax>, variables: &mut HashMap<String, ValueCollection>) -> Option<()> {
        let compilation = unsafe {self.extract_compilation()};
        for statement in block {
            match statement {
                CodeSyntax::For { iterator_variable, iterator_amount, iterator_body } => {
                    let iterator_amount = self.compile_expression(iterator_amount, variables)?.get_as_int_or_error(self.compilation)?;
                    for i in 0..iterator_amount {
                        variables.insert(iterator_variable.to_owned(), ValueCollection::Super(SuperValue::Int(i)));
                        self.compile_code_block(iterator_body, variables);
                    }
                }
                CodeSyntax::ReassignSyntax { variable, value } => {
                    let condition = self.true_if_all_conditions_are_met();

                    let new_value = self.compile_expression(value, variables)?;

                    let var = self.compile_access_expression(variable, variables)?;
                    
                    //Hack to allow writing non-atom-type values to variables
                    if let Some(AtomType::True) = condition.as_atom_type() {
                        *var = new_value;
                    } else {
                        
                        let old_value = var.clone().get_as_atom_tree_if_single_or_error(compilation)?;
    
                        let new_value = new_value.get_as_atom_tree_if_single_or_error(self.compilation)?;
    
                        let selected = Self::select_if_conditions_met(new_value, old_value, condition);
                        //println!("Selected: {:#?}", selected);
                        *var = ValueCollection::Single(selected);
                    }
                    

                    //println!("Vars: {:#?}", variables);
                }
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
                    let value = self.compile_expression(&value, variables).unwrap_or_default();    
                    variables.insert(variable.to_owned(), value.write_as_var(self));
                }
                CodeSyntax::Force { value, type_syntax } => {
                    let value = self.compile_expression(&value, variables)?;
                    let force_type = *type_syntax.as_atom().expect("Todo: Force other types");
                    self.force(value, force_type);
                }
                CodeSyntax::Sub(sub) => {
                    self.compile_sub_call(sub, variables);
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
            ValueCollection::Error => {
                current_string.push_str("<Error>");
            }
            ValueCollection::Super(SuperValue::Int(i)) => {
                current_string.push_str(&i.to_string());
            }
            ValueCollection::Super(SuperValue::String(s)) => {
                current_string.push_str(&s);
            }
            ValueCollection::Array { items } => {
                current_string.push_str("[");
                for val in items {

                    self.format(string_buffer, value_buffer, val, current_string);
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

    pub fn compile_sub_call(&mut self, sub_call_syntax: &SubCallSyntax, variables: &mut HashMap<String, ValueCollection>) -> Option<ValueCollection> {
        let application = match  &sub_call_syntax.application {
            Some(application) => self.compile_expression(application, variables)?,
            None => ValueCollection::Tuple(vec![])
        };
        match &sub_call_syntax.location {
            SubLocation::Super(name) => {
                //println!("Application: {:#?}", application);

                let mut application = match application {
                    ValueCollection::Tuple(t) => {
                        t
                    }
                    _ => vec![application]
                };
                //println!("Application: {:#?}", application);
                match name.as_str() {
                    "add" => {
                        if application.len() == 2 {
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Super(SuperValue::Int(a.wrapping_add(b))))
                        } else {
                            self.compilation.add_error("Incorrect parameters for add function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "sb" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Super(SuperValue::Int(a.wrapping_sub(b))))
                        } else {
                            self.compilation.add_error("Incorrect parameters for sb function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "mul" => {
                        if application.len() == 2 {
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Super(SuperValue::Int(a.wrapping_mul(b))))
                        } else {
                            self.compilation.add_error("Incorrect parameters for mul function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "div" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            if b == 0 {
                                self.compilation.add_error("Division by zero", None);
                                return None;
                            }
                            Some(ValueCollection::Super(SuperValue::Int(a / b)))
                        } else {
                            self.compilation.add_error("Incorrect parameters for div function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "mod" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            if b == 0 {
                                self.compilation.add_error("Division by zero", None);
                                return None;
                            }
                            Some(ValueCollection::Super(SuperValue::Int(a % b)))
                        } else {
                            self.compilation.add_error("Incorrect parameters for mod function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "eq" => {
                        if application.len() == 2 {
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Single(AtomTree::AtomType {atom: if a == b { AtomType::True} else {AtomType::False}} ))
                        } else {
                            self.compilation.add_error("Incorrect parameters for eq function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "ne" => {
                        if application.len() == 2 {
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Single(AtomTree::AtomType {atom: if a != b { AtomType::True} else {AtomType::False}} ))
                        } else {
                            self.compilation.add_error("Incorrect parameters for ne function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "grt" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Single(AtomTree::AtomType {atom: if a > b { AtomType::True} else {AtomType::False}} ))
                        } else {
                            self.compilation.add_error("Incorrect parameters for grt function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "grte" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Single(AtomTree::AtomType {atom: if a >= b { AtomType::True} else {AtomType::False}} ))
                        } else {
                            self.compilation.add_error("Incorrect parameters for grte function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "len" => {
                        if application.len() == 1 {
                            let a = application.pop()?.get_as_array_or_error(self.compilation)?;
                            
                            Some(ValueCollection::Super(SuperValue::Int(a.len())))
                        } else {
                            self.compilation.add_error("Incorrect parameters for len function. Expected 1 integer parameter.", None);
                            None
                        }
                    }
                    "min" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Super(SuperValue::Int(a.min(b))))
                        } else {
                            self.compilation.add_error("Incorrect parameters for min function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    "max" => {
                        if application.len() == 2 {
                            let b = application.pop()?.get_as_int_or_error(self.compilation)?;
                            let a = application.pop()?.get_as_int_or_error(self.compilation)?;
                            Some(ValueCollection::Super(SuperValue::Int(a.max(b))))
                        } else {
                            self.compilation.add_error("Incorrect parameters for max function. Expected 2 integer parameters.", None);
                            None
                        }
                    }
                    _ => {
                        self.compilation.add_error(&format!("Unknown super function: {name}"), None);
                        None
                    }
                }
            }
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
    
    pub fn compile_expression(&mut self, value: &ExpressionSyntax, variables: &mut HashMap<String, ValueCollection>) -> Option<ValueCollection> {
        let compilation = unsafe {self.extract_compilation()};
        match value {
            ExpressionSyntax::Array(expressions) => {
                let items = expressions.iter().map(|exp| self.compile_expression(exp, variables).unwrap_or_default()).collect();
                Some(ValueCollection::Array { items })
            }
            ExpressionSyntax::LengthArray { count, base } => {
                let count = self.compile_expression(&count, variables)?;
                let expression = self.compile_expression(&base, variables).unwrap_or_default();
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
                self.compile_access_expression(&base, variables)?.access_identifier_or_error(field, compilation).cloned() 

            }
            ExpressionSyntax::AccessIdx{base, idx} => {
                self.compile_access_expression(&base, variables)?.access_index_or_error(idx, compilation).cloned()
            }
            ExpressionSyntax::IndexOp { base, index: idx } => {
                let idx = self.compile_expression(idx, variables)?.get_as_int_or_error(self.compilation)?;
                self.compile_access_expression(&base, variables)?.access_indexed_or_error(&idx, compilation).cloned()
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
                    let v = self.compile_expression(v, variables)?;
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
    //Made for recursive mutable functions that return a mutable reference related to self, that doesn't require a mutable reference to self
    pub unsafe fn extract_compilation(&mut self) -> &'a mut Compilation {
        let compilation = &mut *(self.compilation as *mut Compilation);
        compilation
    }
    pub unsafe fn extract<T>(s: &mut T) -> &mut T {
        let s = &mut *(s as *mut T);
        s
    }

    pub fn compile_access_expression<'b>(&mut self, value: &ExpressionSyntax, variables: &'b mut HashMap<String, ValueCollection>) -> Option<&'b mut ValueCollection> {
        let compilation = unsafe {self.extract_compilation()};
        match value {
            ExpressionSyntax::Access { base, field } => {
                let base = self.compile_access_expression(base, variables)?;
                base.access_identifier_or_error(field, compilation)
            }
            ExpressionSyntax::AccessIdx { base, idx } => {
                let base = self.compile_access_expression(base, variables)?;
                base.access_index_or_error(idx, compilation)

            }
            ExpressionSyntax::IndexOp { base, index } => {
                let index = self.compile_expression(index, variables)?.get_as_int_or_error(self.compilation)?;
                let base = self.compile_access_expression(base, variables)?;
                base.access_indexed_or_error(&index, compilation)
            }
            ExpressionSyntax::Variable(name) => {
                if let Some(var) = variables.get_mut(name) {
                    Some(var)
                } else {
                    compilation.add_error(&format!("Variable {name} not found in current scope."), None);
                    None
                }
            }
            _ => {
                compilation.add_error("Internal: Expected access expression (Should've been caught during parsing)", None);
                None
            }
            
        }
    }
}

#[derive(Clone, Default, Debug)]
pub enum ValueCollection {
    #[default]
    Error,
    Array {
        items: Vec<ValueCollection>
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
#[derive(Clone, Debug)]

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
    pub fn get_as_array_or_error(self, compilation: &mut Compilation) -> Option<Vec<ValueCollection>> {
        match self {
            Self::Array { items } => Some(items),
            _ => {
                compilation.add_error("Expected array value", None);
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
    pub fn access_identifier_or_error(&mut self, accessor_name: &String, compilation: &mut Compilation) -> Option<&mut Self> {
        match self {
            Self::Composite { fields, .. } => 
                {
                    if let Some(field) = fields.get_mut(accessor_name) {
                        Some(field)
                    } else {
                        compilation.add_error(&format!("Field \"{}\" not found in composite type", accessor_name), None);
                        None
                    }
                },
            
            _ => 
                {
                    compilation.add_error(&format!("Tried to access field \"{accessor_name}\" on a value that doesn't have fields."), None);
                    None
                }

            
        }
    }
    pub fn access_indexed_or_error(&mut self, idx: &usize, compilation: &mut Compilation) -> Option<&mut Self> {
        match self {
            Self::Array { items } => {
                let item_len = items.len();
                match items.get_mut(*idx) {
                    Some(field) => Some(field),
                    None => {
                        compilation.add_error(&format!("Index {} is out of bounds for array with size {}", idx, item_len), None);
                        None
                    }
                }
                    
            }
            _ => {
                compilation.add_error(&format!("Can't index access a value of this type."), None);
                None
            }
        }
    }
    pub fn access_index_or_error(&mut self, accessor_idx: &usize, compilation: &mut Compilation) -> Option<&mut Self> {
        match self {
            Self::Tuple(fields) => {
                let field_len = fields.len();
                match fields.get_mut(*accessor_idx) {
                    Some(field) => Some(field),
                    None => {
                        compilation.add_error(&format!("Index {} is out of bounds for tuple with size {}", accessor_idx, field_len), None);
                        None
                    }
                }
                    
            }
            Self::Composite {  .. } => {
                compilation.add_error(&format!("Indexing composite types not yet implemented"), None);
                None
            }
            _ => {
                compilation.add_error(&format!("Can't index access a value of this type."), None);
                None
            }
        }
    }
}

