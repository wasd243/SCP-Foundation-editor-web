use lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat};

#[derive(Debug)]
pub(super) struct BlockInfo {
    name: &'static str,
    doc: &'static str,
    attrs: &'static [&'static str],
    closes: bool,
}

impl BlockInfo {
    pub(super) fn to_completion_item(&self) -> CompletionItem {
        let snippet = if self.closes {
            format!("{name}$1]]\n$0\n[[/{name}]]", name = self.name)
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
}

pub(super) const BLOCKS: &[BlockInfo] = &[
    BlockInfo {
        name: "div",
        doc: "",
        attrs: &["class"],
        closes: true,
    },
    BlockInfo {
        name: "collapsible",
        doc: "",
        attrs: &["show", "hide"],
        closes: true,
    },
    BlockInfo {
        name: "image",
        doc: "",
        attrs: &["link"],
        closes: false,
    },
    // TODO: complete completion form.
];
