fn main() {
    let licence = std::path::Path::new("src/licence.rs");
    if !licence.exists() {
        std::fs::copy("src/licence_community.rs", licence)
            .expect("Failed to create src/licence.rs from community stub");
    }
    println!("cargo:rerun-if-changed=src/licence_community.rs");
    println!("cargo:rerun-if-changed=src/licence.rs");
}
