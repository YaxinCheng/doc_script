use crate::ast::Parameter;
use std::collections::HashSet;

pub fn weed(parameters: &[Parameter]) {
    let mut existing_labels = HashSet::new();
    for parameter in parameters {
        match parameter {
            Parameter::Plain(_) => return,
            Parameter::Labelled { label, .. } => {
                if !existing_labels.insert(label) {
                    panic!("Duplicate label ({}) appeared in parameters", label)
                }
            }
        }
    }
}

#[cfg(test)]
mod parameter_weeder_tests {
    use super::weed;
    use crate::ast::{Expression, Name, Parameter};

    fn expression() -> Expression<'static> {
        Expression::ConstUse(Name::simple("expression"))
    }

    #[test]
    fn test_weed_plain_parameters() {
        let parameters = [
            Parameter::Plain(expression()),
            Parameter::Plain(expression()),
        ];
        weed(&parameters)
    }

    #[test]
    fn test_weed_labelled_parameters() {
        let parameters = [
            Parameter::Labelled {
                label: "first",
                content: expression(),
            },
            Parameter::Labelled {
                label: "second",
                content: expression(),
            },
        ];
        weed(&parameters)
    }

    #[test]
    #[should_panic]
    fn test_weed_labelled_parameters_duplicated() {
        let parameters = [
            Parameter::Labelled {
                label: "first",
                content: expression(),
            },
            Parameter::Labelled {
                label: "first",
                content: expression(),
            },
        ];
        weed(&parameters)
    }
}
