import * as monaco from "monaco-editor";
import EditorWorker from "monaco-editor/editor/editor.worker?worker";

// temperarily only import defualt, later for theme configurations
import template from "./templates/default.json";

self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};

monaco.languages.register({ id: "wikidot" });

monaco.languages.setLanguageConfiguration("wikidot", {
  brackets: [["[[", "]]"]],
  autoClosingPairs: [
    { open: "[!--", close: "--]" },
    { open: "[["  , close: "]]"  },
    { open: "["   , close: "]"   },
    { open: "[[/" , close: "]]"  },
    { open: "{"   , close: "}"   },
  ],
});

monaco.editor.defineTheme("scp-dark", template as monaco.editor.IStandaloneThemeData);

const BLOCKS                   = ["note", "=", ">", "<"];                       // Blocks without attributes
const BLOCKS_NO_LINEBREAK      = ["footnote"];                                  // Blocks without attributes but don't need linebreak
const ATTR_BLOCKS              = ["span", "div"];                               // Blocks with attributes
const ATTR_BLOCKS_NO_LINEBREAK = ["size", "module", "html"];                    // Blocks with attributes but don't need linebreak

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
        sugg(
          "collapsible",
          `collapsible show="$1" hide="$2"]]\n$0\n[[/collapsible`,
          range,
        ),
        sugg("footnoteblock", "footnoteblock", range),

        ...ATTR_BLOCKS.map((name) =>
          sugg(name, `${name} $1]]\n$0\n[[/${name}`, range),
        ),
        ...ATTR_BLOCKS_NO_LINEBREAK.map((name) =>
          sugg(name, `${name} $1]] $0 [[/${name}`, range),
        ),
        ...BLOCKS.map((name) =>
          sugg(name, `${name}]]\n$0\n[[/${name}`, range),
        ),
        ...BLOCKS_NO_LINEBREAK.map((name) =>
          sugg(name, `${name}]] $0 [[/${name}`, range),
        ),
    ],
    };
  },
});

/**
 * This function is used to reduce suggestion boilerplate.
 */
function sugg(
  name: string,
  insertText: string,
  range: monaco.IRange,
): monaco.languages.CompletionItem {
  return {
    label: name,
    kind: monaco.languages.CompletionItemKind.Snippet,
    insertText,
    insertTextRules:
      monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    range,
  };
}
