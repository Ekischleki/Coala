use crate::compiler::{atom_tree::{AtomRoot, AtomTree}, compilation::Compilation, settings::Settings, token::Atom};

pub struct IRPipelineBuilder {
    pipeline: Vec<Box<dyn Fn (AtomRoot, &Settings, &mut Compilation) -> AtomRoot>>
}



impl IRPipelineBuilder {
    pub fn new() -> Self {
        Self { pipeline: vec![] }
    }
    pub fn run(&self, mut im: AtomRoot, s: &Settings, c: &mut Compilation ) -> AtomRoot {
        for p in &self.pipeline {
            im = p(im, s, c);
        }
        im
    }
    pub fn push_pipeline(&mut self, f:Box<dyn Fn (AtomRoot, &Settings, &mut Compilation) -> AtomRoot>) {
        self.pipeline.push(f);
    }
    pub fn optimize_simp(&mut self, forced: bool) -> &mut Self {
        self.push_pipeline(Box::new(move |mut im, settings, compilation| {
            if forced || settings.optimize {

                while im.remove_links() {
                    if settings.print_debug_logs {
                        println!("{:#?}", im);
                    }
                }

                let mut i = 0;

                while im.simp_all(compilation, false) {
                    println!("Optimizing, simplifying {i}. run, {} definitions and {} value actions...", im.definitions.len(), im.value_actions.len());

                    if settings.print_debug_logs {
                        println!("{:#?}", im);
                    }
                    while im.remove_links() {
                        if settings.print_debug_logs {
                            println!("{:#?}", im);
                        }
                    }        
                    i += 1;
                }
            }
            im
        }));
        self
    }
    pub fn change_op_or_and(&mut self) -> &mut Self {
        self.push_pipeline(Box::new(move |mut im, _settings, _compilation| {
            im.or_to_and();
            im
        }));
        self
    }
    pub fn inline_vars(&mut self, forced: bool) -> &mut Self {
        self.push_pipeline(Box::new(move |mut im, settings, compilation| {
            if settings.heavy_optimization || forced {
                println!("Heavy optimization enabled, inlining all...");
                im.inline_all();
                println!("Next pass of simplification...");
                while im.simp_all(compilation, false) {
                    if settings.print_debug_logs {
                        println!("{:#?}", im);
                    }    
                }
            } else {
                println!("Heavy optimization disabled, skipping pipeline step.");
            }
            im
        }));
        self
    }
    pub fn outline_expr(&mut self) -> &mut Self {
        self.push_pipeline(Box::new(move |mut im, _settings, _compilation| {
            im.outline_common_expressions();
            im
        }));
        self
    }
}


