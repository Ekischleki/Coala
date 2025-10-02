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
                match args.next() {
                    None => {
                        panic!("Expected argument after -o")
                    }
                    Some(d) => settings.output_directory = d 
                }
            }
            "--dev" => {
                settings.print_debug_logs = true;
            }
            "--expr" => {
                settings.as_expression = true;
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
            optimize: true,
            output_code_logs: true,
            print_debug_logs: false,
            heavy_optimization: true,
            as_expression: true,
            base_path: Some("./".into()),
            ..Default::default()
        }
    } else {
        parse_args()
    };
    
    compiler::compile(&settings);
}
 