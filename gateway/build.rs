use std::process::Command;

fn main() {
    built::write_built_file().expect("failed to acquire build-time information");

    // Re-run build.rs only when the spec or generator config changes
    println!("cargo:rerun-if-changed=openapi.yaml");
    println!("cargo:rerun-if-changed=openapi-to-rust.toml");

    let status = Command::new("openapi-to-rust")
        .arg("generate")
        .status()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to run 'openapi-to-rust'. Ensure it is installed via `cargo install openapi-to-rust`."
            )
        });

    if !status.success() {
        panic!("'openapi-to-rust generate' failed with status: {}", status);
    }
}
