use super::Cursor;

pub fn whitespace(cursor: &mut Cursor) -> usize {
    debug_assert!(cursor.first().map(is_whitespace).unwrap_or_default());
    cursor.eat_while(is_whitespace)
}

pub fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0020}'   // space
            | '\u{0009}' // horizontal tab
            | '\u{000C}' // form feed
    )
}

pub fn is_whitespace_or_newline(c: char) -> bool {
    is_whitespace(c) || is_line_terminator_start(c)
}

pub fn whitespace_and_newline(cursor: &mut Cursor) -> usize {
    cursor.eat_while(is_whitespace_or_newline)
}

pub fn newline(cursor: &mut Cursor) -> usize {
    debug_assert!(cursor
        .first()
        .map(is_line_terminator_start)
        .unwrap_or_default());
    let mut length = 0;
    loop {
        let new_line_length = eat_one_newline(cursor);
        if new_line_length == 0 {
            break;
        }
        length += new_line_length;
    }
    length
}

fn eat_one_newline(cursor: &mut Cursor) -> usize {
    match (cursor.first(), cursor.second()) {
        (Some('\n'), _) => cursor.bump().unwrap().len_utf8(),
        (Some('\r'), Some('\n')) => {
            cursor.bump().unwrap().len_utf8() + cursor.bump().unwrap().len_utf8()
        }
        _ => 0,
    }
}

#[cfg(test)]
pub const LINE_TERMINATORS: &[char] = &['\u{000A}', '\u{000D}'];

pub fn is_line_terminator_start(c: char) -> bool {
    matches!(
        c,
        '\u{000A}'   // newline
            | '\u{000D}' // return
    )
}

#[cfg(test)]
mod whitespace_tests {
    use crate::tokenizer::whitespace::eat_one_newline;
    use quickcheck_macros::quickcheck;

    use super::whitespace;
    use super::Cursor;

    #[quickcheck]
    fn test_tokenize_whitespace(mut text: String) -> bool {
        text.insert(0, ' ');
        let mut cursor = Cursor::from_iter(text.chars());
        let whitespace_char: &[_] = &[' ', '\u{000C}', '\t'];
        let expect_left_text = text.trim_start_matches(whitespace_char);
        let expect_length = text.len() - expect_left_text.len();
        expect_length == whitespace(&mut cursor)
    }

    #[test]
    fn test_eat_one_newline() {
        let mut cursor = Cursor::from_iter("\r\n".chars());
        let length = eat_one_newline(&mut cursor);
        let expected = "\r\n".chars().map(char::len_utf8).sum();
        assert_eq!(length, expected)
    }
}
