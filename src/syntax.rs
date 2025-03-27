use std::{hash::Hash, rc::Rc};

use crate::{graph_label_set::LabelSet, token::{AtomSub, AtomType}};

#[derive(Default, Debug)]
pub struct ProjectSyntax {
    pub problems: Option<Vec<SubstructureSyntax>>,
    pub collections: Vec<CollectionSyntax>

}

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
    LetUnpack {
        variables: Vec<VariableSymbol>,
        value: NodeValueSyntax,
    },
    Force {
        value: NodeValueSyntax,
        type_syntax: TypeSyntax,
    },
    Sub(SubCallSyntax),

}

#[derive(Debug, Clone)]
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

#[derive(Default, Debug, Clone)]

pub struct VariableSymbol {
    pub name: String,
    pub r#type: Rc<LabelSet>
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