use std::{hash::Hash, rc::Rc};

use enum_as_inner::EnumAsInner;

use crate::token::{AtomSub, AtomType};


#[derive(Default, Debug)]
pub struct CollectionSyntax {
    pub subs: Vec<SubstructureSyntax>,
    pub name: String,
}


#[derive(Default, Debug, Clone)]

pub struct SubstructureSyntax {
    pub name: String,
    pub args: Vec<ArgumentSyntax>,
    pub code: Vec<CodeSyntax>,
    pub result: Option<NodeValueSyntax>
}

impl Hash for SubstructureSyntax {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for SubstructureSyntax {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for SubstructureSyntax {}

#[derive( Debug, Clone)]

pub enum CodeSyntax {
    Let{
        variable: String,
        value: NodeValueSyntax,
    },

    Force {
        value: NodeValueSyntax,
        type_syntax: TypeSyntax,
    },
    Sub(SubCallSyntax),

}

#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeSyntax {
    Atom(AtomType),
    Defined {
        structure: String
    },
    Set {
        elements: Vec<TypeSyntax>
    }
}
#[derive(Debug, Clone)]

pub enum NodeValueSyntax {
    Tuple(Vec<Self>),
    Variable(String),
    Sub(Box<SubCallSyntax>),
    Literal(AtomType)
}


#[derive(Debug, Clone)]

pub struct SubCallSyntax {
    pub location: SubLocation,
    pub application: Option<NodeValueSyntax>,
}
#[derive(Debug, Clone)]
pub enum SubLocation {
    Structure {
        collection: String,
        sub: String,
    },
    Atom(AtomSub)
}

#[derive( Debug, Clone)]

pub struct ArgumentSyntax {
    pub name: String,
    pub type_syntax: TypeSyntax
}