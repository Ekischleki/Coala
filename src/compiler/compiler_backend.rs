use std::path::PathBuf;

use crate::compiler::{atom_tree::AtomRoot, atom_tree_to_expr::ArithmeticCircuitsBackend, atom_tree_to_graph::GraphColoringBackend, pipeline_builder::IRPipelineBuilder, settings::{self, Settings}};

pub trait CompilerBackend {
    fn check_settings(&mut self, settings: Settings) -> Result<(), String>;
    fn make_ir_pipeline(&self, pipeline_builder: &mut IRPipelineBuilder);
    fn init(&mut self, im: AtomRoot) -> Result<(), String>;
    ///Compile the im and emit at the base path
    fn emit(self, base: PathBuf) -> Result<(), String>;
}


pub enum Backend {
    GraphColoringBackend(GraphColoringBackend),
    ArithmeticCircuitsBackend(ArithmeticCircuitsBackend)
}

impl Backend {
    pub fn emit(self, base: PathBuf) -> Result<(), String> {
        match self {
            Self::GraphColoringBackend(b) => b.emit(base),
            Self::ArithmeticCircuitsBackend(b) => b.emit(base),
        }
    }
    pub fn init(&mut self, im: AtomRoot) -> Result<(), String> {
        match self {
            Self::GraphColoringBackend(b) => b.init(im),
            Self::ArithmeticCircuitsBackend(b) => b.init(im)
        }
    }
    pub fn make_ir_pipeline(&self, pipeline_builder: &mut IRPipelineBuilder) {
        match self {
            Self::GraphColoringBackend(b) => b.make_ir_pipeline(pipeline_builder),
            Self::ArithmeticCircuitsBackend(b) => b.make_ir_pipeline(pipeline_builder),
        }
    }
}
