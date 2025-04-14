use std::{fs::File, io::Write, path::PathBuf};

use atom_tree_translate::AtomTreeTranslator;
use compilation::Compilation;
use file_reader::FileReader;
use parser::Parser;
use string_file_reader::StringFileReader;
use symbol_table::GlobalSymbolTable;

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
pub mod label;
pub mod graph_label_set;
pub mod symbol_table;
pub mod atom_tree;
pub mod atom_tree_translate;
pub mod atom_tree_to_graph;

fn main() {
    let mut file_reader = StringFileReader::new();
    let mut compilation = Compilation::new();
    let file = "./wip.coala".into();

    file_reader.reset_to_file(&file).unwrap();
    let mut tokens = lexer::tokenize(&mut file_reader, &file, &mut compilation).unwrap();
    println!("{:#?}", tokens);
    let mut parser = Parser::new(tokens, &mut compilation);
    parser.parse_file();
    let collections = parser.collections;
    let problems = parser.problems;
    let atom_tree_translator = AtomTreeTranslator::new(compilation, collections);
    let mut atom_tree = atom_tree_translator.convert(problems);
    println!("{:#?}", atom_tree);

    while atom_tree.remove_links() {
        println!("{:#?}", atom_tree);
    }


    while atom_tree.simp_all() {
        println!("{:#?}", atom_tree);
    }
    
    /* 
    graph_maker.output_as_adjacency_list(false);
    graph_maker.output_as_adjacency_matrix();
    let mut buf = String::new();
    graph_maker.export_as_csv(&mut buf);
    File::create("./compiled.csv").unwrap().write(buf.as_bytes()).unwrap();
    //println!("{:#?}", graph_maker.nodes);
*/
}
 