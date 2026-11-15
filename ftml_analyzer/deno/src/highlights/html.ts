import { TokenType } from "vscode-html-languageservice";

type Tok = { offset: number; length: number; kind: number };

function tokens(req: Req): Tok[] {
  if (req.lang === "html") {
    const scanner = html.createScanner(req.text);
    const out: Tok[] = [];
    for (let t = scanner.scan(); t !== TokenType.EOS; t = scanner.scan()) {
      out.push({ offset: scanner.getTokenOffset(), length: scanner.getTokenLength(), kind: t} );
    }
    return out;
  }
  return [];
}
