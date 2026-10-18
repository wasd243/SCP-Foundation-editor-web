import * as monaco from "monaco-editor";
import { tokenize, type FtmlToken } from "./bridge";

const TYPES = [
  "block", "link", "markup", "structure", "table", "comment",
  "raw", "variable", "operator", "keyword", "type", "attribute",
];

// ftml token name -> semantic type. Tokens not listed here get no color.
const TYPE_BY_TOKEN: Record<string, string> = {
  "left-block": "block",
  "left-block-end": "block",
  "left-block-anchor": "block",
  "left-block-star": "block",
  "right-block": "block",
  "left-math": "block",
  "right-math": "block",

  "left-bracket": "link",
  "left-bracket-anchor": "link",
  "left-bracket-star": "link",
  "right-bracket": "link",
  "left-link": "link",
  "left-link-star": "link",
  "right-link": "link",
  url: "link",
  email: "link",

  bold: "markup",
  italics: "markup",
  underline: "markup",
  superscript: "markup",
  subscript: "markup",
  "left-monospace": "markup",
  "right-monospace": "markup",
  color: "markup",
  "double-dash": "markup",
  "triple-dash": "markup",
  "double-tilde": "markup",
  "left-double-angle": "markup",
  "clear-float-both": "markup",
  "clear-float-left": "markup",
  "clear-float-right": "markup",

  heading: "structure",
  "bullet-item": "structure",
  "numbered-item": "structure",
  quote: "structure",

  "table-column": "table",
  "table-column-right": "table",
  "table-column-center": "table",
  "table-column-title": "table",

  "left-comment": "comment",
  "right-comment": "comment",

  raw: "raw",
  "left-raw": "raw",
  "right-raw": "raw",
  "escaped-double-quote": "raw",
  "escaped-backslash": "raw",

  variable: "variable",

  pipe: "operator",
  equals: "operator",
  colon: "operator",
};

const OPENERS = new Set([
  "left-block",
  "left-block-end",
  "left-block-anchor",
  "left-block-star",
]);

function typeOf(tokens: FtmlToken[], i: number): string | undefined {
  const t = tokens[i];
  if (t.token !== "identifier") return TYPE_BY_TOKEN[t.token];

  const prev = tokens[i - 1];
  const prev2 = tokens[i - 2];
  const next = tokens[i + 1];

  // [[div ...]] -> block name
  if (prev && OPENERS.has(prev.token)) return "keyword";
  // [[module css]] -> module name
  if (prev?.slice === "module" && prev2 && OPENERS.has(prev2.token)) return "type";
  // id="..." -> attribute name
  if (next?.token === "equals") return "attribute";
  return undefined;
}

const legend: monaco.languages.SemanticTokensLegend = {
  tokenTypes: TYPES,
  tokenModifiers: [],
};

const provider = monaco.languages.registerDocumentSemanticTokensProvider(
  "wikidot",
  {
    getLegend: () => legend,
    provideDocumentSemanticTokens(model) {
      const tokens = tokenize(model.getValue());
      const data: number[] = [];
      let prevLine = 1;
      let prevChar = 0;

      for (let i = 0; i < tokens.length; i++) {
        const type = typeOf(tokens, i);
        if (type === undefined) continue;

        const start = model.getPositionAt(tokens[i].span.start);
        const end = model.getPositionAt(tokens[i].span.end);
        if (start.lineNumber !== end.lineNumber) continue; // no multi-line tokens

        const line = start.lineNumber;
        const ch = start.column - 1;
        data.push(
          line - prevLine,
          line === prevLine ? ch - prevChar : ch,
          end.column - start.column,
          TYPES.indexOf(type),
          0,
        );
        prevLine = line;
        prevChar = ch;
      }

      return { data: new Uint32Array(data) };
    },
    releaseDocumentSemanticTokens() {},
  },
);

if (import.meta.hot) {
  import.meta.hot.dispose(() => provider.dispose());
}
