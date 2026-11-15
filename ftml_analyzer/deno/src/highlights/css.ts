import { Scanner, TokenType } from "vscode-css-languageservice/lib/esm/parser/cssScanner.js";

export type Token = {
  offset: number;
  length: number;
  kind: string;
};

export function cssTokens(text: string): Token[] {
  const scanner = new Scanner();
  scanner.setSource(text);
  const tokens: Token[] = [];
  let depth = 0;
  let afterColon = false;

  for (let token = scanner.scan(); token.type !== TokenType.EOF; token = scanner.scan()) {
    let kind: string | undefined;

    switch (token.type) {
      case TokenType.CurlyL:
        depth++;
        afterColon = false;
        break;
      case TokenType.CurlyR:
        depth = Math.max(0, depth - 1);
        afterColon = false;
        break;
      case TokenType.Colon:
        if (depth > 0) afterColon = true;
        break;
      case TokenType.SemiColon:
        afterColon = false;
        break;
      case TokenType.AtKeyword:
        kind = "keyword";
        break;
      case TokenType.Ident:
        kind = depth > 0 ? (afterColon ? "variable" : "property") : "class";
        break;
      case TokenType.Hash:
        kind = depth > 0 ? "number" : "class";
        break;
      case TokenType.String:
        kind = "string";
        break;
      case TokenType.Num:
      case TokenType.Percentage:
      case TokenType.Dimension:
      case TokenType.Length:
      case TokenType.EMS:
      case TokenType.EXS:
      case TokenType.Time:
      case TokenType.Freq:
      case TokenType.Resolution:
        kind = "number";
        break;
      case TokenType.Comment:
      case TokenType.SingleLineComment:
        kind = "comment";
        break;
    }

    if (kind) {
      tokens.push({ offset: token.offset, length: token.len, kind });
    }
  }

  return tokens;
}
