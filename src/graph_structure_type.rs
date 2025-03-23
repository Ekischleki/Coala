use std::rc::Rc;

use crate::graph_label_set::LabelSet;
#[derive(Default)]
pub struct StructureSymbol {
    subs: Vec<Rc<StructureSubSymbol>>

}



pub struct StructureSubSymbol {
    args: Vec<ArgumentSyntax>

}

pub enum CodeSyntax {
    Let{
        variable: Rc<VariableSymbol>,
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

pub enum NodeValueSyntax {
    Tuple(Vec<Self>),
    Variable(String),
    Sub(Box<SubCallSyntax>)
}


pub struct VariableSymbol {
    name: String,
    r#type: Rc<LabelSet>
}


pub struct SubCallSyntax {
    pub structure: String,
    pub sub: String,
    pub application: Option<NodeValueSyntax>,
}




pub struct ArgumentSyntax {
    name: String,
    r#type: Rc<LabelSet> 
}