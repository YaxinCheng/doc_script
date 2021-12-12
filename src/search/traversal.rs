pub struct Traversal<Type, GetNext> {
    current: Option<Type>,
    get_next: GetNext,
}

impl<Type, GetNext> Traversal<Type, GetNext>
where
    GetNext: Fn(&Type) -> Option<Type>,
{
    pub fn traverse(source: Type, get_next: GetNext) -> Self {
        Traversal {
            current: Some(source),
            get_next,
        }
    }
}

impl<Type, GetNext> Iterator for Traversal<Type, GetNext>
where
    GetNext: Fn(&Type) -> Option<Type>,
{
    type Item = Type;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.current.as_ref().and_then(|item| (self.get_next)(item)) {
            self.current.replace(next)
        } else {
            self.current.take()
        }
    }
}

#[cfg(test)]
mod traversal_test {
    use super::Traversal;

    #[test]
    fn test_traversal_forward() {
        let actual = Traversal::traverse(1, |item: &usize| Some(*item * 2))
            .take(5)
            .collect::<Vec<_>>();
        let expected = [1, 2, 4, 8, 16];
        assert_eq!(actual, expected)
    }
}
