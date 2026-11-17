use lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

macro_rules! info {
    ( $( ($name:expr, $doc:expr, $attrs:expr, $closes:expr) ),* $(,)? ) => {
        &[ $( BlockInfo::new($name, $doc, $attrs, $closes) ),* ]
    };
}

/// Different with `InlineBlocks`, this contains `Blocks` that should be completed in a single line.
const BLOCKS_SHOULD_INLINE: &[&str] = &["footnote", "span", "size"];

const NO_ATTR: &[&str] = &[""];

const SIZE_ATTRS: &[&str] = &[
    "large", "small", "medium", "10px", "15px", "20px", "1em", "2em",
];

const HTML_LIKE_ATTRS: &[&str] = &["class", "id", "style"];

/// Registry completions here.
pub(super) const BLOCKS: &[BlockInfo] = info! {
    ("div", "", HTML_LIKE_ATTRS, true),
    ("span", "", HTML_LIKE_ATTRS, true),
    ("collapsible", "", &["show", "hide"], true),
    ("image", "", NO_ATTR, false),
    ("footnote", "", NO_ATTR, true),
    ("note", "", NO_ATTR, true),
    ("footnoteblock", "", NO_ATTR, false),
    // TODO: Currently, we could not fetch and resolve the included page, so the attrs could not
    // become like rust-analyser's function parameter completions.
    ("include", "", NO_ATTR, false),
    // Modules that have close tags like `module css`.
    ("module", "", &["css"], true),
    // Module rate.
    ("module rate", "", NO_ATTR, false),
    ("audio", "", &["align"], false),
    ("user", "", NO_ATTR, false),
    ("*user", "", NO_ATTR, false),
    ("size", "", SIZE_ATTRS, true),
    ("<", "", NO_ATTR, true),
    ("=", "", NO_ATTR, true),
    (">", "", NO_ATTR, true),
    // TODO: complete completion form.
};

#[derive(Debug)]
/// This is a builtin structure for auto completions.
pub(super) struct BlockInfo {
    pub(super) name: &'static str,
    doc: &'static str,
    attrs: &'static [&'static str],
    closes: bool,
}

impl BlockInfo {
    const fn new(
        name: &'static str,
        doc: &'static str,
        attrs: &'static [&'static str],
        closes: bool,
    ) -> Self {
        Self {
            name,
            doc,
            attrs,
            closes,
        }
    }

    /// Convert single `BlockInfo` to `CompletionItem`.
    pub(super) fn to_completion_item(&self) -> CompletionItem {
        let snippet = if self.closes {
            format!("{name}$1]]\n$0\n[[/{name}", name = self.name)
        } else if self.closes && BLOCKS_SHOULD_INLINE.contains(&self.name) {
            format!("{name}$1]]$0[[/{name}", name = self.name)
        } else {
            format!("{}$0", self.name)
        };

        CompletionItem {
            label: self.name.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(self.doc.to_owned()),
            insert_text: Some(snippet),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    }

    /// Convert attrs to completion items.
    pub(super) fn attrs_to_completion_items(&self) -> Vec<CompletionItem> {
        self.attrs
            .iter()
            .map(|a| CompletionItem {
                label: a.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                insert_text: Some(format!("{a}=\"$1\"$0")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }
}
