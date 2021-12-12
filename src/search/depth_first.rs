use std::option::Option::Some;

/// DepthFirst traverses with a depth first order
/// (**right to left** regarding the order of the given children).
pub struct DepthFirst<Type, Move, Verify> {
    get_children: Move,
    verifier: Verify,
    stack: Vec<Type>,
}

impl<Type, Move, Children, Verify> DepthFirst<Type, Move, Verify>
where
    Move: Fn(Type) -> Children,
    Children: IntoIterator<Item = Type>,
    Verify: Fn(&Type) -> bool,
{
    pub fn find(start: Type, target: Verify, get_children: Move) -> Self {
        DepthFirst {
            get_children,
            verifier: target,
            stack: vec![start],
        }
    }
}

impl<Type, Move, Children, Verify> Iterator for DepthFirst<Type, Move, Verify>
where
    Move: Fn(Type) -> Children,
    Children: IntoIterator<Item = Type>,
    Verify: Fn(&Type) -> bool,
{
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            if (self.verifier)(&item) {
                return Some(item);
            } else {
                self.stack.extend((self.get_children)(item))
            }
        }
        None
    }
}

#[cfg(test)]
mod depth_first_test {
    use crate::search::DepthFirst;

    #[test]
    fn test_depth_first_find_odd() {
        const LIMIT: usize = 10;
        let iter = DepthFirst::find(
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
                .into_iter()
            },
        )
        .collect::<Vec<_>>();
        let expected = vec![5, 1];
        assert_eq!(iter, expected)
    }
}
