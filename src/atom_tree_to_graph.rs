use std::default;

use crate::{atom_tree::AtomRoot, token::AtomType};

pub struct AtomTreeCompiler {
    tree: AtomRoot,
    input_values: Vec<AtomType>,
    pub nodes: Vec<Node>,

}

#[derive(Default)]

pub struct Node {
    pub connections: Vec<usize>,
    pub color: Label,
}

impl Node {
    pub fn new(compiler: &mut AtomTreeCompiler) -> usize {
        compiler.nodes.push(Node::default());
        return compiler.nodes.len() - 1;
    }

    pub fn connect(a: usize, b: usize, compiler: &mut AtomTreeCompiler) {
        compiler.nodes[a].connections.push(b.clone());
        compiler.nodes[b].connections.push(a.clone());
    }
}
#[derive(Default)]
pub enum Label {
    True,
    False,
    Neutral,
    #[default]
    Null
}



impl AtomTreeCompiler {
    pub fn compile(mut self) {
        //let true_value = Node::new(compiler)
    }
}