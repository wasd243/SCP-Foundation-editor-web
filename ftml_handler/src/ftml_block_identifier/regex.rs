/// Generate regex to ignore upper/lower case.
macro_rules! re_slice {
    ($name:ident, $regex:expr) => {
        pub(crate) static $name: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($regex).unwrap());
    };
}

re_slice!(MODULE, r"(?i)^module$");
re_slice!(CSS, r"(?i)^css$");
re_slice!(HTML, r"(?i)^html$");
