use std::process::Command;
use std::fs;

fn main() {
    // Path to contracts directory
    let contracts_dir = "src/contracts";

    // Read directory entries
    let entries = fs::read_dir(contracts_dir).expect("contracts directory not found");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        // Filter .wat files
        if path.extension().and_then(|s| s.to_str()) == Some("wat") {
            let wat_path = path.to_str().unwrap();
            let wasm_path = path.with_extension("wasm");
            let wasm_path_str = wasm_path.to_str().unwrap();

            println!("cargo:rerun-if-changed={}", wat_path);

            // Run wat2wasm command
            let status = Command::new("wat2wasm")
                .arg(wat_path)
                .arg("-o")
                .arg(wasm_path_str)
                .status()
                .expect("Failed to run wat2wasm");

            if !status.success() {
                panic!("wat2wasm failed for {}", wat_path);
            } else {
                println!("Compiled {} to {}", wat_path, wasm_path_str);
            }
        }
    }
}