use super::Cursor;
use super::LiteralKind;
use crate::tokenizer::whitespace::{
    is_line_terminator_start, is_whitespace, whitespace_and_newline,
};

fn is_string_header(cursor: &Cursor) -> bool {
    match cursor.first() {
        Some('"') => true,
        Some('r') => matches!(cursor.second(), Some('#')),
        _ => false,
    }
}

pub fn string(cursor: &mut Cursor) -> (usize, LiteralKind) {
    debug_assert!(is_string_header(cursor));
    let length = match cursor.first() {
        Some('"') => simple_string(cursor),
        Some('r') => raw_string(cursor),
        leading => unreachable!("string cannot start with: {:?}", leading),
    };
    (length, LiteralKind::String)
}

fn simple_string(cursor: &mut Cursor) -> usize {
    let mut total_len = cursor.bump().expect("start quote").len_utf8(); // start quote
    loop {
        let eaten_length = cursor.eat_while(|c| c != '"' && c != '\\');
        total_len += eaten_length;
        total_len += match cursor.first() {
            Some('\\') => eat_escaped_char(cursor),
            Some('"') => break,
            _ => panic!("string not closed"),
        }
    }
    total_len + cursor.bump().unwrap().len_utf8() // close quote
}

fn raw_string(cursor: &mut Cursor) -> usize {
    let mut total_len = cursor.bump().unwrap().len_utf8(); // r
    let leading_pounds = eat_pound(cursor, None);
    assert!(
        leading_pounds > 0,
        r##"raw string should be in the format of `r#.."..."#..`"##
    );
    let start_quote = cursor.bump();
    assert_eq!(start_quote, Some('"'), r#"Missing `"` after # sign"#);
    total_len += start_quote.unwrap().len_utf8();
    loop {
        total_len += cursor.eat_while(|c| c != '"');
        match cursor.first() {
            Some('"') => {
                let close_quote = cursor.bump().unwrap().len_utf8();
                let trailing_pounds = eat_pound(cursor, Some(leading_pounds));
                total_len += close_quote + trailing_pounds;
                if trailing_pounds == leading_pounds {
                    break;
                }
            }
            _ => panic!("string not closed"),
        }
    }
    total_len + leading_pounds
}

fn eat_escaped_char(cursor: &mut Cursor) -> usize {
    debug_assert_eq!(cursor.first(), Some('\\'));
    let backslash = cursor.bump().unwrap().len_utf8();
    let leading_char = cursor.first().expect("No character after backslash");
    let escaped_length = match leading_char {
        't' | 'n' | 'r' | '"' | '\'' | '\\' => cursor.bump().unwrap().len_utf8(),
        whitespace if is_whitespace(whitespace) || is_line_terminator_start(whitespace) => {
            whitespace_and_newline(cursor)
        }
        _ => panic!("Escape invalid character"),
    };
    backslash + escaped_length
}

fn eat_pound(cursor: &mut Cursor, limit: Option<usize>) -> usize {
    if !matches!(cursor.first(), Some('#')) {
        return 0;
    }
    if let Some(limit) = limit {
        let mut length = 0;
        while matches!(cursor.first(), Some('#')) && length < limit {
            length += cursor.bump().unwrap().len_utf8();
        }
        length
    } else {
        cursor.eat_while(|c| c == '#')
    }
}

#[cfg(test)]
mod string_tests {
    use quickcheck::{quickcheck, TestResult};

    use super::{string, Cursor};

    fn test_string(target: &str) {
        let mut cursor = Cursor::from_iter(target.chars());
        let length = string(&mut cursor).0;
        assert_eq!(target.len(), length)
    }

    #[test]
    fn test_simple_string() {
        let targets = [
            r#""text""#,
            r#""emoji🌎""#,
            r#""escaped backslash\\""#,
            r#""escaped\"quote""#,
            r#""escaped\tnumber""#,
            r#""test\n\tmultiple\nline""#,
        ];
        for target in targets {
            test_string(target)
        }
    }

    #[test]
    fn test_raw_strings() {
        let targets = [
            r##"r#"test with "quote""#"##,
            r###"r##"test with #"pound quote"#"##"###,
            r##"r#"test
            multiple
            lines"#"##,
            r##"r#"test\useless\backslash\""#"##,
        ];
        for target in targets {
            test_string(target)
        }
    }

    #[test]
    fn test_raw_string_extra_pounds() {
        let target = r###"r#"test"##"###;
        let expected = target.len() - '#'.len_utf8();
        let actual = string(&mut Cursor::from_iter(target.chars())).0;
        assert_eq!(expected, actual)
    }

    #[test]
    #[should_panic]
    fn test_string_unclosed() {
        test_string(r#""unclosed"#)
    }

    #[test]
    #[should_panic]
    fn test_string_escaped_unclose() {
        test_string(r#""escaped\""#)
    }

    #[test]
    #[should_panic]
    fn test_string_invalid_escaped() {
        test_string(r#""escaped\a"#)
    }

    #[test]
    #[should_panic]
    fn test_string_linebreak() {
        test_string(r#""escaped\n"#);
    }

    #[test]
    #[should_panic]
    fn test_raw_string_missing_start_quote() {
        test_string(r##"r#random"#"##);
    }

    #[test]
    #[should_panic]
    fn test_raw_string_unclosed() {
        test_string(r##"r#"random#"##);
    }

    #[test]
    #[should_panic]
    fn test_raw_string_unmatched_pounds() {
        test_string(r####"r##"test"#"####);
    }

    #[test]
    fn test_strings_without_quotes_newline_backslash() {
        quickcheck(quickcheck_test_strings as fn(String) -> TestResult);
    }

    fn quickcheck_test_strings(s: String) -> TestResult {
        if s.chars().any(|c| matches!(c, '"' | '\\')) {
            TestResult::discard()
        } else {
            let target = format!(r#""{}""#, s);
            let mut cursor = Cursor::from_iter(target.chars());
            let length = string(&mut cursor).0;
            TestResult::from_bool(target.len() == length)
        }
    }

    #[test]
    fn test_string_followed_by_other_token() {
        quickcheck(test_string_followed_by_other_token_by_length as fn(String) -> TestResult);
    }

    fn test_string_followed_by_other_token_by_length(s: String) -> TestResult {
        if s.chars().any(|c| matches!(c, '"' | '\\')) {
            TestResult::discard()
        } else {
            let target = format!(r#""{}".toUppercase()"#, s);
            let max_length = target.len() - ".toUppercase()".len();
            let mut cursor = Cursor::from_iter(target.chars());
            let length = string(&mut cursor).0;
            TestResult::from_bool(max_length == length)
        }
    }

    #[test]
    fn test_complicated_raw_string() {
        quickcheck(test_raw_string_complicated as fn(String) -> TestResult)
    }

    fn test_raw_string_complicated(s: String) -> TestResult {
        let target = format!(r#####"r###"{}"###"#####, s);
        let expected = target.len();
        let mut cursor = Cursor::from_iter(target.chars());
        let length = string(&mut cursor).0;
        TestResult::from_bool(expected == length)
    }

    #[test]
    fn test_string_escape_newline() {
        let target = r#""a string with\ 
        escaped newline""#;
        let mut cursor = Cursor::from_iter(target.chars());
        let length = string(&mut cursor).0;
        assert_eq!(length, target.len())
    }
}
