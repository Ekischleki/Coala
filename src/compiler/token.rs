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
    
    Delimiter(Delimiter),
    Identifier(String),
    Atom(Atom),
    Integer(usize),
    String(String),
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

#[derive(Debug, Clone, Copy, EnumAsInner, PartialEq, Eq, Hash)]
pub enum AtomType {
    True,
    False,
}
impl AtomType {
    pub fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
        }
    }
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum Keyword {
    If,
    Let,
    Else,
    Super,  
    Force,  
    Output,
    Problem,
    Solution,
    Composite,
    Collection,
    SubStructure,
} //Sorting this by length was not intentional, but lets go
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum Delimiter {
    Brace(Brace, BraceState),
    Colon,
    DoubleColon,
    Comma,
    Period,
    Semicolon,
    Equals,
    ThickArrowRight,
    ThinArrowRight,
}
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq, Copy)]

pub enum Brace {
    Round,
    Curly,
    Square
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]

pub enum BraceState {
    Open,
    Closed
}