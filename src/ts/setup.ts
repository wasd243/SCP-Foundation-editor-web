import * as monaco from "monaco-editor";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";

self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

monaco.languages.register({ id: "wikidot" });

monaco.languages.setLanguageConfiguration("wikidot", {
  brackets: [["[[", "]]"]],
  autoClosingPairs: [
    { open: "[[" , close: "]]" },
    { open: "[[/", close: "]]" },
  ],
});

monaco.editor.defineTheme("scp-dark", {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "block", foreground: "569cd6" },
    { token: "link", foreground: "4fc1ff" },
    { token: "markup", foreground: "d7ba7d" },
    { token: "structure", foreground: "ce9178", fontStyle: "bold" },
    { token: "table", foreground: "dcdcaa" },
    { token: "comment", foreground: "6a9955", fontStyle: "italic" },
    { token: "raw", foreground: "ce9178" },
    { token: "variable", foreground: "b5cea8" },
    { token: "operator", foreground: "d4d4d4" },
    { token: "keyword", foreground: "c586c0" },
    { token: "type", foreground: "4ec9b0" },
    { token: "attribute", foreground: "9cdcfe" },
  ],
  colors: {},
});

// Blocks that follow the simple template: name]] ... [[/name
const BLOCKS = ["div", "note", "span", "size", "=", ">", "<"];

monaco.languages.registerCompletionItemProvider("*", {
  triggerCharacters: ["["],
  provideCompletionItems(model, position) {
    const prefixRange = {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: Math.max(1, position.column - 2),
      endColumn: position.column,
    };
    const prefix = model.getValueInRange(prefixRange);
    if (prefix !== "[[") return { suggestions: [] };

    const word = model.getWordUntilPosition(position);
    const range = {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn,
    };
    return {
      suggestions: [
        // Special case: has attributes
        {
          label: "collapsible",
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: `collapsible show="" hide=""]]\n$0[[/collapsible`,
          insertTextRules:
            monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: "footnoteblock",
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: `footnoteblock`,
          insertTextRules:
            monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        {
          label: "footnote",
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: `footnote]] $0 [[/footnote`,
          insertTextRules:
            monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        },
        // Everything else generated from the list
        ...BLOCKS.map((name) => ({
          label: name,
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: `${name}]]\n$0[[/${name}`,
          insertTextRules:
            monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          range,
        })),
      ],
    };
  },
});
