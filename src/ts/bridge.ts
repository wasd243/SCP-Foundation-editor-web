import { tokenization } from "ftml_handler";

export interface FtmlToken {
  token: string;
  slice: string;
  span: { start: number; end: number };
  "lsp-mark"?: "html" | "css"
}

export function tokenize(src: string): FtmlToken[] {
  try {
    return tokenization(src) as FtmlToken[];
  } catch (err) {
    console.error("tokenization failed:", err);
    return [];
  }
}
