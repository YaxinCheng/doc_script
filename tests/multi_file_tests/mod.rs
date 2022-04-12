#![cfg(test)]

use super::COMPILER_LOCK;
use doc_script::compile;
use std::io::Result;
use tempdir::TempDir;

#[test]
fn test_separate_file_hello_world() -> Result<()> {
    let compiled = compile_multi_files([
        r#"const INFO = "Hello World""#,
        r#"const Main = Doc { Text(INFO) }"#,
    ])?;
    let compiled_str = std::str::from_utf8(&compiled).expect("Not utf8");
    assert_eq!(
        compiled_str,
        r#"Doc: {children: [Text: {content: "Hello World",},],}"#
    );
    Ok(())
}

fn compile_multi_files<const N: usize>(file_content: [&str; N]) -> Result<Vec<u8>> {
    let id = std::thread::current().id();
    let project_dir = TempDir::new(&format!("example{id:?}"))?;
    let mut file_names = Vec::with_capacity(N);
    for (index, content) in file_content.iter().enumerate() {
        let file_name = format!("file{index}.ds");
        let source_file_path = project_dir.path().join(&file_name);
        std::fs::write(&source_file_path, content)?;
        file_names.push(file_name);
    }
    let _locked = COMPILER_LOCK.lock().expect("Failed to lock");
    std::env::set_current_dir(&project_dir)?;
    let compiled = compile(&file_names);
    Ok(compiled)
}
