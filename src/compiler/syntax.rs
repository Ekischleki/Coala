use std::{hash::Hash, ops::Sub};

use enum_as_inner::EnumAsInner;

use crate::compiler::token::{AtomSub, AtomType};

use super::code_location::{CodeLocation, LocationValue};

#[derive(Default, Debug)]
pub struct CollectionSyntax {
    pub subs: Vec<SubstructureSyntax>,
    pub name: LocationValue<String>,
}

#[derive(Default, Debug, Clone)]
pub struct SubstructureSyntax {
    pub name: LocationValue<String>,
    pub args: Vec<TypedIdentifierSyntax>,
    pub code: Vec<CodeSyntax>,
    pub result: Option<ExpressionSyntax>
}

impl Hash for SubstructureSyntax {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.value.hash(state);
    }
}

impl PartialEq for SubstructureSyntax {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl Eq for SubstructureSyntax {}

#[derive(Debug, Clone)]
pub enum CodeSyntax {
    ReassignSyntax {
        variable: ExpressionSyntax,
        value: ExpressionSyntax
    },
    For {
        iterator_variable: LocationValue<String>,
        iterator_amount: ExpressionSyntax,
        iterator_body: Vec<CodeSyntax>
    },
    If {
        condition: ExpressionSyntax,
        condition_true: Vec<CodeSyntax>
    },
    IfElse {
        condition: ExpressionSyntax,
        condition_true: Vec<CodeSyntax>,
        condition_false: Vec<CodeSyntax>
    },
    Let {
        variable: LocationValue<String>,
        value: ExpressionSyntax,
    },
    Force {
        value: ExpressionSyntax,
        type_syntax: TypeSyntax,
    },
    Sub(SubCallSyntax),
    Output {
        expression: ExpressionSyntax
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum TypeSyntax {
    Atom(AtomType),
    Set {
        elements: Vec<TypeSyntax>
    },
    Composite {
        name: LocationValue<String>,
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionSyntax {
    String(LocationValue<String>),
    Int(LocationValue<usize>),
    Tuple(Vec<ExpressionSyntax>),
    Array(Vec<ExpressionSyntax>),
    LengthArray {
        count: Box<ExpressionSyntax>,
        base: Box<ExpressionSyntax>
    },
    Variable(LocationValue<String>),
    Access {
        base: Box<ExpressionSyntax>,
        field: LocationValue<String>,
    },
    AccessIdx {
        base: Box<ExpressionSyntax>,
        idx: LocationValue<usize>,
    },
    IndexOp {
        base: Box<ExpressionSyntax>,
        index: Box<ExpressionSyntax>
    },
    Sub(Box<SubCallSyntax>),
    Literal(LocationValue<AtomType>),
    CompositeConstructor {
        type_name: LocationValue<String>,
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
        collection: LocationValue<String>,
        sub: LocationValue<String>,
    },
    Super(LocationValue<String>),
    Atom(LocationValue<AtomSub>)
}
impl SubLocation {
    pub fn code_location(&self) -> Option<CodeLocation> {
        match self {
            SubLocation::Structure { collection, sub } => Some(collection.location.as_ref().unwrap_or(sub.location.as_ref()?).to(sub.location.as_ref()?)),
            SubLocation::Super(location) => location.location.clone(),
            SubLocation::Atom(location) => location.location.clone()
        }
    }
}
#[derive(Debug, Clone)]
pub struct TypedIdentifierSyntax {
    pub name: LocationValue<String>,
    pub type_syntax: TypeSyntax
}

#[derive(Debug, Clone)]
pub struct FieldAssignSyntax {
    pub left: LocationValue<String>,
    pub right: ExpressionSyntax,
}

#[derive(Debug, Clone)]
pub struct CompositeTypeSyntax {
    pub name: LocationValue<String>,
    pub fields: Vec<TypedIdentifierSyntax>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImportSyntax {
    pub path: Vec<LocationValue<String>>,
}