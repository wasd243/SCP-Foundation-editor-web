//! Use toml for expectations
//! ```toml,ignore
//! [[expectation]]
//! kind = "Comment"
//! span = { start = [u32, u32], end = [u32, u32] }
//! ```
use std::str::FromStr;

use ftml_analyzer::nodes::Node;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Expect {
    pub expectation: Vec<Expectation>,
}

#[derive(Debug, Deserialize)]
pub struct Expectation {
    pub kind: String,
    pub span: ExpectSpan,
}

#[derive(Debug, Deserialize)]
pub struct ExpectSpan {
    pub start: (u32, u32),
    pub end: (u32, u32),
}

impl FromStr for Expect {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Expectation {
    pub fn matches_node(&self, node: &Node) -> bool {
        let (kind, node_span) = node.info();
        self.kind == kind && self.span.start == node_span.start && self.span.end == node_span.end
    }
}
