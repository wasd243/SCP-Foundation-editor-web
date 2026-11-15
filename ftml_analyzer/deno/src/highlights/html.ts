import { getLanguageService, TokenType } from "vscode-html-languageservice";

import { cssTokens, Token } from "./css.ts";

const html = getLanguageService();

export function htmlTokens(text: string): Token[] {
  const scanner = html.createScanner(text);
  const tokens: Token[] = [];

  for (let type = scanner.scan(); type !== TokenType.EOS; type = scanner.scan()) {
    const kind = htmlTokenKind(type);
    if (kind) {
      tokens.push({
        offset: scanner.getTokenOffset(),
        length: scanner.getTokenLength(),
        kind,
      });
    }
  }

  for (const range of styleRanges(text)) {
    for (const token of cssTokens(text.slice(range.start, range.end))) {
      tokens.push({ ...token, offset: token.offset + range.start });
    }
  }

  return tokens;
}

function htmlTokenKind(type: TokenType): string | undefined {
  switch (type) {
    case TokenType.StartTag:
    case TokenType.EndTag:
      return "type";
    case TokenType.AttributeName:
      return "property";
    case TokenType.AttributeValue:
      return "string";
    case TokenType.Comment:
    case TokenType.StartCommentTag:
    case TokenType.EndCommentTag:
      return "comment";
    case TokenType.Doctype:
    case TokenType.StartDoctypeTag:
    case TokenType.EndDoctypeTag:
      return "keyword";
    case TokenType.Script:
      return "string";
    default:
      return undefined;
  }
}

export function styleRanges(text: string): Array<{ start: number; end: number }> {
  const ranges: Array<{ start: number; end: number }> = [];
  const style = /<style\b[^>]*>([\s\S]*?)<\/style\s*>/gi;

  for (const match of text.matchAll(style)) {
    if (match.index === undefined || match[1] === undefined) continue;
    const start = text.indexOf(">", match.index) + 1;
    ranges.push({ start, end: start + match[1].length });
  }

  return ranges;
}

export function styleDocument(text: string): string {
  const ranges = styleRanges(text);
  const chars: string[] = text
    .split("")
    .map(char => (char === "\n" || char === "\r" ? char : " "));

  for (const range of ranges) {
    for (let i = range.start; i < range.end; i++) chars[i] = text[i];
  }

  return chars.join("");
}
