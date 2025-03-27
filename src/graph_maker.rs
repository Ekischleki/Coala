use std::{cell::RefCell, collections::{HashMap, HashSet}, fmt::format, rc::Rc};

use crate::{compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, syntax::{ArgumentSyntax, CodeSyntax, NodeValueSyntax, ProjectSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypeSyntax}, token::{AtomSub, AtomType}};

type NodeRef = usize;

#[derive(Default, Debug)]
pub struct Node {
    pub connections: Vec<usize>
}

impl Node {
    pub fn new(graph_maker: &mut GraphMaker) -> NodeRef {
        graph_maker.nodes.push(Node::default());
        return graph_maker.nodes.len() - 1;
    }

    pub fn connect(a: NodeRef, b: NodeRef, graph_maker: &mut GraphMaker) {
        graph_maker.node(a).connections.push(b.clone());
        graph_maker.node(b).connections.push(a.clone());
    }
}


pub struct GraphMaker {
    pub nodes: Vec<Node>,
    pub label_types: Vec<usize>,
    pub true_label: NodeRef,
    pub false_label: NodeRef,
    pub neutral_label: NodeRef,
    pub project: ProjectSyntax,
}

pub struct GlobalSymbolTable {

}

impl GraphMaker {

    pub fn compile(&mut self, compilation: &mut Compilation) {
        if let Some(problems) = std::mem::replace(&mut self.project.problems, Some(vec![]))  {
            for problem in problems {
                let inputs = problem.args.iter().map(|a| {
                    let node = Node::new(self);
                    force_type(node, &a.type_syntax, self);
                    node
                }).collect();
                compile_sub(problem, self, compilation, inputs);

            }
        } else {
            compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("This project doesn't define any problems, so it cannot be compiled."), None, DiagnosticPipelineLocation::Assembling));
        }
    } 

    pub fn compile_function(&mut self, location: &SubLocation, inputs: Vec<usize>, compilation: &mut Compilation) -> Option<Vec<usize>> {
        match location {
            SubLocation::Structure { collection, sub } => {
                let collection = self.project.collections.iter().find(|&f| &f.name == collection).unwrap();
                let sub = collection.subs.iter().find(|&f| &f.name == sub).unwrap();
                return compile_sub(sub.clone(), self, compilation, inputs);
            }
            SubLocation::Atom(AtomSub::Not) => {
                assert_eq!(inputs.len(), 1);
                let input = inputs[0];
                force_bool(input, self);
                let inverted_node = Node::new(self);
                force_bool(inverted_node, self);

                Node::connect(input, inverted_node, self);
                return Some(vec![inverted_node])
            }
            SubLocation::Atom(AtomSub::Or) => {
                assert_eq!(inputs.len(), 2);
                assert_eq!(self.label_types.len(), 3);
                let a = inputs[0];
                let b = inputs[1];
                force_bool(a, self);
                force_bool(b, self);

                let c_1 = Node::new(self);
                Node::connect(a, c_1, self);
                Node::connect(b, c_1, self);

                let a_1 = Node::new(self);
                Node::connect(a, a_1, self);

                let b_1 = Node::new(self);
                Node::connect(b, b_1, self);

                Node::connect(a_1, b_1, self);

                let c_2 = Node::new(self);
                Node::connect(c_2, self.true_label, self);
                Node::connect(c_2,c_1, self);

                let output = Node::new(self);
                force_bool(output, self);
                Node::connect(output, c_2, self);
                Node::connect(output, a_1, self);
                Node::connect(output, b_1, self);
                return Some(vec![output])
            }
        }

    }
    pub fn node(&mut self, idx: NodeRef) -> &mut Node {
        &mut self.nodes[idx]
    }

    pub fn new(project: ProjectSyntax) -> Self {
        let mut graph_maker =  Self { nodes: vec![], label_types: vec![], true_label: 0, false_label: 0, neutral_label: 0, project };
        graph_maker.true_label = Node::new(&mut graph_maker);
        graph_maker.false_label = Node::new(&mut graph_maker);
        graph_maker.neutral_label = Node::new(&mut graph_maker);
        Node::connect(graph_maker.true_label, graph_maker.false_label, &mut graph_maker);
        Node::connect(graph_maker.true_label, graph_maker.neutral_label, &mut graph_maker);
        Node::connect(graph_maker.false_label, graph_maker.neutral_label, &mut graph_maker);
        graph_maker.label_types = vec![graph_maker.true_label, graph_maker.false_label, graph_maker.neutral_label];
        graph_maker

        
    }
}


