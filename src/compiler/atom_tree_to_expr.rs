use std::{collections::{btree_map::IterMut, HashSet}, fmt::Write, fs::File, hash::Hash, os::unix::fs::FileExt};

use crate::compiler::{atom_tree::{AtomRoot, AtomTree, ValueAction}, atom_tree_to_graph::Node, atom_tree_translate::ValueCollection, compiler_backend::CompilerBackend};

pub struct ArithmeticCircuitsBackend {
    exported_vars: HashSet<usize>,
    var_definitions: Vec<String>,
    atom_tree: Option<AtomRoot>
}


impl CompilerBackend for ArithmeticCircuitsBackend {
    fn init(&mut self, im: AtomRoot) -> Result<(), String> {
        self.atom_tree = Some(im);
        Ok(())
    }

    fn emit(self, mut base: std::path::PathBuf) -> Result<(), String> {
        base.push("arit_circ.txt");
        let file = File::open(base).map_err(|e| e.to_string())?;
        file.write_all_at(self.compile().as_bytes(), 0).map_err(|e| e.to_string())
    }
    
    fn check_settings(&mut self, settings: super::settings::Settings) -> Result<(), String> {
        Ok(())
    }
    
    fn make_ir_pipeline(&self, pipeline_builder: &mut super::pipeline_builder::IRPipelineBuilder) {
        pipeline_builder
        .optimize_simp(false)
        .inline_vars(false)
        .change_op_or_and()
        .optimize_simp(false)
        .outline_expr();
    }
}

impl ArithmeticCircuitsBackend {
    pub fn new() -> Self {
        //atom_tree.outline_common_expressions();

        //println!("{:#?}", atom_tree);
        Self {
            atom_tree: None,
            exported_vars: HashSet::new(),
            var_definitions: vec![],
        }
    }

    pub fn compile(mut self) -> String {
        

        let mut eval = String::new();
        for (i, (item, _)) in self.atom_tree.clone().unwrap().value_actions.iter().filter(|v| if let ValueAction::PartialOut = v.1 {true} else {false}).enumerate() {

            let element = format!("(({}%2)<<{i})", self.compile_tree(item));
            if i != 0 {
                eval.push('|');
            }
            eval.write_str(&element);
            
        }
        let mut var_defs = String::new();
        for var_def in self.var_definitions {
            var_defs.push_str(&format!("{var_def}\n"));
        }
        format!("{var_defs}\n{eval}")

    }

            
    

    pub fn compile_tree(&mut self, tree: &AtomTree) -> String {
        match tree {
            AtomTree::Not(input) => {
                let a = self.compile_tree(input);
                format!("({a}+1)") //Apparently the server dont like ⊻
            }
            AtomTree::Op(a) => { //Op expected to be and
                let mut res = format!("");
                let mut elements: Vec<_> = a.iter().map(|e| self.compile_tree(e)).collect();
                match elements.len() {
                    0 => return format!("0"),
                    1 => return elements.pop().unwrap(),
                    _ => {}
                }
                for i in 0..elements.len() {
                    if i != 0 {
                        res.write_str("*" );
                    }
                    let e = &elements[i];
                    res.write_str(&e);
                }
                // res.write_str(")");
                res
            }
            AtomTree::SeedLabel(s) => {
                if s.is_true() {
                    format!("1")
                } else {
                    format!("0")
                }
            }
            AtomTree::AtomType{atom: s} => {
                if s.is_true() {
                    format!("1")
                } else {
                    format!("0")
                }
            }
            AtomTree::Variable { id } => {
                if !self.exported_vars.contains(id) {
                    let var_def = self.compile_tree(&self.atom_tree.as_ref().unwrap().definitions.get(id).unwrap().definition.clone());
                    self.var_definitions.push(format!("a{id} := {var_def}"));
                    self.exported_vars.insert(*id);
                }
                format!("a{id}")
            }
            _ => todo!("{:#?}", tree)
        }
    }
}