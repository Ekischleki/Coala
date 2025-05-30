pub mod compiler;

use std::env::args;
use compiler::settings::Settings;

fn parse_args() -> Settings {
    let mut args = args();
    let mut settings = Settings::default();
    //Consume path of executable
    args.next();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--nocolor" => {
                settings.color = true
            } 
            "--release" => {
                settings.color = true;
                settings.output_code_logs = false;
                settings.optimize = true;
            }
            "--heavy" => {
                settings.heavy_optimization = true;
            }
            "-p" => {
                settings.base_path = args.next();
            }
            "-o" => {
                settings.output_directory = args.next();
            }
            "--dev" => {
                settings.print_debug_logs = true;
            }
            _ => {}
        }
    }

    settings
}

fn main() {
    let settings = if args().count() == 1 {
        Settings {
            ignore_errors: false,
            optimize: false,
            output_code_logs: true,
            print_debug_logs: false,
            heavy_optimization: false,
            base_path: Some("./".into()),
            ..Default::default()
        }
    } else {
        parse_args()
    };
    
    compiler::compile(&settings);
}
 