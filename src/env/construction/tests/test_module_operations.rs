use crate::env::scope::GLOBAL_SCOPE;
use crate::env::Environment;

#[test]
fn test_simple_module() {
    let env = Environment::builder().add_modules(&[vec!["test"]]).build();
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"))
}

#[test]
fn test_nested_module() {
    let env = Environment::builder()
        .add_modules(&[vec!["test", "doc"]])
        .build();
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"));
    let scope = env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .modules
        .get("test")
        .unwrap();
    let scope = env.get_scope(*scope);
    assert!(scope.name_spaces.modules.contains_key("doc"));
}

#[test]
fn test_duplicated_module() {
    let env = Environment::builder()
        .add_modules(&[vec!["test"], vec!["test"]])
        .build();
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"))
}

#[test]
fn test_nested_diverge_module() {
    let env = Environment::builder()
        .add_modules(&[vec!["test", "doc"], vec!["test", "image"]])
        .build();
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"));
    let scope = env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .modules
        .get("test")
        .unwrap();
    let scope = env.get_scope(*scope);
    assert!(scope.name_spaces.modules.contains_key("doc"));
    assert!(scope.name_spaces.modules.contains_key("image"));
}
