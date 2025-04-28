use std::{collections::{HashMap, HashSet}, default};

use enum_as_inner::EnumAsInner;

use crate::{atom_tree_to_graph::Label, compilation::Compilation, token::AtomType};
#[derive(Debug, Clone, Default)]

pub struct AtomRoot {
    pub definitions: HashMap<usize, VarDefinition>,
    variable_id: usize,
    pub value_actions: Vec<(AtomTree, ValueAction)>
}

#[derive(Debug, Clone)]
pub enum ValueAction {
    Output(String),
    Restriction(AtomType)
}

impl AtomRoot {
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
                        inline_definitions.insert(*id, Box::new(AtomTree::AtomType { atom: *t }));
                        *value = AtomTree::DoNotRemoveMarker(Box::new(AtomTree::Variable { id: *id }));
                    }
                    //force not a => atom type <=> force a => not atom type
                    AtomTree::Not(a) => {
                        *changed = true;
                        
                        *t = t.not();
                        *value = *(a).clone();
                    }

                    AtomTree::Or(a, b) if t.is_false() => {
                        *changed = true;
                        remove_indecies.push(i);
                        add_items.push((*a.to_owned(), ValueAction::Restriction(AtomType::False)));
                        add_items.push((*b.to_owned(), ValueAction::Restriction(AtomType::False)));
                    }
                    //something like force true => true
                    AtomTree::AtomType { atom } => {
                        if atom == t {
                            compilation.add_info("Force statement is trivially always successful.", None);
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
    }

    pub fn simp_all(&mut self, compilation: &mut Compilation) -> bool {
        let mut changed = false;
        self.apply_to_all_trees(|t| {
            t.simp_rec(&mut changed)
        });
        println!("{:#?}", self);
        self.simp_force(compilation, &mut changed);

        return changed;
    }

    pub fn finalize_simp(&mut self) {
        self.apply_to_all_trees_mut(|t| {
            t.finalize_simp()
        });
    }

    pub fn remove_links(&mut self) -> bool {
        let mut inline_definitions = HashMap::new();
        let mut exclude_for_now = HashSet::new(); //To stop errors, we can only replace one definition at a time.
        for (id, definition) in &self.definitions {
            if exclude_for_now.contains(id) {continue;}
            match &*definition.definition {
                AtomTree::Variable { id: v } => {
                    if inline_definitions.contains_key(v) {continue;}

                    inline_definitions.insert(*id, definition.definition.to_owned());
                    exclude_for_now.insert(*v);
                }
                AtomTree::AtomType { atom } => {
                    inline_definitions.insert(*id, definition.definition.to_owned());
                    
                }
                _ => {}
            }
        }
        println!("{:#?}", inline_definitions);
        if inline_definitions.len() == 0 {
            return false;
        }

        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.inline_var(&inline_definitions);
        });
        for (removed_id, _) in inline_definitions {
            self.definitions.remove(&removed_id);
        }

        true
    }
    pub fn inline_vars(&mut self) -> bool {
        return false; //todo: fix this garbage
        let mut var_uses = HashMap::new();
        self.apply_to_all_trees(|tree| {tree.count_var_use(&mut var_uses); tree});

       
        let mut inline_definitions: HashMap<usize, Box<AtomTree>> = HashMap::new();
        let mut changed = false;
        for (variable, uses) in var_uses.iter() {
            match uses {
                0 => {
                    self.definitions.remove(variable);
                }
                1 => {
                    let definition = self.definitions.remove(variable).expect("Expected definition to be some.").definition;
                    inline_definitions.insert(*variable, definition);
                    changed = true;
                }
                _ => {}
            }
        }
        self.apply_to_all_trees_mut(|atom_tree| {
            atom_tree.inline_var(&inline_definitions);
        });
        changed
    }
    fn apply_to_all_trees_mut<F>(&mut self, predicate: F)
    where
        F: Fn(&mut AtomTree)
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



#[derive(EnumAsInner, Debug, Clone, PartialEq, Eq, Hash)]
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
        Box<AtomTree>,
        Box<AtomTree>
    )
    
}


impl AtomTree {
    fn count_var_use(&self, counter: &mut HashMap<usize, usize>) {
        match self {
            Self::Variable { id } => {*counter.entry(*id).or_insert(0) += 1;},
            Self::Not(a) => {a.count_var_use(counter);},
            Self::Or(a, b) => {a.count_var_use(counter); b.count_var_use(counter);},
            _ => {}
        }
    }
    fn finalize_simp(&mut self) {
        match self {
            Self::DoNotRemoveMarker(m) => {
                *self = *m.to_owned();
                self.finalize_simp();
            }

            Self::Not(a) => {a.finalize_simp();},
            Self::Or(a, b) => {a.finalize_simp(); b.finalize_simp();},
            _ => {}
        }
    }
    fn inline_var(&mut self, inline_definitions: &HashMap<usize, Box<AtomTree>>) {
        match self {
            Self::Variable { id:  self_id } => {        
                if let Some(v) = inline_definitions.get(self_id) {
                    *self = *v.clone();
                }
            },
            Self::Not(a) => {a.inline_var(inline_definitions);},
            Self::Or(a, b) => {a.inline_var(inline_definitions); b.inline_var(inline_definitions);},
            _ => {}
        }
        
    }
    pub fn simp_rec(self, changed: &mut bool) -> Self {
        let mut simp_children = match self {
            Self::Not(a) => Self::Not(a.simp_rec(changed).into()),
            Self::Or(a, b) => Self::Or(a.simp_rec(changed).into(), b.simp_rec(changed).into()),
            _ => self
        };
        loop {
            let simplified;
            (simplified, simp_children) = simp_children.simp();
            if !simplified {
                break;
            }
            *changed = true;
        } 
        return simp_children;
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
            Self::Or(a, b) if a.is_atom_type() => {
                match a.into_atom_type().unwrap() {
                    AtomType::True => Self::AtomType { atom: AtomType::True },
                    AtomType::False => *b,
                }
            }
            Self::Or(a, b) if b.is_atom_type() => {
                match b.into_atom_type().unwrap() {
                    AtomType::True => Self::AtomType { atom: AtomType::True },
                    AtomType::False => *a,
                }
            }
            Self::Or(a, b) if a == b => {
                *a
            },
            Self::Or(a, b) if *a == Self::Not(b.clone()) => {
                Self::AtomType { atom: AtomType::True }
            },
            Self::Or(a, b) if Self::Not(a.clone()) == *b => {
                Self::AtomType { atom: AtomType::True }
            },
            _ => return (false, self)
        })
    }
}