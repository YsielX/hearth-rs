use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn sources(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, files);
        } else if path.extension().is_some_and(|value| value == "rs") {
            files.push(path);
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut files = vec![
        root.join("Cargo.lock"),
        root.join("Cargo.toml"),
        root.join("crates/hearth-env-py/build.rs"),
    ];
    for name in [
        "hearth-core",
        "hearth-bot",
        "hearth-script",
        "hearth-env",
        "hearth-env-py",
    ] {
        let directory = root.join("crates").join(name);
        files.push(directory.join("Cargo.toml"));
        sources(&directory.join("src"), &mut files);
        println!("cargo:rerun-if-changed={}", directory.join("src").display());
    }
    files.sort();
    let mut hash = 14695981039346656037u64;
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path.strip_prefix(&root).unwrap().to_string_lossy();
        for byte in relative
            .as_bytes()
            .iter()
            .copied()
            .chain([0])
            .chain(fs::read(&path).unwrap())
            .chain([0])
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(1099511628211);
        }
    }
    println!("cargo:rustc-env=HEARTH_ENGINE_BUILD={hash:016x}");
}
