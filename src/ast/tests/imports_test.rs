use super::super::Import;
use super::*;
use crate::ast::Name;

#[test]
fn test_simple_import() {
    let parse_tree = parse(tokenize("use images.mountains.blue"));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::SingleImportDeclaration)),
        |node| node.children_owned().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find ImportDeclaration");
    let import = Import::from(node);
    assert_eq!(
        import,
        Import::Single(Name::Qualified(vec!["images", "mountains", "blue"]))
    )
}

#[test]
fn test_multiple_import() {
    let parse_tree = parse(tokenize(
        "use images.canada.{ mountains.blue, lakes.ontario, parks }",
    ));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::MultipleImportDeclaration)),
        |node| node.children_owned().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find MultipleImportDeclaration");
    let import = Import::from(node);
    assert_eq!(
        import,
        Import::Multiple {
            prefix: Name::Qualified(vec!["images", "canada"]),
            suffices: vec![
                Name::Qualified(vec!["mountains", "blue"]),
                Name::Qualified(vec!["lakes", "ontario"]),
                Name::Simple("parks")
            ]
        }
    )
}

#[test]
fn test_imports_wildcard() {
    let parse_tree = parse(tokenize("use images.canada.*"));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| matches!(node.kind(), Some(NodeKind::WildcardImportDeclaration)),
        |node| node.children_owned().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find WildcardImportDeclaration");
    let import = Import::from(node);
    assert_eq!(
        import,
        Import::Wildcard(Name::Qualified(vec!["images", "canada"]))
    )
}

#[test]
#[should_panic]
fn negative_multiple_with_wildcard() {
    let _ = parse(tokenize("use images.canada.{ mountains.blue, * }"));
}
