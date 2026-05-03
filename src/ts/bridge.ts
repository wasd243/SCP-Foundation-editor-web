import { tokenization } from "ftml_handler";

export interface FtmlToken {
  token: string;
  slice: string;
  span: { start: number; end: number };
}

export function tokenize(src: string): FtmlToken[] {
  try {
    return tokenization(src) as FtmlToken[];
  } catch (err) {
    console.error("tokenization failed:", err);
    return [];
  }
}
