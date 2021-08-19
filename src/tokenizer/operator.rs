use super::Cursor;

fn is_operator_start(c: char) -> bool {
    matches!(
        c,
        '=' | '>' | '<' | '!' | '~' | '+' | '-' | '*' | '/' | '&' | '|' | '^' | '%'
    )
}

pub fn operator(cursor: &mut Cursor) -> usize {
    debug_assert!(cursor.first().map(is_operator_start).unwrap_or_default());
    match cursor.bump().unwrap() {
        unary @ '~' => unary.len_utf8(),
        potential_arrow @ ('-' | '=') => eat_potential_arrow(potential_arrow, cursor),
        op_with_equals @ ('+' | '!' | '^' | '/' | '%') => {
            eat_operator_with_equals(op_with_equals, cursor)
        }
        double_op @ ('*' | '&' | '|') => eat_double_operator(double_op, cursor),
        shift_op @ ('<' | '>') => eat_shift_operator(shift_op, cursor),
        unexpected => unreachable!("Unexpected char for operator: {}", unexpected),
    }
}

fn eat_potential_arrow(leading: char, cursor: &mut Cursor) -> usize {
    leading.len_utf8()
        + match cursor.first() {
            Some('=' | '>') => cursor.bump().unwrap().len_utf8(),
            _ => 0,
        }
}

// =, ==, +, +=, -, -=, !, !=, ^, ^=, /, /=, %, %=
fn eat_operator_with_equals(leading: char, cursor: &mut Cursor) -> usize {
    leading.len_utf8()
        + match cursor.first() {
            Some('=') => cursor.bump().unwrap().len_utf8(),
            _ => 0,
        }
}

// *, **, *=, &, &&, &=, |, ||, |=
fn eat_double_operator(leading: char, cursor: &mut Cursor) -> usize {
    leading.len_utf8()
        + match cursor.first() {
            Some(op) if op == leading || op == '=' => cursor.bump().unwrap().len_utf8(),
            _ => 0,
        }
}

// >, >>, >>=, <, <<, <<=
fn eat_shift_operator(leading: char, cursor: &mut Cursor) -> usize {
    leading.len_utf8()
        + match cursor.first() {
            Some('=') => cursor.bump().unwrap().len_utf8(),
            Some(op) if op == leading => {
                let tail_shift = cursor.bump().unwrap().len_utf8();
                if matches!(cursor.first(), Some('=')) {
                    tail_shift + cursor.bump().unwrap().len_utf8()
                } else {
                    tail_shift
                }
            }
            _ => 0,
        }
}

#[cfg(test)]
mod operator_tests {
    use std::array::IntoIter;

    use quickcheck::{quickcheck, TestResult};

    use super::{is_operator_start, operator, Cursor};

    const ALL_OPERATORS: [&str; 33] = [
        "=", "==", ">", ">=", ">>", ">>=", "<", "<=", "<<", "<<=", "!", "!=", "~", "+", "+=", "-",
        "-=", "*", "*=", "/", "/=", "&", "&&", "&=", "|", "||", "|=", "^", "^=", "%", "%=", "->",
        "=>",
    ];

    #[test]
    fn test_is_operator_start_tokenizing_success() {
        let targets = [
            '=', '>', '<', '!', '~', '+', '-', '*', '/', '&', '|', '^', '%',
        ];
        assert!(IntoIter::new(targets).all(is_operator_start))
    }

    #[test]
    fn test_operators() {
        for target in ALL_OPERATORS {
            let mut cursor = Cursor::from_iter(target.chars());
            let size = operator(&mut cursor);
            assert_eq!(size, target.len())
        }
    }

    #[test]
    fn test_is_operator_start() {
        quickcheck(quickcheck_is_operator_start as fn(String) -> TestResult)
    }

    fn quickcheck_is_operator_start(s: String) -> TestResult {
        if s.is_empty() {
            TestResult::discard()
        } else if IntoIter::new(ALL_OPERATORS).any(|i| s.starts_with(i)) {
            TestResult::from_bool(is_operator_start(
                s.chars().next().expect("Checked by is_empty()"),
            ))
        } else {
            TestResult::from_bool(!is_operator_start(
                s.chars().next().expect("Checked by is_empty()"),
            ))
        }
    }

    #[test]
    fn test_operator_tokenizing() {
        quickcheck(quickcheck_operator_tokenizing as fn(String) -> TestResult);
    }

    fn quickcheck_operator_tokenizing(s: String) -> TestResult {
        let suffix;
        if s.is_empty() || is_operator_start(s.chars().next().expect("Checked by is_empty()")) {
            suffix = "";
        } else {
            suffix = &s;
        }
        for target in ALL_OPERATORS {
            let text = format!("{}{}", target, suffix);
            let mut cursor = Cursor::from_iter(text.chars());
            if operator(&mut cursor) != target.len() {
                return TestResult::from_bool(false);
            }
        }
        TestResult::from_bool(true)
    }
}
