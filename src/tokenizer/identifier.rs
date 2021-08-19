use super::Cursor;

pub fn identifier<'a>(cursor: &mut Cursor<'a>) -> usize {
    debug_assert!(cursor.first().map(is_identifier_start).unwrap_or_default());
    cursor.eat_while(is_identifier_continue)
}

pub fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic()
}

pub fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod identifier_tests {
    use super::Cursor;
    use super::identifier;

    #[test]
    fn test_ascii_letters() {
        let content = "random ";
        let expected = content.len() - 1;
        let actual = test_identifier(content);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_snake_case() {
        let content = "random_snake";
        let expected = content.len();
        let actual = test_identifier(content);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_unicode() {
        let content = "ひらがな_カタカナ_漢字";
        let expected = content.len();
        let actual = test_identifier(content);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_emoji_should_fail() {
        let content = "t😂_❌";
        let expected = 1;
        let actual = test_identifier(content);
        assert_eq!(expected, actual)
    }

    fn test_identifier(content: &str) -> usize {
        let mut cursor = Cursor::from_iter(content.chars());
        identifier(&mut cursor)
    }
}
