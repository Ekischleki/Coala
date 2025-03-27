use std::path::PathBuf;

use enum_as_inner::EnumAsInner;



use super::code_location::CodeLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    code_location: CodeLocation,
    token_type: TokenType
}
impl Token {


    pub fn to(&self, end: &Self) -> CodeLocation {
        let start = &self.code_location;
        let end = &end.code_location;
        start.to(end)
    }

    pub fn new(token_type: TokenType, code_location: CodeLocation) -> Self {
        Self {
            token_type,
            code_location
        }
    }

    pub fn eof(path: &PathBuf) -> Self {
        Self {
            token_type: TokenType::EOF,
            code_location: CodeLocation::new(path.to_owned())
        }
    }
    
    pub fn code_location(&self) -> &CodeLocation {
        &self.code_location
    }
    
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn into_token_type(self) -> TokenType {
        self.token_type
    }
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum TokenType {
    Keyword(Keyword),
    ThickArrowRight,
    ThinArrowRight,
    Delimiter(Delimiter),
    Identifier(String),
    Atom(Atom),
    EOF,
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum Atom {
    Type(AtomType),
    Sub(AtomSub),
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum AtomSub { //Atomic submarine
    Or,
    Not
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum AtomType {
    True,
    False,
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum Keyword {
    Let,
    Takes,
    Force,
    Own,
    
    Problem,
    Structure,
    SubStructure,
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum Delimiter {
    Brace(Brace),
    Colon,
    DoubleColon,
    Comma,
    Semicolon,
    Equals
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum Brace {
    Round(BraceState),
    Curly(BraceState),
    Square(BraceState)
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum BraceState {
    Open,
    Closed
}