use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let index_path = manifest_dir.join("data/test_input/builtin_scripts.tsv");
    println!("cargo:rerun-if-changed={}", index_path.display());
    let index = fs::read_to_string(&index_path).expect("read builtin input script index");
    let mut generated = String::from(
        "pub(super) fn builtin_script_text(scenario: &str) -> Option<&'static str> {\n    match scenario {\n",
    );
    for line in index.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (id, file_name) = line.split_once('\t').expect("builtin script index row");
        assert!(
            !file_name.contains('/') && !file_name.contains('\\') && !file_name.contains(".."),
            "invalid builtin script path {file_name:?}"
        );
        let script_path = manifest_dir.join("data/test_input").join(file_name);
        assert!(
            script_path.is_file(),
            "missing builtin script {file_name:?}"
        );
        println!("cargo:rerun-if-changed={}", script_path.display());
        generated.push_str(&format!(
            "        {id:?} => Some(include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/data/test_input/{file_name}\"))),\n"
        ));
    }
    generated.push_str("        _ => None,\n    }\n}\n");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output"))
        .join("lay_test_input_builtin_scripts.rs");
    fs::write(output, generated).expect("write embedded input script map");
}