pub struct ContextSymbols {

}
#[derive(Clone)]
pub struct CompiledSub {
    input_ptrs: Vec<NodeRef>,
    //substructure: Vec<NodeRef>,
    output_ptr: NodeRef,
}

impl CompiledSub {
    //Insert values such that input values are in the newly input pointers, and all pointers now point to the same element.
    pub fn inline(&self, graph_maker: &mut GraphMaker, input_ptrs: Vec<usize>) -> Vec<usize> {
        todo!()
    }
}

pub struct ContextSymbolTable {
    variables: HashMap<String, NodeRef>,
}

pub fn compile_sub(sub_syntax: SubstructureSyntax, graph_maker: &mut GraphMaker, compilation: &mut Compilation, args: Vec<usize>) -> Option<Vec<usize>>{
    //setup args
    let mut variables = HashMap::new();
    map_args(&args, &sub_syntax.args, &mut variables);

    let mut context_symbol_table = ContextSymbolTable {
        variables
    };

    for code in &sub_syntax.code {
        match code {
            CodeSyntax::Let { variable, value } => {
                if let Some(value) = compile_value(value, &mut context_symbol_table, graph_maker, compilation) {
                    assert_eq!(value.len(), 1);
                    context_symbol_table.variables.insert(variable.to_owned(), value[0]);
                }
            }
            CodeSyntax::Force { value, type_syntax } => {
                if let Some(values) = compile_value(value, &context_symbol_table, graph_maker, compilation) {
                    for value in values {
                        force_type(value, type_syntax, graph_maker);
                    }
                }
            }
            _ => todo!()
        }
    }

    if let Some(res) = &sub_syntax.result {
        compile_value(res, &context_symbol_table, graph_maker, compilation)
    } else {
        Some(vec![])
    }
    
}

pub fn force_bool(value: usize, graph_maker: &mut GraphMaker) {
    Node::connect(value, graph_maker.neutral_label, graph_maker);
}

pub fn force_type(value: usize, type_syntax: &TypeSyntax, graph_maker: &mut GraphMaker) {
    let mut blacklist_types: HashSet<usize> = graph_maker.label_types.clone().into_iter().collect(); 
    match type_syntax {
        TypeSyntax::Atom(AtomType::True) => {
            blacklist_types.remove(&graph_maker.true_label);
        }
        TypeSyntax::Atom(AtomType::False) => {
            blacklist_types.remove(&graph_maker.false_label);
        }
        TypeSyntax::Defined { structure} => {
            if structure != "bool" {todo!()}
            blacklist_types.remove(&graph_maker.true_label);
            blacklist_types.remove(&graph_maker.false_label);
        }
        TypeSyntax::Set { elements } => todo!()
    }
    for t in blacklist_types  {
        Node::connect(value, t, graph_maker);
    }
}

pub fn map_args(input_args: &Vec<usize>, map_args: &Vec<ArgumentSyntax>, map: &mut HashMap<String, NodeRef>) {
    assert_eq!(map_args.len(), input_args.len());

    for i in 0..map_args.len() {
        map.insert(map_args[i].name.to_owned(), input_args[i]);
    }
}
 
pub fn compile_value(node_value_syntax: &NodeValueSyntax, context_symbol_table: &ContextSymbolTable, graph_maker: &mut GraphMaker, compilation: &mut Compilation) -> Option<Vec<NodeRef>> {
    match node_value_syntax {
        NodeValueSyntax::Literal(AtomType::False) => {
            return Some(vec![graph_maker.false_label.clone()]);
        }
        NodeValueSyntax::Literal(AtomType::True) => {
            return Some(vec![graph_maker.true_label.clone()]);
        }
        NodeValueSyntax::Variable(v) => {
            match context_symbol_table.variables.get(v) {
                None => {
                    compilation.add_diagnostic(Diagnostic::new(crate::diagnostic::DiagnosticType::Error, format!("The variable {v} doesn't exist in this context"), None, DiagnosticPipelineLocation::Assembling));
                    return None;
                }
                Some(s) => {
                    return Some(vec![s.clone()]);
                }
            }
        }
        NodeValueSyntax::Sub(sub_call) => {
            let application = match  &sub_call.application {
                Some(application) => compile_value(application, context_symbol_table, graph_maker, compilation)?,
                None => vec![]
            };

            return graph_maker.compile_function(&sub_call.location, application, compilation);
        }
        NodeValueSyntax::Tuple(t) => {
            let mut res = vec![];
            for v in t {
                res.append(&mut compile_value(v, context_symbol_table, graph_maker, compilation)?);
            }
            return Some(res);
        }
        _ => todo!()
    }
}