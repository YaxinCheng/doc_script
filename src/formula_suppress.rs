#![cfg(test)]
/// FormulaSuppress used in tests to suppress one or many of following processes:
///     * preluding std library into GLOBAL scope (some tests do not test std)
///     * entry constant check
///
/// # Note:
/// FormulaSuppress has already considered that tests are run under multithreading
/// environment, and it has used different thread id to generate distinguishable
/// keys as environment variables
///
/// # Warning:
/// changed environment will be unset when the object is dropped
#[derive(Default)]
pub struct FormulaSuppress {
    prelude_std_enabled: bool, // true to enable, false to suppress
    entry_check_enabled: bool,
}

impl FormulaSuppress {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn allow_prelude_std() -> Self {
        FormulaSuppress {
            prelude_std_enabled: true,
            ..Default::default()
        }
    }

    pub fn suppress(&self) {
        std::env::set_var(
            Self::prelude_std_key(),
            self.prelude_std_enabled.to_string(),
        );
        std::env::set_var(
            Self::entry_check_key(),
            self.entry_check_enabled.to_string(),
        );
    }

    pub fn prelude_std_suppressed() -> bool {
        std::env::var(Self::prelude_std_key())
            .map(|value| value == "false")
            .unwrap_or_default()
    }

    pub fn entry_check_suppressed() -> bool {
        std::env::var(Self::entry_check_key())
            .map(|value| value == "false")
            .unwrap_or_default()
    }

    pub fn prelude_std_key() -> String {
        let thread_id = std::thread::current().id();
        format!("doc_script:test:PRELUDE_STD:{thread_id:?}")
    }

    pub fn entry_check_key() -> String {
        let thread_id = std::thread::current().id();
        format!("doc_script:test:ENTRY_CHECK:{thread_id:?}")
    }
}

impl Drop for FormulaSuppress {
    fn drop(&mut self) {
        std::env::remove_var(FormulaSuppress::prelude_std_key());
        std::env::remove_var(FormulaSuppress::entry_check_key());
    }
}
