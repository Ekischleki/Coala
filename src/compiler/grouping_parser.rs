use crate::{compilation::Compilation, token::Token, type_stream::TypeStream};

pub fn group_structures(compilation: &mut Compilation, tokens: TypeStream<Token>) -> Vec<RawStructure> {
    let mut res = vec![];
    
}

pub fn group_sub(compilation: &mut Compilation, tokens: TypeStream<Token>) {

}

pub struct RawStructure {
    structure: Token,
    identifier: Token,
    open_brace: Token,
    contents: Vec<Token>,
    closed_brace: Token,
}