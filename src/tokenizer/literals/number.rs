use crate::tokenizer::LiteralKind;

use super::Cursor;

pub fn binary(cursor: &mut Cursor) -> (usize, LiteralKind) {
    debug_assert!(matches!(
        cursor.first().zip(cursor.second()),
        Some(('0', 'b') | ('0', 'B'))
    ));
    let _zero = cursor
        .bump()
        .expect("binary should start with 0b")
        .len_utf8();
    let _b = cursor
        .bump()
        .expect("binary should start with 0b")
        .len_utf8();
    let rest = cursor.eat_while(|c| matches!(c, '0' | '1'));
    assert!(
        rest > 0,
        "Binary literal should have at least 1 number following `0b`"
    );
    (_zero + _b + rest, LiteralKind::Binary)
}

pub fn hex(cursor: &mut Cursor) -> (usize, LiteralKind) {
    debug_assert!(matches!(
        cursor.first().zip(cursor.second()),
        Some(('0', 'x') | ('0', 'X'))
    ));
    let _zero = cursor.bump().expect("hex should start with 0x").len_utf8();
    let _x = cursor.bump().expect("hex should start with 0x").len_utf8();
    let rest = cursor.eat_while(|c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'));
    assert!(
        rest > 0,
        "Hex literal should have at least 1 number following `0x`"
    );
    (_zero + _x + rest, LiteralKind::Hex)
}

pub fn number(cursor: &mut Cursor) -> (usize, LiteralKind) {
    debug_assert!(matches!(cursor.first(), Some('0'..='9' | '.')));
    let first_digit = cursor
        .bump()
        .expect("First digit is checked with debug_assert");
    match (first_digit, cursor.first()) {
        ('.', _) => (1 + integer(cursor), LiteralKind::Floating),
        ('0', Some('.')) => match cursor.second() {
            Some('0'..='9') => (1 + floating(cursor), LiteralKind::Floating),
            _ => (1, LiteralKind::Integer),
        },
        ('0', None) => (1, LiteralKind::Integer),
        ('0', Some(following)) if !matches!(following, '0'..='9') => (1, LiteralKind::Integer),
        ('1'..='9', _) => {
            let integer_part = 1 + integer(cursor);
            match (cursor.first(), cursor.second()) {
                (Some('.'), Some('0'..='9')) => {
                    (integer_part + floating(cursor), LiteralKind::Floating)
                }
                _ => (integer_part, LiteralKind::Integer),
            }
        }
        (start, Some(following)) => unreachable!(
            "Unexpected number tokenizing. Text starts with {}{}...",
            start, following
        ),
        (start, None) => unreachable!("Unexpected number tokenizing. Text starts with {}", start),
    }
}

fn floating(cursor: &mut Cursor) -> usize {
    debug_assert!(matches!(cursor.first(), Some('.')));
    let _dot = cursor
        .bump()
        .expect("floating should starts with a dot")
        .len_utf8();
    let fraction_part = integer(cursor);
    _dot + fraction_part
}

fn integer(cursor: &mut Cursor) -> usize {
    cursor.eat_while(|c| matches!(c, '0'..='9'))
}

#[cfg(test)]
mod number_tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn test_integer(num: u32) -> bool {
        let num = num.to_string();
        let mut cursor = Cursor::from_iter(num.chars());
        let length = number(&mut cursor).0;
        num.len() == length
    }

    #[test]
    #[should_panic]
    fn test_zero_leading_integer() {
        let num = "029";
        let mut cursor = Cursor::from_iter(num.chars());
        let _ = number(&mut cursor).0;
    }

    #[quickcheck]
    fn test_floating_full(num: u32) -> bool {
        let num = format!("{num}.{num}", num = num);
        let mut cursor = Cursor::from_iter(num.chars());
        let length = number(&mut cursor).0;
        num.len() == length
    }

    #[test]
    fn test_zero_point_something() {
        let num = "0.382";
        let mut cursor = Cursor::from_iter(num.chars());
        let length = number(&mut cursor).0;
        assert_eq!(num.len(), length)
    }

    #[quickcheck]
    fn test_floating_only_fraction(num: u32) -> bool {
        let num = format!(".{}", num);
        let mut cursor = Cursor::from_iter(num.chars());
        let length = number(&mut cursor).0;
        num.len() == length
    }

    #[test]
    fn test_binary() {
        let targets = [
            "0b0", "0b01", "0b10", "0b001", "0b10101", "0B0", "0B01", "0B10", "0B001", "0B10101",
        ];
        for target in targets {
            let mut cursor = Cursor::from_iter(target.chars());
            assert_eq!(target.len(), binary(&mut cursor).0)
        }
    }

    #[test]
    #[should_panic]
    fn test_empty_binary() {
        let mut cursor = Cursor::from_iter("0b".chars());
        binary(&mut cursor);
    }

    #[test]
    fn test_not_binary() {
        let mut cursor = Cursor::from_iter("0b102".chars());
        assert_eq!(binary(&mut cursor).0, "0b10".len())
    }

    #[test]
    fn test_hex() {
        let targets = [
            "0x0",
            "0x01A",
            "0x123456789ABCDEF",
            "0xFFFF",
            "0xCACA1024",
            "0x0",
            "0x01a",
            "0x123456789abcdef",
            "0xffff",
            "0xcaca1024",
            "0X0",
            "0X01a",
            "0X123456789abcdef",
            "0Xffff",
            "0Xcaca1024",
            "0X0",
            "0X01A",
            "0X123456789ABCDEF",
            "0XFFFF",
            "0XCACA1024",
        ];
        for target in targets {
            let mut cursor = Cursor::from_iter(target.chars());
            assert_eq!(target.len(), hex(&mut cursor).0)
        }
    }

    #[test]
    #[should_panic]
    fn test_empty_hex() {
        let mut cursor = Cursor::from_iter("0x".chars());
        hex(&mut cursor);
    }

    #[test]
    fn test_not_hex() {
        let mut cursor = Cursor::from_iter("0xCANADA".chars());
        assert_eq!(hex(&mut cursor).0, "0xCA".len());
    }

    #[test]
    fn test_incomplete_float() {
        let mut cursor = Cursor::from_iter("3.".chars());
        assert_eq!(number(&mut cursor).0, "3".len())
    }
}
