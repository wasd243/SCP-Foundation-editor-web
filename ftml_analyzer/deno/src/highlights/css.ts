import { Scanner, TokenType as CssTokenType } from "vscode-css-languageservice/lib/esm/parser/cssScanner.js";

function cssTokens(text: string): Tok[] {
  const sc = new Scanner();
  sc.setSource(text);
  const out: Tok[] = [];
  let depth = 0;
  let afterColon = false;

  for (let t = sc.scan(); t.type !== CssTokenType.EOF; t = sc.scan()) {
    let kind = -1; 
    switch (t.type) {
      case CssTokenType.CurlyL: depth++; afterColon = false; break;
      case CssTokenType.CurlyR: depth--; afterColon = false; break;
      case CssTokenType.Colon: if (depth > 0) afterColon = true; break;
      case CssTokenType.SemiColon: afterColon = false; break;
      case CssTokenType.AtKeyword: kind = 0; break;
      case CssTokenType.Ident:
        kind = depth > 0 ? (afterColon ? 3 : 1) : 2;
        break;
      case CssTokenType.Hash: kind = depth > 0 ? 4 : 2; break;
      case CssTokenType.String: kind = 5; break;
      case CssTokenType.Num:
      case CssTokenType.Percentage:
      case CssTokenType.Dimension:
      case CssTokenType.Length: kind = 4; break;
    }
    if (kind >= 0) out.push({ offset: t.offset, length: t.len, kind });
  }
  return out;
}
