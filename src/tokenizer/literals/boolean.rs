pub fn is_boolean(s: &str) -> bool {
    matches!(s, "true" | "false")
}