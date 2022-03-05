use super::COMPILER_LOCK;
use crate::compile_to;
use std::io::Result;
use tempdir::TempDir;

#[test]
fn test_doc_build_empty_single_file() -> Result<()> {
    let compiled = compile_single_file(r#"const Main = Doc { }"#)?;
    let compiled_str = std::str::from_utf8(&compiled).expect("Not utf8");
    assert_eq!(compiled_str, "");
    Ok(())
}

#[test]
fn test_doc_build_simple_single_file() -> Result<()> {
    let compiled = compile_single_file(r#"const Main = Doc { Text("Hello World") }"#)?;
    let compiled_str = std::str::from_utf8(&compiled).expect("Not utf8");
    assert_eq!(
        compiled_str,
        r#"Doc: {children: [Text: {content: "Hello World",},],}"#
    );
    Ok(())
}

#[test]
fn test_doc_wrapped_element() -> Result<()> {
    let compiled = compile_single_file(
        r#"
    struct WrappedText(text: String) {
        const rendered = Text(self.text)
    }
    
    const Main = Doc { WrappedText("Hello World") }
    "#,
    )?;
    let compiled_str = std::str::from_utf8(&compiled).expect("Not utf8");
    assert_eq!(
        compiled_str,
        r#"Doc: {children: [Text: {content: "Hello World",},],}"#
    );
    Ok(())
}

#[test]
fn test_wrapped_container() -> Result<()> {
    let compiled = compile_single_file(
        r#"
    struct WrappedDoc(children: Children) {
        const rendered = Doc(self.children)
    }
    
    const Main = WrappedDoc { Text("Hello World") }
    "#,
    )?;
    let compiled_str = std::str::from_utf8(&compiled).expect("Not utf8");
    assert_eq!(
        compiled_str,
        r#"Doc: {children: [Text: {content: "Hello World",},],}"#
    );
    Ok(())
}

fn compile_single_file(content: &str) -> Result<Vec<u8>> {
    let id = std::thread::current().id();
    let project_dir = TempDir::new(&format!("example{id:?}"))?;
    let source_file_path = project_dir.path().join("main.ds");
    std::fs::write(&source_file_path, content)?;
    let _locked = COMPILER_LOCK.lock().expect("Failed to lock");
    std::env::set_current_dir(&project_dir)?;
    let compiled = compile_to(
        vec!["main.ds".to_string()],
        crate::code_generation::generate_code_to_buffer,
    );
    Ok(compiled)
}
