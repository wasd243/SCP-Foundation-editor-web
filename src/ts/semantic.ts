import * as monaco from "monaco-editor";
import { tokenize, type FtmlToken, type FtmlDiagnostic } from "./bridge";
import { createHighlighter } from "shiki";

const SHIKI_THEME = "vitesse-dark";
const hl = await createHighlighter({ themes: [SHIKI_THEME], langs: ["html", "css"] });

const TYPES = [
  "block", "link", "markup", "structure", "table", "comment",
  "raw", "variable", "operator", "keyword", "type", "attribute", "heading",
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

  heading: "heading",
  "bullet-item": "structure",
  "numbered-item": "structure",
  quote: "comment",

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

function typeOf(tokens: FtmlToken[], index: number): string | undefined {
  const t = tokens[index];
  if (t["sub-lang-mark"]) return undefined;
  if (t.ty !== "identifier") return TYPE_BY_TOKEN[t.ty];

  const prev = tokens[index - 1];
  const prev2 = tokens[index - 2];
  const next = tokens[index + 1];

  // [[div ...]] -> block name
  if (prev && OPENERS.has(prev.ty)) return "keyword";
  // [[module css]] -> module name
  if (prev?.slice === "module" && prev2 && OPENERS.has(prev2.ty)) return "type";
  // id="..." -> attribute name
  if (next?.ty === "equals") return "attribute";
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
    provideDocumentSemanticTokens(model: monaco.editor.ITextModel) {
      const tokens: FtmlToken[] = tokenize(model.getValue()).tokens;
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

type SubHighlight = {
  span: { start: number; end: number };
  slice: string;
  lang: "html" | "css";
};

function collectSubHighlights(tokens: FtmlToken[], source: string): SubHighlight[] {
  const highlights: SubHighlight[] = [];
  let current: SubHighlight | undefined;

  for (const token of tokens) {
    const lang = token["sub-lang-mark"];
    if (!lang) {
      current = undefined;
      continue;
    }

    if (!current || current.lang !== lang) {
      current = {
        span: { start: token.span.start, end: token.span.end },
        slice: "",
        lang,
      };
      highlights.push(current);
    } else {
      current.span.end = token.span.end;
    }
    current.slice = source.slice(current.span.start, current.span.end);
  }

  return highlights;
}

const styleEl = document.head.appendChild(document.createElement("style"));
const colorClasses = new Map<string, string>();

function cssClassForColor(color: string): string {
  let cls = colorClasses.get(color);
  if (!cls) {
    cls = `sub-hl-${colorClasses.size}`;
    colorClasses.set(color, cls);
    styleEl.append(`.${cls}{color:${color} !important}`);
  }
  return cls;
}

function createSubHighlightDecorations(
  model: monaco.editor.ITextModel,
  highlight: SubHighlight,
): monaco.editor.IModelDeltaDecoration[] {
  const lines = hl.codeToTokensBase(highlight.slice, {
    lang: highlight.lang,
    theme: SHIKI_THEME,
  });
  const decorations: monaco.editor.IModelDeltaDecoration[] = [];

  for (const line of lines) {
    for (const token of line) {
      if (!token.color || token.content.length === 0) continue;
      const start: number = highlight.span.start + token.offset;
      const end: number = start + token.content.length;
      decorations.push({
        range: monaco.Range.fromPositions(
          model.getPositionAt(start),
          model.getPositionAt(end),
        ),
        options: { inlineClassName: cssClassForColor(token.color) },
      });
    }
  }

  return decorations;
}

// ignore false positives in `[[include]]` blocks
const INCLUDE_RE = /\[\[\s*include\s[\s\S]*?]]/gi;

function includeRanges(source: string): Array<[number, number]> {
  return Array.from(source.matchAll(INCLUDE_RE), (m): [number, number] => {
    const start: number = m.index ?? 0;
    return [start, start + m[0].length];
  });
}

function updateDiagnostics(
  model: monaco.editor.ITextModel,
  diagnostics: FtmlDiagnostic[],
): void {
  const ranges = includeRanges(model.getValue());
  const visible: FtmlDiagnostic[] = diagnostics.filter(
      (d) => !ranges.some(([from, to]) => d.span.start >= from && d.span.start < to),
  );

  const textLength: number = model.getValueLength();
  const markers: monaco.editor.IMarkerData[] = visible.map((d) => {
    const start: number = d.span.start;
    // zero-width squiggles are invisible, so widen to at least one character
    const end: number = Math.min(Math.max(d.span.end, start + 1), textLength);
    const start_ = model.getPositionAt(start);
    const end_ = model.getPositionAt(end);
    return {
      severity: monaco.MarkerSeverity.Warning,
      message: d.msg,
      startLineNumber: start_.lineNumber,
      startColumn: start_.column,
      endLineNumber: end_.lineNumber,
      endColumn: end_.column,
    };
  });
  monaco.editor.setModelMarkers(model, "ftml", markers);
}

const subDecorationIds = new WeakMap<monaco.editor.ITextModel, string[]>();

export function updateSubHighlights(model: monaco.editor.ITextModel): void {
  const source: string = model.getValue();
  const { tokens, diagnostics } = tokenize(source);
  const decorations: monaco.editor.IModelDeltaDecoration[] = [];

  for (const highlight of collectSubHighlights(tokens, source)) {
    decorations.push(...createSubHighlightDecorations(model, highlight));
  }

  const previous: string[] = subDecorationIds.get(model) ?? [];
  subDecorationIds.set(model, model.deltaDecorations(previous, decorations));

  updateDiagnostics(model, diagnostics);
}
