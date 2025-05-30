use std::{collections::{HashMap, HashSet}, hash::{BuildHasher, Hash, Hasher}};

use enum_as_inner::EnumAsInner;

use crate::compiler::{atom_tree_to_graph::Label, compilation::Compilation, token::AtomType};
#[derive(Debug, Clone, Default)]

pub struct AtomRoot {
    pub definitions: HashMap<usize, VarDefinition>,
    variable_id: usize,
    pub value_actions: Vec<(AtomTree, ValueAction)>
}

#[derive(Debug, Clone)]
pub enum ValueAction {
    Output(Vec<String>, Vec<AtomTree>),
    Restriction(AtomType)
}

impl AtomRoot {
    pub fn inline_all(&mut self) {
        let mut new_definitions = HashMap::new();
        let old_definitions = self.definitions.clone();
        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.inline_all(&mut new_definitions, &old_definitions);
        });
        self.definitions = self.definitions.drain().filter(|(_, definition)| definition.definition.is_seed_label()).collect();
        
    }
    //Run after definitions have been simplified 
    pub fn simp_force(&mut self, compilation: &mut Compilation, changed: &mut bool) {
        let mut remove_indecies = vec![];
        let mut add_items = vec![];
        //For if we have force a => true, we can for the entirety of the program assume that a is true, because the graph wouldn't be solvable anyways if it wasn't. We just have to not replace it for the restriction
        let mut inline_definitions = HashMap::new();
        for (i, (value, action)) in self.value_actions.iter_mut().enumerate() {
            if let ValueAction::Restriction(t) = action {
                match value {
                    AtomTree::Variable { id } => {
                        inline_definitions.insert(AtomTree::Variable { id: *id }, Box::new(AtomTree::AtomType { atom: *t }));
                        *value = AtomTree::DoNotRemoveMarker(Box::new(AtomTree::Variable { id: *id }));
                        //*changed = true; We cant set changed to true here, because it would result in an infinite loop
                    }
                    //force not a => atom type <=> force a => not atom type
                    AtomTree::Not(a) => {
                        *changed = true;
                        
                        *t = t.not();
                        *value = *(a).clone();
                    }

                    AtomTree::Or(s) if t.is_false() => {
                        *changed = true;
                        remove_indecies.push(i);
                        for a in s {
                            add_items.push((a.to_owned(), ValueAction::Restriction(AtomType::False)));

                        }
                    }
                    //something like force true => true
                    AtomTree::AtomType { atom } => {
                        if atom == t {
                            //compilation.add_info("Force statement is trivially always successful.", None);
                            remove_indecies.push(i);
                        } else {
                            compilation.add_warning("Force statement makes graph trivially unsolvable.", None);
                        }
                    }

                    _ => {}
                }
            }
        }
        for remove in remove_indecies.iter().rev() {
            self.value_actions.remove(*remove);
        }
        self.value_actions.append(&mut add_items);

        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.inline_var(&inline_definitions);
        });
        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.remove_marker();
        });
    }

    pub fn simp_all(&mut self, compilation: &mut Compilation) -> bool {
        let mut changed = false;
        self.simp_force(compilation, &mut changed);
        //println!("Simp force: {:#?}", self);
        self.apply_to_all_trees(|t| {
            t.simp_rec(&mut changed)
        });

        return changed;
    }
    pub fn outline_common_expressions(&mut self) {
        let mut outlined = HashMap::new();
        let mut var_id = self.variable_id;
        self.apply_to_all_trees_mut(|t| {
            if t.is_seed_label() {
                return;
            }
            t.outline_common_expressions(&mut outlined, &mut var_id);
        });
        for (def, id) in outlined {
            self.definitions.insert(id, VarDefinition { id, definition: def.into() });
        }
    }
    pub fn finalize_simp(&mut self) {
        self.apply_to_all_trees_mut(|t| {
            t.remove_marker();

        });


    }

    pub fn remove_links(&mut self) -> bool {
        let mut inline_definitions = HashMap::new();
        let mut exclude_for_now = HashSet::new(); //To stop errors, we can only replace one definition at a time.
        for (id, definition) in &self.definitions {
            if exclude_for_now.contains(id) {continue;}
            match &*definition.definition {
                AtomTree::Variable { id: v } => {
                    if inline_definitions.contains_key(&AtomTree::Variable { id: *v }) {continue;}

                    inline_definitions.insert(AtomTree::Variable { id: *id }, definition.definition.to_owned());
                    exclude_for_now.insert(*v);
                }
                AtomTree::AtomType { .. } => {
                    inline_definitions.insert(AtomTree::Variable { id: *id }, definition.definition.to_owned());
                    
                }
                _ => {}
            }
        }
        //println!("{:#?}", inline_definitions);
        if inline_definitions.len() == 0 {
            return false;
        }

        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.inline_var(&inline_definitions);
        });
        for (removed_id, _) in inline_definitions {
            self.definitions.remove(&removed_id.into_variable().unwrap());
        }

        true
    }
    
    fn apply_to_all_trees_mut<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut AtomTree)
    {
        for (_, definition) in self.definitions.iter_mut() {
            predicate(&mut definition.definition);
        }
        for (restriction, _) in self.value_actions.iter_mut() {
            predicate(restriction)
        }
    }
    
    fn apply_to_all_trees<F>(&mut self, mut predicate: F)
    where
        F: FnMut(AtomTree) -> AtomTree
    {
        let mut new_definitions = HashMap::new();
        for (id, definition) in self.definitions.drain() {
            new_definitions.insert(id, VarDefinition {definition: Box::new(predicate(*definition.definition)), id});
        }
        self.definitions = new_definitions;
        let mut new_restrictions = Vec::with_capacity(self.value_actions.len());
        for (restriction, restriction_type) in self.value_actions.drain(..) {
            new_restrictions.push((predicate(restriction), restriction_type))
        }
        self.value_actions = new_restrictions;
    }
    pub fn define_new_var(&mut self, value: AtomTree) -> usize {
        let id = self.variable_id;
        self.variable_id += 1;
        self.definitions.insert(id, VarDefinition { id, definition: value.into()});
        id
    }
    pub fn define_restriction(&mut self, value: AtomTree, t: AtomType) {
        self.value_actions.push((value, ValueAction::Restriction(t)));
    }
}
#[derive(Debug, Clone)]

