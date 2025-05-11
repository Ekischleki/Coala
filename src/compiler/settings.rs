#[derive(Debug, Clone)]
pub struct Settings {
    pub color: bool,
    pub optimize: bool,
    pub heavy_optimization: bool,
    pub output_code_logs: bool,
    pub print_debug_logs: bool,
    pub output_diagnostics: bool,
    pub base_path: Option<String>,
    pub output_directory: Option<String>,
    pub ignore_errors: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { color: true, optimize: false, output_code_logs: true, print_debug_logs: false, output_diagnostics: true, base_path: None, output_directory: None, ignore_errors: false, heavy_optimization: false }
    }
}
