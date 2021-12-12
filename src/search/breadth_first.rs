use std::collections::VecDeque;
use std::fmt::Debug;

pub struct BreadthFirst<Type, Move, Verify> {
    get_children: Move,
    verifier: Verify,
    queue: VecDeque<Type>,
}

impl<Type, Move, Children, Verify> BreadthFirst<Type, Move, Verify>
where
    Move: Fn(Type) -> Children,
    Children: IntoIterator<Item = Type>,
    Verify: Fn(&Type) -> bool,
{
    pub fn find(start: Type, target: Verify, get_children: Move) -> Self {
        BreadthFirst {
            get_children,
            verifier: target,
            queue: VecDeque::from([start]),
        }
    }

    pub fn find_from(
        start: impl IntoIterator<Item = Type>,
        target: Verify,
        get_children: Move,
    ) -> Self {
        BreadthFirst {
            get_children,
            verifier: target,
            queue: start.into_iter().collect(),
        }
    }
}

impl<Type, Move, Children, Verify> Iterator for BreadthFirst<Type, Move, Verify>
where
    Type: Debug,
    Move: Fn(Type) -> Children,
    Children: IntoIterator<Item = Type>,
    Verify: Fn(&Type) -> bool,
{
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.queue.pop_front() {
            if (self.verifier)(&item) {
                return Some(item);
            } else {
                self.queue.extend((self.get_children)(item))
            }
        }
        None
    }
}

#[cfg(test)]
mod breadth_first_test {
    use super::BreadthFirst;

    #[test]
    fn test_breadth_first_find_odd() {
        const LIMIT: usize = 10;
        let iter = BreadthFirst::find(
            0_usize,
            |value| value % 2 == 1,
            |value| {
                if value * 2 + 1 > LIMIT {
                    vec![]
                } else if value * 2 + 2 < LIMIT {
                    vec![value * 2 + 1, value * 2 + 2]
                } else {
                    vec![value * 2 + 1]
                }
            },
        )
        .collect::<Vec<_>>();
        let expected = vec![1, 5];
        assert_eq!(iter, expected)
    }
}
