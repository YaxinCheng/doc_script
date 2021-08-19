pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "break"
            | "default"
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
    )
}

#[cfg(test)]
mod keyword_tests {
    use super::is_keyword;

    const KEYWORDS: &[&str] = &[
        "break", "const", "continue", "default", "else", "for", "fn", "if", "impl", "return",
        "super", "struct", "self", "use",
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
