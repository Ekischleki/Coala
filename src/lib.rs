pub mod compiler;

#[cfg(test)]
mod tests {
    use crate::compiler::{self, settings::Settings};

    #[test]
    pub fn cmp() {
        let settings = Settings::default();
        compiler::compile(&settings);
    }
}