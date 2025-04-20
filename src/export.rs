use std::collections::HashSet;

use crate::atom_tree_to_graph::{Label, Node};

pub fn export_as_csv(nodes: &Vec<Node>, buf_edges: &mut String, buf_labels: &mut String) {
    buf_labels.push_str("Id,Label,Color\n");
    for (i, node) in nodes.iter().enumerate() {
        match node.label {
            Label::False => buf_labels.push_str(&format!("{i},FALSE,#FF0000\n")),
            Label::True => buf_labels.push_str(&format!("{i},TRUE,#00FF00\n")),
            Label::Neutral => buf_labels.push_str(&format!("{i},NEUTRAL,#0000FF\n")),
            Label::Null => buf_labels.push_str(&format!("{i},NULL,#555555\n")),

        }
        let set: HashSet<_> = node.connections.iter().collect();
        buf_edges.push_str(&format!("{i}")); 
        for &elem in set {
            buf_edges.push_str(&format!(";{elem}"));
        }
        buf_edges.push('\n');
    }
}