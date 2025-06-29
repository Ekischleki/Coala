use std::collections::HashMap;

pub struct Scope<'a, T> {
    vars: Vec<HashMap<&'a str, T>>
}

impl<T> Scope<'_, T> {
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut T> {
        for v in self.vars.iter().rev() {
            //if let Some()
        }
        todo!()
    }
}