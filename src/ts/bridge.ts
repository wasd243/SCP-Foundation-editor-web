import { tokenization } from "ftml_handler";

export interface FtmlToken {
  token: string;
  slice: string;
  span: { start: number; end: number };
  "lsp-mark"?: "html" | "css";
  "include-fold-mark"?: "include-start" | "include-end";
}

export interface FtmlDiagnostic {
  msg: string;
  span: { start: number; end: number };
}

export interface FtmlResult {
  tokens: FtmlToken[];
  diagnostics: FtmlDiagnostic[];
}

const EMPTY: FtmlResult = { tokens: [], diagnostics: [] };

// don't redo the work for the same text
let lastSrc: string | undefined;
let lastResult: FtmlResult = EMPTY;

export function tokenize(src: string): FtmlResult {
  if (src === lastSrc) return lastResult;
  try {
    const result = tokenization(src) as FtmlResult;
    lastSrc = src;
    lastResult = result;
    return result;
  } catch (err) {
    console.error("tokenization failed:", err);
    return EMPTY;
  }
}
