/// From <https://github.com/matklad/once_cell/blob/master/examples/regex.rs>
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> =
            once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

// Export the macro.
pub(crate) use regex;
