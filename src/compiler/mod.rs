use std::{fs::File, io::Write, path::PathBuf};

use atom_tree_to_graph::GraphColoringBackend;
use atom_tree_translate::AtomTreeTranslator;
use compilation::Compilation;
use diagnostic::Diagnostic;
use file_reader::FileReader;
use parser::Parser;
use settings::Settings;
use string_file_reader::StringFileReader;
use syntax::ImportSyntax;

use crate::compiler::{atom_tree_to_expr::ArithmeticCircuitsBackend, compiler_backend::Backend};

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
mod lib_embed;
pub mod typecheck;
pub mod scope;
pub mod atom_tree_to_expr;
pub mod compiler_backend;
pub mod pipeline_builder;
pub mod architecture;



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
pub fn to_path(base_path: &PathBuf, import_syntax: &ImportSyntax) -> Result<PathBuf, Diagnostic> {
    let mut path = base_path.clone();
    
    for part in import_syntax.path.iter().take(import_syntax.path.len() - 1) {
        path.push(part.value.to_owned());
        if !path.exists() {
            return Err(Diagnostic::new(
                diagnostic::DiagnosticType::Error,
                format!("Couldn't find part of import path"),
                part.location.clone(),
                diagnostic::DiagnosticPipelineLocation::IO
            ));
        }
    }
    let file = import_syntax.path.last().unwrap().to_owned();
    path.push(format!("{}.coala", file.value));
    if !path.is_file() {
        return Err(Diagnostic::new(
            diagnostic::DiagnosticType::Error,
            format!("Couldn't find part of import path (\"{}.coala\")", file.value),
            file.location.clone(),
            diagnostic::DiagnosticPipelineLocation::IO
        ));
    }

    Ok(path)
}
pub fn parse_file<T: FileReader>(file: &PathBuf, internal: bool, file_reader: &mut T, parser: &mut Parser, settings: &Settings) {
    println!("Reading file ({:?})...", file);
    if !internal {
        if let Err(diagnostic) = file_reader.reset_to_file(file) {
            parser.compilation.add_diagnostic(diagnostic);
            return;
        }
    }
    println!("Lexing file ({:?})...", file);
    let tokens = lexer::tokenize(file_reader, file, parser.compilation).unwrap();
    println!("Preparsing file ({:?})...", file);
    let mut tokens = block_parser::TokenBlock::from_token_stream(tokens, parser.compilation).unwrap();
    println!("Parsing file ({:?})...", file);
    parser.parse_file(&mut tokens);
}
pub fn compile(settings: &Settings) {
    println!("Loading project...");
    let project = &settings.base_path;
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
    let base_path: PathBuf = file.into();
    let mut parser = Parser::new(&mut compilation);
    let mut file_reader = StringFileReader::new();

    let mut main_file = base_path.clone();
    main_file.push("main.coala");

    if main_file.is_file() { 

        parse_file(&main_file, false, &mut file_reader,  &mut parser, settings);
    } else {
        compilation.add_error("No main file found. Please make sure your base directory contains a file called \"main.coala\"", None);
        end_compilation(settings, &compilation);
        return;
    }

    while let Some((import_syntax, _)) = parser.imports.iter().find(|f| !f.1) {
        let import_syntax = import_syntax.clone();
        *parser.imports.get_mut(&import_syntax).unwrap() = true;
        if import_syntax.path.first().unwrap().value == "std" {
            let mut path = String::new();
            import_syntax.path.iter().skip(1).for_each(|part| {
                if path.is_empty() {
                    path.push_str(&format!("{}", part.value));
                } else {
                    path.push_str(&format!("/{}", part.value));
                }
            });
            path.push_str(".coala");
            let file = lib_embed::get_std_lib_file(&path);
            if file.is_none() {
                parser.compilation.add_error(&format!("Couldn't find standard library file {path}"), None);
                continue;
            }
            let file = file.unwrap();
            file_reader.reset_to_string(&file);
            parse_file(&PathBuf::from(format!("std/{path}")), true, &mut file_reader, &mut parser, settings);
            continue;
        }
        let file = match to_path(&base_path, &import_syntax) {
            Ok(path) => path,
            Err(diagnostic) => {
                parser.compilation.add_diagnostic(diagnostic);
                continue;
            }
        };
        
        parse_file(&file, false, &mut file_reader, &mut parser, settings);
    }

    let project = parser.project;
    if settings.print_debug_logs {
        println!("Project: {:#?}", project);
    }

    if !(settings.ignore_errors || compilation.is_error_free()) {
        end_compilation(settings, &compilation);
        return;
    }
    println!("Compiling project to IR...");
    let atom_tree_translator = AtomTreeTranslator::new(&mut compilation, project.collections, project.composite_types);
    let mut atom_tree = atom_tree_translator.convert(project.problems, project.solutions);
    if settings.print_debug_logs {
        println!("{:#?}", atom_tree);
    }

    let mut backend = if settings.as_expression {
        Backend::ArithmeticCircuitsBackend(ArithmeticCircuitsBackend::new())
    } else {
        Backend::GraphColoringBackend(GraphColoringBackend::new())
    };

    let mut pipeline = pipeline_builder::IRPipelineBuilder::new();
    backend.make_ir_pipeline(&mut pipeline);
    atom_tree = pipeline.run(atom_tree, &settings, &mut compilation);
    if let Err(e) = backend.init(atom_tree) {
        compilation.add_error(&format!("Failed to initialize backend: {e}"), None);
        end_compilation(settings, &compilation);
    }

    if let Err(e) = backend.emit(PathBuf::from(&settings.output_directory)) {
        compilation.add_error(&format!("Failed to emit: {e}"), None);
        end_compilation(settings, &compilation);
    }
   
    end_compilation(settings, &compilation);
}