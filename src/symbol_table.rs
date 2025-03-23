use std::{collections::HashMap, rc::Rc};

use crate::graph_structure_type::{StructureSubSymbol, StructureSymbol, VariableSymbol};

pub struct GlobalSymbolTable {

}


pub struct ContextSymbolTable<'a> {
    parent: Option<&'a Self>,
    variables: HashMap<String, Rc<VariableSymbol>>,
    accessible_structures: HashMap<String, Rc<StructureSymbol>>,
}