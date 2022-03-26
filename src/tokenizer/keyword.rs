pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "break"
            | "const"
            | "continue"
            | "else"
            | "for"
            | "fn"
            | "if"
            | "impl"
            | "return"
            | "super"
            | "struct"
            | "self"
            | "use"
            | "trait"
            | "in"
    )
}

#[cfg(test)]
mod keyword_tests {
    use super::is_keyword;

    const KEYWORDS: &[&str] = &[
        "break", "const", "continue", "else", "for", "fn", "if", "impl", "return", "super",
        "struct", "self", "use", "trait", "in",
    ];

    #[test]
    fn test_keyword_pass() {
        for keyword in KEYWORDS {
            assert!(is_keyword(keyword))
        }
    }

    #[test]
    fn test_keyword_with_suffix() {
        for keyword in KEYWORDS {
            let keyword = format!("{}_suffix", keyword);
            assert!(!is_keyword(&keyword))
        }
    }
}
