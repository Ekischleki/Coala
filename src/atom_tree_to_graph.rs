/*
Emulates and compiles an AtomTree consisting of abstract logic gates into an actual graph.
*/


use std::{collections::{HashMap, HashSet}, default};

use crate::{atom_tree::{AtomRoot, AtomTree, ValueAction}, token::AtomType};

pub struct AtomTreeCompiler {
    true_node: usize,
    false_node: usize,
    neutral_node: usize,
    tree: AtomRoot,
    nodes: Vec<Node>,
    //Maps a variable id in the Atom tree to its corrosponding node.
    variable_mappings: HashMap<usize, usize>
}
impl AtomTreeCompiler {
    pub fn new(tree: AtomRoot) -> Self {
        Self {
            true_node: 0,
            false_node: 0,
            neutral_node: 0,
            nodes: vec![],
            variable_mappings: HashMap::new(),

            tree
        }
    }

    pub fn compile(mut self) -> Vec<Node> {

        
        let true_node = Node::new(&mut self, Label::True);
        Node::force_whitelist(true_node, &mut self, vec![Label::True]);
        let false_node = Node::new(&mut self, Label::False);
        Node::force_whitelist(false_node, &mut self, vec![Label::False]);
        let neutral_node = Node::new(&mut self, Label::Neutral);
        Node::force_whitelist(neutral_node, &mut self, vec![Label::Neutral]);


        self.true_node = true_node;
        self.false_node = false_node;
        self.neutral_node = neutral_node;


        for (atom_tree, action) in self.tree.value_actions.drain(..).collect::<Vec<_>>() {
            match action {
                ValueAction::Restriction(r) => {
                    let node = self.compile_tree(&atom_tree);
                    Node::force_whitelist(node, &mut self, vec![r.into()]);
                }
                _ => todo!()
            }

        }

        for i in 0..self.nodes.len() {
            Node::connect_with_whitelist(i, &mut self);
        }

        self.nodes

    }
    fn compile_var(&mut self, var_id: usize) -> usize {
        let definition = &*self.tree.definitions.remove(&var_id).expect("Probably a selfreferential varialbe (shouldn't happen)").definition;
        match definition {
            AtomTree::SeedLabel(l) => {
                let var_node = Node::new(self, *l);
                self.variable_mappings.insert(var_id, var_node);
                var_node
            }
            AtomTree::AtomType { .. } => {
                todo!("Filter constant values.")
            }
            _ => {
                let var_node = self.compile_tree(&definition);
                self.variable_mappings.insert(var_id, var_node);
                var_node
            }
        }
    }
    fn compile_tree(&mut self, atom_tree: &AtomTree) -> usize {
        match atom_tree {
            AtomTree::Variable { id } => { //Try to find the variable node, otherwise create it.
                if let Some(var) = self.variable_mappings.get(id) {
                    return *var;
                }
                
            self.compile_var(*id)
            
        }
            AtomTree::Not(a) => {
                let input = self.compile_tree(a);
                Node::force_bool(input, self);
                let res = self.nodes[input].label.not();
                let inverted_node = Node::new(self, res);
                Node::force_bool(inverted_node, self);

                inverted_node
            }
            AtomTree::Or(a, b) => {
                let a = self.compile_tree(a);
                let b = self.compile_tree(b);
                Node::force_bool(a, self);
                Node::force_bool(b, self);
                let a_label = self.nodes[a].label;
                let b_label = self.nodes[b].label;
                let res = a_label.or(&b_label);

                let c_1 = Node::new(self, 
                    match res {
                        Label::Null => Label::Null,
                        Label::False => Label::True,
                        Label::True => Label::Neutral,
                        _ => panic!()
                    }
                );
                Node::force_bool_plus_neutral(c_1, self);
                Node::connect(a, c_1, self);
                Node::connect(b, c_1, self);

                let a_1 = Node::new(self, match (res, b_label) {
                    (Label::Null, _) => Label::Null,
                    (Label::True, Label::False) => Label::False,
                    _ => Label::Neutral
                });
                Node::force_bool_plus_neutral(a_1, self);

                Node::connect(a, a_1, self);

                let b_1 = Node::new(self, match (res, b_label) {
                    (Label::Null, _) => Label::Null,
                    (Label::True, Label::False) => Label::Neutral,
                    (Label::False, _) => Label::True,
                    _ => Label::False
                });
                Node::force_bool_plus_neutral(b_1, self);

                Node::connect(b, b_1, self);

                Node::connect(a_1, b_1, self);

                let c_2 = Node::new(self, match res {
                    Label::False => Label::Neutral,
                    Label::True => Label::False,
                    Label::Null => Label::Null,
                    _ => panic!()
                });
                Node::force_whitelist(c_2, self, vec![Label::False, Label::Neutral]);
                Node::connect(c_2,c_1, self);

                let output = Node::new(self, res);
                Node::force_bool(output, self);
                Node::connect(output, c_2, self);
                Node::connect(output, a_1, self);
                Node::connect(output, b_1, self);
                return output;

            },
            AtomTree::AtomType { .. } => panic!("Atom types should have been simplified away"),

            _ => panic!()
        }
    }
}
#[derive(Default, Debug)]

