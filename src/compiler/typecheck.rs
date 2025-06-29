use std::collections::HashMap;

use super::{compilation::Compilation, syntax::{CodeSyntax, Project, SubstructureSyntax, TypeSyntax}};

pub fn type_check(project: &Project, compilation: &mut Compilation) {
    for collection in &project.collections {
        collection.subs.iter().for_each(|s| check_sub(s, compilation));
    }
}

fn check_sub(sub: &SubstructureSyntax, compilation: &mut Compilation) {
    let mut vars: HashMap<&str, TypeSyntax> = HashMap::new();
    for input in &sub.args {
        vars.insert(&input.name.value, input.type_syntax.to_owned());
    }
    for s in &sub.code {
        match s {
            CodeSyntax::For { iterator_variable, iterator_amount, iterator_body } => todo!(

            ),
            _ => todo!()
        }
        todo!()
    }
}

//fn check_code_block(code: &Vec<CodeSyntax>, vars: HashMap<>)