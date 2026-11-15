import { TextDocument } from "vscode-languageserver-textdocument";
import { getCSSLanguageService } from "vscode-css-languageservice";
import { getLanguageService as getHTMLLanguageService } from "vscode-html-languageservice";

import { htmlTokens, styleDocument, styleRanges } from "./highlights/html.ts";
import { cssTokens } from "./highlights/css.ts";

type Req = {
  lang: "css" | "html";
  method: "diagnose" | "complete" | "hover" | "tokens";
  text: string;
  position: { line: number; character: number };
};

const css = getCSSLanguageService();
const html = getHTMLLanguageService();

function handle(req: Req): unknown {
  const doc = TextDocument.create(`virtual://doc.${req.lang}`, req.lang, 0, req.text);

  if (req.method === "tokens") {
    const raw = req.lang === "css" ? cssTokens(req.text) : htmlTokens(req.text);
    return raw.map(token => {
      const position = doc.positionAt(token.offset);
      return {
        line: position.line,
        character: position.character,
        length: token.length,
        kind: token.kind,
      };
    });
  }

  if (req.lang === "css") {
    const sheet = css.parseStylesheet(doc);
    switch (req.method) {
      case "diagnose":
        return css.doValidation(doc, sheet);
      case "complete":
        return css.doComplete(doc, req.position, sheet);
      case "hover":
        return css.doHover(doc, req.position, sheet);
    }
  }

  const page = html.parseHTMLDocument(doc);
  const styleText = styleDocument(req.text);
  const styleDoc = TextDocument.create("virtual://doc.style.css", "css", 0, styleText);
  const styleSheet = css.parseStylesheet(styleDoc);
  const offset = doc.offsetAt(req.position);
  const inStyle = styleRanges(req.text).some(
    range => offset >= range.start && offset <= range.end,
  );

  switch (req.method) {
    case "diagnose":
      return css.doValidation(styleDoc, styleSheet);
    case "complete":
      return inStyle
        ? css.doComplete(styleDoc, req.position, styleSheet)
        : html.doComplete(doc, req.position, page);
    case "hover":
      return inStyle
        ? css.doHover(styleDoc, req.position, styleSheet)
        : html.doHover(doc, req.position, page);
  }
}

(globalThis as any).handle = (req: Req): string => {
  try {
    return JSON.stringify({ result: handle(req) ?? null });
  } catch (e) {
    return JSON.stringify({ error: String(e) });
  }
};
