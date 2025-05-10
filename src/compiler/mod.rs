use std::{fs::File, io::Write, path::PathBuf};

use atom_tree_to_graph::AtomTreeCompiler;
use atom_tree_translate::AtomTreeTranslator;
use compilation::Compilation;
use file_reader::FileReader;
use parser::Parser;
use settings::Settings;
use string_file_reader::StringFileReader;

pub mod code_location;
pub mod export;
pub mod diagnostic;
pub mod token;
pub mod atom_tree;
pub mod settings;
mod compilation;
mod file_reader;
mod lexer;
mod syntax;
mod string_file_reader;
mod type_stream;
mod parser;
mod atom_tree_translate;
mod atom_tree_to_graph;
mod block_parser;

fn end_compilation(settings: &Settings, compilation: &Compilation) {
    if !compilation.is_error_free() {
        println!("Compilation finished with errors.")
    } else {
        println!("Compilation finished without errors.")
    }
    if settings.output_diagnostics {
        for diagnostic in compilation.diagnostics() {
            if let Some(location) = &diagnostic.location {
                if let Some(section) = &location.section {
                    println!("File:{} C:{}-{}: {:?}: {}", 
                    location.path.to_str().unwrap_or("???"), 
                    section.location_begin, 
                    section.location_end,
                    diagnostic.diagnostic_type,
                    diagnostic.description
                    );
                } else {
                    println!("File:{}: {:?}: {}", 
                    location.path.to_str().unwrap_or("???"), 
                    diagnostic.diagnostic_type,
                    diagnostic.description
                    );
                }
            } else {
                println!("{:?}: {}", 
                diagnostic.diagnostic_type,
                diagnostic.description
                );
            }
        }
    }
}

pub fn compile(settings: &Settings) {
    println!("Loading project...");
    let project = &settings.project_path;
    let mut compilation = Compilation::new(settings.to_owned());
    let file: String;
    match project {
        Some(s) => {file = s.to_owned();}
        None => {
            compilation.add_error("No project file was specified", None);
            end_compilation(settings, &compilation);
            return;
        }

    }
    let file: PathBuf = file.into();
    
    if !file.exists() {
        compilation.add_error("Couldn't find project file", None);
        end_compilation(settings, &compilation);
        return;
    }
    
    let mut file_reader = StringFileReader::new();
    file_reader.reset_to_file(&file).unwrap();
    println!("Lexing project...");

    let tokens = lexer::tokenize(&mut file_reader, &file, &mut compilation).unwrap();
    if settings.print_debug_logs {
        println!("{:#?}", tokens);
    }

    let mut tokens = block_parser::TokenBlock::from_token_stream(tokens, &mut compilation).unwrap();
    if settings.print_debug_logs {
        println!("{:#?}", tokens);
    }
    //Brace errors tend to be heavy and have a lot of side effects, so we'll stop here if any are found to not confuse the user
    if !(settings.ignore_errors || compilation.is_error_free()) {
        end_compilation(settings, &compilation);
        return;
    }
    println!("Parsing project...");

    let mut parser = Parser::new(&mut compilation);
    parser.parse_file(&mut tokens);
    let collections = parser.collections;
    
    let problems = parser.problems;
    
    let solutions = parser.solutions;
    
    let composites = parser.composite_types;
    if settings.print_debug_logs {
        println!("Collections: {:#?}", collections);
        println!("Problems: {:#?}", problems);
        println!("Solutions: {:#?}", solutions);
        println!("Composites: {:#?}", composites);
    }

    if !(settings.ignore_errors || compilation.is_error_free()) {
        end_compilation(settings, &compilation);
        return;
    }
    println!("Compiling project to IR...");
    let atom_tree_translator = AtomTreeTranslator::new(&mut compilation, collections, composites);
    let mut atom_tree = atom_tree_translator.convert(problems, solutions);
    if settings.print_debug_logs {
        println!("{:#?}", atom_tree);
    }
    if settings.optimize {

        while atom_tree.remove_links() {
            if settings.print_debug_logs {
                println!("{:#?}", atom_tree);
            }
        }
        while atom_tree.inline_vars() {
            if settings.print_debug_logs {
                println!("{:#?}", atom_tree);
            }
        }        
        
        

        while atom_tree.simp_all(&mut compilation) {
            if settings.print_debug_logs {
                println!("{:#?}", atom_tree);
            }
            while atom_tree.remove_links() {
                if settings.print_debug_logs {
                    println!("{:#?}", atom_tree);
                }
            }
            while atom_tree.inline_vars() {
                if settings.print_debug_logs {
                    println!("{:#?}", atom_tree);
                }
            }        
        }
        if settings.heavy_optimization {
            println!("Heavy optimization enabled, inlining all...");
            atom_tree.inline_all();
            println!("Second pass of simplification...");
            while atom_tree.simp_all(&mut compilation) {
                if settings.print_debug_logs {
                    //println!("{:#?}", atom_tree);
                }    
            }
        }
        println!("Outlining common expressions");
        atom_tree.outline_common_expressions();
        if settings.print_debug_logs {
            println!("{:#?}", atom_tree);
        }
        println!("Next pass of simplification...");
        while atom_tree.simp_all(&mut compilation) {
                if settings.print_debug_logs {
                    println!("{:#?}", atom_tree);
                }
                while atom_tree.remove_links() {
                    if settings.print_debug_logs {
                        println!("{:#?}", atom_tree);
                    }
                }
                while atom_tree.inline_vars() {
                    if settings.print_debug_logs {
                        println!("{:#?}", atom_tree);
                    }
                }        
            }
        
    }
    println!("Finalizing IR simplification...");
    atom_tree.finalize_simp();
    if settings.print_debug_logs {
        println!("{:#?}", atom_tree);
    }
    println!("Compiling and running IR...");

    let atom_tree_compiler = AtomTreeCompiler::new(atom_tree);
    let nodes = atom_tree_compiler.compile();
    //println!("{:#?}:{}", nodes, nodes.len());
    println!("Exporting compilation results...");

    let mut buf_edges = String::new();
    let mut buf_labels = String::new();

    export::export_as_csv(&nodes, &mut buf_edges, &mut buf_labels);
    File::create("./compiled_edges.csv").unwrap().write(buf_edges.as_bytes()).unwrap();
    File::create("./compiled_labels.csv").unwrap().write(buf_labels.as_bytes()).unwrap();

    end_compilation(settings, &compilation);
}