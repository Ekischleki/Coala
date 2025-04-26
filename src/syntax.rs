use std::{collections::HashMap, hash::Hash, rc::Rc};

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
    pub args: Vec<TypedIdentifierSyntax>,
    pub code: Vec<CodeSyntax>,
    pub result: Option<ExpressionSyntax>
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
        value: ExpressionSyntax,
    },

    Force {
        value: ExpressionSyntax,
        type_syntax: TypeSyntax,
    },
    Sub(SubCallSyntax),

}




#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeSyntax {
    Atom(AtomType),
    Set {
        elements: Vec<TypeSyntax>
    },
    Composite {
        name: String,
    }
}
#[derive(Debug, Clone)]

pub enum ExpressionSyntax {
    Tuple(Vec<Self>),
    Variable(String),
    Access{
        base: Box<Self>,
        field: String,
    },
    AccessIdx {
        base: Box<Self>,
        idx: usize,
    },
    Sub(Box<SubCallSyntax>),
    Literal(AtomType),
    CompositeConstructor{
        type_name: String,
        field_assign: Vec<FieldAssignSyntax>
    }
}


#[derive(Debug, Clone)]

pub struct SubCallSyntax {
    pub location: SubLocation,
    pub application: Option<ExpressionSyntax>,
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

pub struct TypedIdentifierSyntax {
    pub name: String,
    pub type_syntax: TypeSyntax
}

#[derive( Debug, Clone)]

pub struct FieldAssignSyntax {
    pub left: String,
    pub right: ExpressionSyntax,
}

#[derive( Debug, Clone)]
pub struct CompositeTypeSyntax {
    pub name: String,
    pub fields: Vec<TypedIdentifierSyntax>
}