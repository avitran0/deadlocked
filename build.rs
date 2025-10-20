use copy_to_output::copy_to_output;

fn main() {
    println!("cargo:rerun-if-changed=resources/source2viewer/Source2Viewer-CLI");
    let _ = copy_to_output(
        "resources/source2viewer",
        &std::env::var("PROFILE").unwrap(),
    );
}
