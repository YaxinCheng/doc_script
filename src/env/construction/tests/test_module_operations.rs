use super::super::ModuleAdder;
use super::construct_env;
use crate::env::scope::GLOBAL_SCOPE;

#[test]
fn test_simple_module() {
    let mut env = construct_env();
    let module_adder = ModuleAdder(&mut env);
    module_adder.add_modules(&[vec!["test"]]);
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"))
}

#[test]
fn test_nested_module() {
    let mut env = construct_env();
    let module_adder = ModuleAdder(&mut env);
    module_adder.add_modules(&[vec!["test", "doc"]]);
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
    let mut env = construct_env();
    let module_adder = ModuleAdder(&mut env);
    module_adder.add_modules(&[vec!["test"], vec!["test"]]);
    let global_scope = env.get_scope(GLOBAL_SCOPE);
    assert!(global_scope.name_spaces.modules.contains_key("test"))
}

#[test]
fn test_nested_diverge_module() {
    let mut env = construct_env();
    let module_adder = ModuleAdder(&mut env);
    module_adder.add_modules(&[vec!["test", "doc"], vec!["test", "image"]]);
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
