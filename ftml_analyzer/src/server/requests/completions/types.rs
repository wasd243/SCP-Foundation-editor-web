use lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

macro_rules! info {
    ($name:expr, $doc:expr, $attrs:expr, $closes:expr) => {
        BlockInfo::new($name, $doc, $attrs, $closes)
    };
}

/// Registry completions here.
pub(super) const BLOCKS: &[BlockInfo] = &[
    info!("div", "", &["class", "id", "style"], true),
    info!("collapsible", "", &["show=", "hide="], true),
    info!("image", "", &[""], false),
    // TODO: complete completion form.
];

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
        } else {
            format!("{}$0]]", self.name)
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
                insert_text: Some(format!("{a}=\"$1\"")),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }
}
