use std::{collections::HashMap, rc::Rc};

use crate::syntax::{SubstructureSyntax, CollectionSyntax, VariableSymbol};

pub struct GlobalSymbolTable {

}


pub struct ContextSymbolTable<'a> {
    parent: Option<&'a Self>,
    variables: HashMap<String, Rc<VariableSymbol>>,
    accessible_structures: HashMap<String, Rc<CollectionSyntax>>,
}