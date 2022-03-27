pub(crate) struct Iterating<E>(IteratorSize<E>);

enum IteratorSize<E> {
    Empty,
    Once(Option<E>),
    Twice(Option<(E, E)>),
    Many(Vec<E>),
}

impl<E> Iterating<E> {
    pub const fn empty() -> Self {
        Self(IteratorSize::Empty)
    }

    pub const fn once(element: E) -> Self {
        Self(IteratorSize::Once(Some(element)))
    }

    pub const fn twice(first: E, second: E) -> Self {
        Self(IteratorSize::Twice(Some((first, second))))
    }
}

impl<E> Iterator for Iterating<E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IteratorSize::Empty => None,
            IteratorSize::Once(element) => {
                let output = element.take();
                self.0 = IteratorSize::Empty;
                output
            }
            IteratorSize::Twice(elements) => {
                // consider using unzip when it becomes stable
                let (first, second) = elements.take().expect("Twice cannot be empty for sure");
                self.0 = IteratorSize::Once(Some(second));
                Some(first)
            }
            IteratorSize::Many(elements) => {
                let output = elements.pop();
                if elements.len() == 2 {
                    self.0 = IteratorSize::Twice(elements.pop().zip(elements.pop()));
                }
                output
            }
        }
    }
}

impl<E> FromIterator<E> for Iterating<E> {
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        let mut elements = iter.into_iter().collect::<Vec<_>>();
        elements.reverse();
        let iterator = match elements.len() {
            0 => IteratorSize::Empty,
            1 => IteratorSize::Once(elements.pop()),
            2 => IteratorSize::Twice(elements.pop().zip(elements.pop())),
            _ => IteratorSize::Many(elements),
        };
        Iterating(iterator)
    }
}

#[cfg(test)]
mod iterating_tests {
    use crate::iterating::Iterating;

    #[test]
    fn test_none() {
        let mut iterator = Iterating::<()>::empty();
        assert_eq!(iterator.next(), None)
    }

    #[test]
    fn test_once_iter() {
        let mut iterator = Iterating::once(42);
        assert_eq!(iterator.next(), Some(42));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_twice_iter() {
        let mut iterator = Iterating::twice(1, 2);
        assert_eq!(iterator.next(), Some(1));
        assert_eq!(iterator.next(), Some(2));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_from_iter_none() {
        let mut iterator = std::iter::empty::<()>().collect::<Iterating<_>>();
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_from_iter_once() {
        let mut iterator = std::iter::once(42).collect::<Iterating<_>>();
        assert_eq!(iterator.next(), Some(42));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_from_iter_twice() {
        let mut iterator = std::iter::once(1)
            .chain(std::iter::once(2))
            .collect::<Iterating<_>>();
        assert_eq!(iterator.next(), Some(1));
        assert_eq!(iterator.next(), Some(2));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_many() {
        let mut iterator = vec![1, 2, 3, 4].into_iter().collect::<Iterating<_>>();
        assert_eq!(iterator.next(), Some(1));
        assert_eq!(iterator.next(), Some(2));
        assert_eq!(iterator.next(), Some(3));
        assert_eq!(iterator.next(), Some(4));
        assert_eq!(iterator.next(), None);
    }
}