pub struct Node {
    pub connections: Vec<usize>,
    pub label: Label,
    pub label_whitelist: HashSet<Label>
}

impl Node {
    pub fn new(compiler: &mut AtomTreeCompiler, label: Label) -> usize {
        compiler.nodes.push(Node { connections: vec![], label, label_whitelist: Label::ALL_LABELS.iter().map(|i| *i).collect() });
        return compiler.nodes.len() - 1;
    }
    pub fn force_bool(v: usize, compiler: &mut AtomTreeCompiler) {
        Self::force_whitelist(v, compiler, vec![Label::True, Label::False]);
    }
    pub fn force_bool_plus_neutral(v: usize, compiler: &mut AtomTreeCompiler) {
        Self::force_whitelist(v, compiler, vec![Label::True, Label::False, Label::Neutral]);

    }
    pub fn force_whitelist(v: usize, compiler: &mut AtomTreeCompiler, whitelist: Vec<Label>) {
        let node_whitelist = &mut compiler.nodes[v].label_whitelist;
        //println!("Node whitelist: {:#?}\nRequested: {:#?}", node_whitelist, whitelist);
        let mut union = HashSet::new();
        for element in whitelist {
            if node_whitelist.contains(&element) {
                union.insert(element);
            }
        }
        if union.is_empty() {
            println!("Node restrictions made graph unsolvable.")
        }
        *node_whitelist = union;
    }
    pub fn connect_with_whitelist(v: usize, compiler: &mut AtomTreeCompiler) {
        let node_whitelist = &compiler.nodes[v].label_whitelist;

        let connect_labels = Label::ALL_LABELS
        .iter()
        .filter(|l| !node_whitelist.contains(l));

        for label in connect_labels.collect::<Vec<_>>() {
            match label {
                Label::False => Node::connect(v, compiler.false_node, compiler),
                Label::True => Node::connect(v, compiler.true_node, compiler),
                Label::Neutral => Node::connect(v, compiler.neutral_node, compiler),
                _ => {println!("Unexpected label")}
            }
        }
        

    }
    pub fn connect(a: usize, b: usize, compiler: &mut AtomTreeCompiler) {
        compiler.nodes[a].connections.push(b.clone());
        compiler.nodes[b].connections.push(a.clone());
        if  compiler.nodes[a].label == Label::Null {
            return;
        }
        if compiler.nodes[a].label == compiler.nodes[b].label {
            println!("Compilation error lead to not-solved graph")
        }
    }
}
#[derive(Debug, Default, PartialEq, Hash, Eq, Clone, Copy)]
pub enum Label {
    True,
    False,
    Neutral,
    #[default]
    Null
}

impl From<AtomType> for Label {
    fn from(value: AtomType) -> Self {
        match value {
            AtomType::False => Label::False,
            AtomType::True => Label::True,
        }
    }
}

impl Label {
    const ALL_LABELS: &[Label] = &[Label::True, Label::False, Label::Neutral];

    pub fn or(&self, other: &Self) -> Self {
        if self == &Label::Null || other == &Label::Null {
            return Label::Null;
        }
        match (self, other) {
            (Label::True, Label::True) => Label::True,
            (Label::False, Label::True)=> Label::True,
            (Label::True, Label::False)=> Label::True,
            (Label::False, Label::False)=> Label::False,

            _ => panic!("Incalid label config for or function.")
        }
    }
    pub fn and(&self, other: &Self) -> Self {
        if self == &Label::Null || other == &Label::Null {
            return Label::Null;
        }
        match (self, other) {
            (Label::True, Label::True) => Label::True,
            (Label::False, Label::True)=> Label::False,
            (Label::True, Label::False)=> Label::False,
            (Label::False, Label::False)=> Label::False,

            _ => panic!("Incalid label config for or function.")
        }
    }
    pub fn not(&self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Null => Self::Null,
            Self::Neutral => panic!("Can't invert non-boolean value")
        }
    }
}


