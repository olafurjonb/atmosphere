pub mod compiler;
pub mod formula;
pub mod schema;
pub mod spatial;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("====================================================");
        eprintln!("  ASSC: Atmosphere Spatial Audio Compiler");
        eprintln!("====================================================");
        eprintln!("Usage:");
        eprintln!("  {} <input.jaff> <output.wav>", args[0]);
        eprintln!();
        eprintln!("Description:");
        eprintln!("  Compiles a declarative JSON scene file (.jaff) and");
        eprintln!("  its associated audio sources into a high-fidelity");
        eprintln!("  7.1.4 multi-channel spatial audio WAV file.");
        eprintln!("====================================================");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Starting compilation: {} -> {}", input_path, output_path);

    match compiler::compile_scene(input_path, output_path) {
        Ok(_) => {
            println!("Success! Spatial audio compiled successfully.");
        }
        Err(e) => {
            eprintln!("Error compiling scene: {}", e);
            process::exit(1);
        }
    }
}
