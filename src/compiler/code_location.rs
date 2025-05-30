use std::{hash::{Hash, Hasher}, path::PathBuf};


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodeLocation {
    pub path: PathBuf,
    pub section: Option<Section>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section { pub location_begin: u64, pub location_end: u64 }

impl CodeLocation {

    pub fn to(&self, end: &Self) -> Self {
        Self::section(self, end)
    }


    pub fn section(beginning: &Self, end: &Self) -> Self {
        assert_eq!(beginning.path, end.path);
        if beginning.section.is_some() != end.section.is_some() {
            panic!("Both code locations must have either some sections, or none")
        }
        Self {
            path: beginning.path.to_owned(),
            section: if beginning.section.is_some() { 
                Some(Section {
                    location_begin: beginning.section.as_ref().unwrap().location_begin, 
                    location_end: end.section.as_ref().unwrap().location_end 
                })
            } else {None}
        }
    }

    pub fn new(path: PathBuf) -> Self {
        Self{
            path,
            section: None
        }
    }

    pub fn with_section(path: PathBuf, location_begin: u64, location_end: u64) -> Self {
        Self{
            path,
            section: Some(Section {
                location_begin,
                location_end
            })
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocationValue<T> {
    pub location: Option<CodeLocation>,
    pub value: T
} 

impl<T: Eq> Eq for LocationValue<T> {}

impl <T: Hash> Hash for LocationValue<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialEq> PartialEq for LocationValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}


impl<T> LocationValue<T> {
    pub fn new(location: Option<CodeLocation>, value: T) -> Self {
        Self { location, value }
    }
}

impl<T> From<T> for LocationValue<T> {
    fn from(value: T) -> Self {
        Self { location: None, value }
    }
}