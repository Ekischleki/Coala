use std::{env::args, fs::File, io::Write, path::PathBuf};

use atom_tree_to_graph::AtomTreeCompiler;
use atom_tree_translate::AtomTreeTranslator;
use compilation::Compilation;
use file_reader::FileReader;
use parser::Parser;
use string_file_reader::StringFileReader;

pub mod syntax;
//pub mod graph_maker;
pub mod code_location;
pub mod compilation;
pub mod diagnostic;
pub mod file_reader;
pub mod lexer;
pub mod string_file_reader;
pub mod token;
pub mod type_stream;
pub mod parser;
pub mod atom_tree;
pub mod atom_tree_translate;
pub mod atom_tree_to_graph;
pub mod export;
pub mod block_parser;

fn main() {
    let mut args = args();

    let file = if args.len() != 2 {
        println!("Expected one input file path to compile.");
        "./wip.coala".into()
    } else {
        args.next();
        args.next().unwrap()
    };
    let mut file_reader = StringFileReader::new();
    let mut compilation = Compilation::new();
    let file: PathBuf = file.into();

    if !file.exists() {
        println!("Couldn't find file.");
        return;
    }

    file_reader.reset_to_file(&file).unwrap();
    let mut tokens = lexer::tokenize(&mut file_reader, &file, &mut compilation).unwrap();
    println!("{:#?}", tokens);

    let mut tokens = block_parser::TokenBlock::from_token_stream(tokens, &mut compilation).unwrap();
    println!("{:#?}", tokens);

    let mut parser = Parser::new(&mut compilation);
    parser.parse_file(&mut tokens);
    let collections = parser.collections;
    println!("Collections: {:#?}", collections);

    let problems = parser.problems;
    println!("Problems: {:#?}", problems);

    let solutions = parser.solutions;
    println!("Solutions: {:#?}", solutions);

    let composites = parser.composite_types;
    println!("Composites: {:#?}", composites);

    let atom_tree_translator = AtomTreeTranslator::new(&mut compilation, collections, composites);
    let mut atom_tree = atom_tree_translator.convert(problems, solutions);
    println!("{:#?}", atom_tree);

    while atom_tree.remove_links() {
        println!("{:#?}", atom_tree);
    }

    while atom_tree.inline_vars() {
        println!("{:#?}", atom_tree);
    } 

    while atom_tree.simp_all(&mut compilation) {
        println!("{:#?}", atom_tree);
        while atom_tree.remove_links() {
            println!("{:#?}", atom_tree);
        }
        while atom_tree.inline_vars() {
            println!("{:#?}", atom_tree);
        }        

    }

    atom_tree.finalize_simp();
    println!("{:#?}", atom_tree);

    let atom_tree_compiler = AtomTreeCompiler::new(atom_tree);
    let nodes = atom_tree_compiler.compile();
    println!("{:#?}:{}", nodes, nodes.len());

    let mut buf_edges = String::new();
    let mut buf_labels = String::new();

    export::export_as_csv(&nodes, &mut buf_edges, &mut buf_labels);
    File::create("./compiled_edges.csv").unwrap().write(buf_edges.as_bytes()).unwrap();
    File::create("./compiled_labels.csv").unwrap().write(buf_labels.as_bytes()).unwrap();

    println!("{:#?}",  compilation.diagnostics())
    
    /* 
    graph_maker.output_as_adjacency_list(false);
    graph_maker.output_as_adjacency_matrix();
    let mut buf = String::new();
    graph_maker.export_as_csv(&mut buf);
    File::create("./compiled.csv").unwrap().write(buf.as_bytes()).unwrap();
    //println!("{:#?}", graph_maker.nodes);
*/
}
 