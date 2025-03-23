use std::path::PathBuf;

use compilation::Compilation;
use file_reader::FileReader;
use string_file_reader::StringFileReader;

pub mod graph_structure_type;
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
    let file = "./test.coala".into();
    file_reader.reset_to_file(&file).unwrap();
    let tokens = lexer::tokenize(&mut file_reader, &file, &mut compilation);
    println!("{:#?}", compilation.diagnostics());

    println!("{:#?}", tokens)
}
