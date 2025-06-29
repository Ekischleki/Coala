use crate::compiler::{atom_tree::{AtomRoot, AtomTree}, atom_tree_to_graph::Node};

pub struct AtomTreeToExpr {
    output: AtomTree,
}


impl AtomTreeToExpr {
    pub fn new(output: AtomTree) -> Self {
        Self {
            output
        }
    }

    pub fn compile() {
        todo!()
    }
}