pub struct VarDefinition {
    pub id: usize,
    pub definition: Box<AtomTree>
}



#[derive(EnumAsInner, Debug, Clone)]
pub enum AtomTree {
    SeedLabel(Label),
    //Can be safely removed after simplifying is done.
    DoNotRemoveMarker(Box<AtomTree>),
    Variable {
        id: usize,
    },
    AtomType {
        atom: AtomType
    },
    Not(
       Box<AtomTree> 
    ),
    Or (
        Vec<AtomTree>
    )
    
}
impl Hash for AtomTree {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Variable { id } => id.hash(state),
            Self::AtomType { atom } => atom.hash(state),
            Self::Not(a) => a.hash(state),
            Self::Or(v) => {
                //Hash the vector in a way that is not order dependent
                let mut hash = 0;
                for a in v {
                    hash ^= {
                        let mut s = std::hash::BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().build_hasher();
                        a.hash(&mut s);
                        s.finish() as i32
                    };
                }
                hash.hash(state);
            },
            _ => {}
        }
    }
}
impl Eq for AtomTree {
}
impl PartialEq for AtomTree {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Variable { id: a }, Self::Variable { id: b }) => a == b,
            (Self::AtomType { atom: a }, Self::AtomType { atom: b }) => a == b,
            (Self::Not(a), Self::Not(b)) => a == b,
            (Self::Or(a), Self::Or(b)) => {
                //Check of the two vectors are equal, regardless of order
                if a.len() != b.len() {
                    return false;
                }
                let a_set: HashSet<_> = a.iter().collect();
                let b_set: HashSet<_> = b.iter().collect();
                a_set == b_set
            },
            
            _ => false
        }
    }
}

impl Default for AtomTree {
    fn default() -> Self {
        Self::AtomType { atom: AtomType::False }
    }
}

