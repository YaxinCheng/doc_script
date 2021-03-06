use super::whitespace;
use super::Cursor;

pub fn comment(cursor: &mut Cursor) -> usize {
    debug_assert!(has_comment_header(cursor));
    cursor.eat_while(|c| !whitespace::is_line_terminator_start(c))
}

fn has_comment_header(cursor: &Cursor) -> bool {
    matches!(cursor.first().zip(cursor.second()), Some(('/', '/')))
}

#[cfg(test)]
mod comment_test {
    use quickcheck_macros::quickcheck;

    use super::comment;
    use super::whitespace;
    use super::Cursor;

    #[quickcheck]
    fn test_tokenize_comment(mut content: String) -> bool {
        content.insert_str(0, "//");
        let expected_length = content
            .find(whitespace::LINE_TERMINATORS)
            .unwrap_or(content.len());
        let actual_length = comment(&mut Cursor::from_iter(content.chars()));
        expected_length == actual_length
    }
}
