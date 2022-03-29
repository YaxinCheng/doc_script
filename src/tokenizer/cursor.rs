use std::str::Chars;

pub struct Cursor<I: Iterator> {
    prev: Option<I::Item>,
    current: I,
}

impl<I: Iterator> Cursor<I> {
    pub fn from_iter(iter: I) -> Self {
        Cursor {
            prev: None,
            current: iter,
        }
    }
}

impl<I> Cursor<I>
where
    I: Iterator + Clone,
    I::Item: Copy,
{
    pub fn first(&self) -> Option<I::Item> {
        self.nth(0)
    }

    pub fn second(&self) -> Option<I::Item> {
        self.nth(1)
    }

    pub fn nth(&self, n: usize) -> Option<I::Item> {
        self.current.clone().nth(n)
    }

    pub fn bump(&mut self) -> Option<I::Item> {
        let first = self.current.next()?;
        self.prev = Some(first);
        Some(first)
    }
}

impl<'a> Cursor<Chars<'a>> {
    /// `eat_while` repeats consuming characters as long as `predicate` returns true.
    /// and eventually it outputs the **number of bytes** of the consumed characters.
    pub fn eat_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> usize {
        let mut eaten_length = 0;

        while let Some(c) = self.first() {
            if predicate(c) {
                eaten_length += self.bump().unwrap().len_utf8();
            } else {
                break;
            }
        }
        eaten_length
    }
}

#[cfg(test)]
mod cursor_tests {
    use quickcheck_macros::quickcheck;

    use super::Cursor;

    #[quickcheck]
    fn test_eat_while_finish(text: String) -> bool {
        let mut cursor = Cursor::from_iter(text.chars());
        let length = cursor.eat_while(|c| c != '"');
        let expected = text.find('"').unwrap_or(text.len());
        length == expected
    }

    #[test]
    fn test_eat_while_with_unicode_letters() {
        let test = "你好,🌎";
        let mut cursor = Cursor::from_iter(test.chars());
        let length = cursor.eat_while(|c| c != ',');
        let (first, last) = test.split_at(length);
        assert_eq!(first, "你好");
        assert_eq!(last, ",🌎");
    }
}