impl AtomTree {
    //Todo: Find a notion of ordering atom trees inside an or statement, and then reduce or statements to only be binary
    fn outline_common_expressions(&mut self, outlined: &mut HashMap<AtomTree, usize>, var_count: &mut usize) {
        match self {
            Self::Variable { .. } => {
                return;
            }
            Self::Not(a) => {a.outline_common_expressions(outlined, var_count);},
            Self::Or(v) => {v.iter_mut().for_each(|f| f.outline_common_expressions(outlined, var_count))},
            _ => {}
        }
        if let Some(id) = outlined.get(self) {
            //println!("Found common expression: {:#?} => {}", self, id);
            *self = AtomTree::Variable { id: *id };
        } else {
            
            *var_count += 1;
            let mut new_var = AtomTree::Variable { id: *var_count };
            std::mem::swap(self, &mut new_var);
            
            //println!("Adding new expression: {:#?} => {}", new_var, var_count);
            outlined.insert(new_var, *var_count);
            //println!("Old: {:#?}", old);

        }
    }
    fn inline_all(&mut self, new_definitions: &mut HashMap<usize, Box<AtomTree>>, definitions: &HashMap<usize, VarDefinition>) {
        match self {
            Self::Variable { id } => {
                if let Some(v) = new_definitions.get(id) {
                    *self = *v.clone();
                } else {
                    let mut definition = definitions.get(id).expect("Expected definition to be some. (Probably a self referential variable, which shouldn't occur.)").clone();
                    
                    //We do not inline seed labels, since we don't wanna loose where they are coming from
                    if definition.definition.is_seed_label() {
                        return;
                    }
                    definition.definition.inline_all(new_definitions, definitions);
                    new_definitions.insert(*id, definition.definition);
                    *self = *new_definitions.get(id).unwrap().clone();
                }
            },
            Self::Not(a) => {a.inline_all(new_definitions, definitions);},
            Self::Or(v) => {v.iter_mut().for_each(|f| f.inline_all(new_definitions, definitions))},
            _ => {}
        }
    }

    fn count_var_use(&self, counter: &mut HashMap<usize, usize>) {
        match self {
            Self::Variable { id } => {*counter.entry(*id).or_insert(0) += 1;},
            Self::Not(a) => {a.count_var_use(counter);},
            Self::Or(v) => {v.iter().for_each(|f| f.count_var_use(counter))},
            _ => {}
        }
    }
    fn remove_marker(&mut self) {
        match self {
            Self::DoNotRemoveMarker(m) => {
                *self = *m.to_owned();
                self.remove_marker();
            }

            Self::Not(a) => {a.remove_marker();},
            Self::Or(v) => {v.iter_mut().for_each(|f| f.remove_marker());},
            _ => {}
        }
    }
    fn inline_var(&mut self, inline_definitions: &HashMap<AtomTree, Box<AtomTree>>) {
        match self {
            Self::Not(a) => {a.inline_var(inline_definitions);},
            Self::Or(v) => {v.iter_mut().for_each(|f| f.inline_var(inline_definitions))},
            _ => {}
        }
        if let Some(v) = inline_definitions.get(self) {
            *self = *v.clone();
            return;
        }
        
    }
    pub fn simp_rec(self, changed: &mut bool) -> Self {
        let mut simp_children = match self {
            Self::Not(a) => Self::Not(a.simp_rec(changed).into()),
            Self::Or(v) => Self::Or(v.into_iter().map(|a| a.simp_rec(changed)).collect()),
            _ => self
        };
        loop {
            let simplified;

            //println!("Simp before: {:#?}", simp_children);
            (simplified, simp_children) = simp_children.simp();
            //println!("Simp loop: {:#?}", simp_children);
            if !simplified {
                break;
            }
            *changed = true;
        } 
        return simp_children;
    }
    pub fn simp_multi_or(children: &mut Vec<AtomTree>) -> bool {
        todo!()
    }
    pub fn simp(self) -> (bool, Self) {
        //println!("Simp: {:#?}", self);
        (true, match self {
            
            Self::Not(a) if a.is_not() => {
                *a.into_not().unwrap()
            },
            Self::Not(a) if a.is_atom_type() => {
                Self::AtomType { atom: a.into_atom_type().unwrap().not() }
            },
            Self::Or(v) => {
                let mut new_v = HashSet::with_capacity(v.len() / 2);

                let mut changed = false;
                for f in v {
                    match f {
                        Self::AtomType { atom } => {
                            //If atom type is false, we don't need to add it to the set, since it will not change the result of the or operation 
                            if atom.is_true() {
                                return (true, Self::AtomType { atom: AtomType::True });
                            }
                        }
                        Self::Or(a) => {
                            //If we have an or inside of an or, we can just add the elements to the set
                            for a in a {
                                new_v.insert(a);
                            }
                            changed = true;
                        }
                        _ => {
                            new_v.insert(f);
                        }
                    }
                }

                

                let elems: Vec<AtomTree> = new_v.into_iter().collect();
                if elems.len() == 0 {
                    return (true, Self::AtomType { atom: AtomType::False });
                }
                if elems.len() == 1 {
                    return (true, elems.into_iter().next().unwrap());
                }
                return (changed, Self::Or(elems));
                
            }

            _ => return (false, self)
        })
    }
}