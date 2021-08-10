use super::Cursor;

pub fn whitespace<'a>(cursor: &mut Cursor<'a>) -> usize {
    debug_assert!(cursor.first().map(is_whitespace).unwrap_or_default());
    cursor.eat_while(is_whitespace)
}

pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0020}'   // space
            | '\u{0009}' // horizontal tab
            | '\u{000C}' // form feed
    ) || is_line_terminator(c)
}

pub const LINE_TERMINATORS: &[char] = &['\u{000A}', '\u{000D}'];

pub fn is_line_terminator(c: char) -> bool {
    matches!(
        c,
        '\u{000A}'   // newline
            | '\u{000D}' // return
    )
}

#[cfg(test)]
mod whitespace_tests {
    use super::whitespace;
    use super::Cursor;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_tokenize_whitespace(mut text: String) -> bool {
        text.insert(0, ' ');
        let mut cursor = Cursor::from_iter(text.chars());
        let whitespace_char: &[_] = &[' ', '\u{000C}', '\t', '\n', '\r'];
        let expect_left_text = text.trim_start_matches(whitespace_char);
        let expect_length = text.len() - expect_left_text.len();
        expect_length == whitespace(&mut cursor)
    }
}
