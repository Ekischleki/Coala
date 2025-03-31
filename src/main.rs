use std::{fs::File, io::Write, path::PathBuf};

use compilation::Compilation;
use file_reader::FileReader;
use graph_maker::GraphMaker;
use parser::Parser;
use string_file_reader::StringFileReader;
use symbol_table::GlobalSymbolTable;
use syntax::ProjectSyntax;

pub mod syntax;
pub mod graph_maker;
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

fn main() {
    let mut file_reader = StringFileReader::new();
    let mut compilation = Compilation::new();
    let file = "./wip.coala".into();

    file_reader.reset_to_file(&file).unwrap();
    let mut tokens = lexer::tokenize(&mut file_reader, &file, &mut compilation).unwrap();
    println!("{:#?}", tokens);
    let mut parser = Parser::new(tokens, &mut compilation);
    parser.parse_file();
    let mut graph_maker = GraphMaker::new(ProjectSyntax {
        collections: parser.collections,
        problems: Some(parser.problems)
    });
    graph_maker.compile(&mut compilation);
    println!("{:#?}", compilation.diagnostics());
    graph_maker.output_as_adjacency_list(false);
    graph_maker.output_as_adjacency_matrix();
    let mut buf = String::new();
    graph_maker.export_as_csv(&mut buf);
    File::create("./compiled.csv").unwrap().write(buf.as_bytes()).unwrap();
    //println!("{:#?}", graph_maker.nodes);

}
 