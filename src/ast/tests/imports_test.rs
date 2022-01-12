use super::super::Import;
use super::*;

#[test]
fn test_simple_import() {
    let parse_tree = parse(tokenize("use images.mountains.blue;"));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| {
            matches!(
                node.kind(),
                Some(NodeKind::SingleImportDeclarationStatement)
            )
        },
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find ImportDeclaration");
    let import = Import::from(node);
    assert_eq!(import, Import::Single(vec!["images", "mountains", "blue"]))
}

#[test]
fn test_multiple_import() {
    let parse_tree = parse(tokenize(
        "use images.canada.{ mountains.blue, lakes.ontario, parks }\n",
    ));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| {
            matches!(
                node.kind(),
                Some(NodeKind::MultipleImportDeclarationStatement)
            )
        },
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find MultipleImportDeclarationStatement");
    let import = Import::from(node);
    assert_eq!(
        import,
        Import::Multiple {
            prefix: vec!["images", "canada"],
            suffices: vec![
                vec!["mountains", "blue"],
                vec!["lakes", "ontario"],
                vec!["parks"]
            ]
        }
    )
}

#[test]
fn test_imports_wildcard() {
    let parse_tree = parse(tokenize("use images.canada.*\n"));
    let node = DepthFirst::find(
        parse_tree.root,
        |node| {
            matches!(
                node.kind(),
                Some(NodeKind::WildcardImportDeclarationStatement)
            )
        },
        |node| node.children().unwrap_or_default(),
    )
    .next()
    .expect("Cannot find WildcardImportDeclarationStatement");
    let import = Import::from(node);
    assert_eq!(import, Import::Wildcard(vec!["images", "canada"]))
}

#[test]
#[should_panic]
fn negative_multiple_with_wildcard() {
    let _ = parse(tokenize("use images.canada.{ mountains.blue, * }\n"));
}
