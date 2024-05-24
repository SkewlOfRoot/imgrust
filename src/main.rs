use std::env;
use std::process;

use imgrust::{compress_image_files, CommandArgs};

fn main() {
    let args: Vec<String> = env::args().collect();

    let command_args = CommandArgs::new(&args).unwrap_or_else(|err| {
        println!("problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = compress_image_files(
        command_args.input_path.as_str(),
        command_args.output_path.as_str(),
    ) {
        println!("Application error {}", e);
        process::exit(1);
    }
}
