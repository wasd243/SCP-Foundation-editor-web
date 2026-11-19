use regex::Regex;
use std::sync::LazyLock;

pub(super) static CSS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^css$").unwrap());

pub(super) static HTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^html$").unwrap());

pub(super) static MODULE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^module$").unwrap());

pub(super) static CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^code$").unwrap());

pub(super) static TYPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^type$").unwrap());
