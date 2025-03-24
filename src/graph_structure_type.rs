use std::rc::Rc;

use crate::graph_label_set::LabelSet;
#[derive(Default, Debug)]
pub struct StructureSymbol {
    subs: Vec<Rc<StructureSubSymbol>>

}


#[derive(Default, Debug)]

pub struct StructureSubSymbol {
    args: Vec<ArgumentSyntax>

}
#[derive( Debug)]

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
        r#type: Rc<LabelSet>,
    },
    Sub(SubCallSyntax),

}
#[derive(Debug)]

pub enum NodeValueSyntax {
    Tuple(Vec<Self>),
    Variable(String),
    Sub(Box<SubCallSyntax>)
}

#[derive(Default, Debug)]

pub struct VariableSymbol {
    name: String,
    r#type: Rc<LabelSet>
}

#[derive(Default, Debug)]

pub struct SubCallSyntax {
    pub structure: String,
    pub sub: String,
    pub application: Option<NodeValueSyntax>,
}



#[derive(Default, Debug)]

pub struct ArgumentSyntax {
    name: String,
    r#type: Rc<LabelSet> 
}