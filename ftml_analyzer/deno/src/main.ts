import { TextDocument } from "vscode-languageserver-textdocument";
import { getCSSLanguageService } from "vscode-css-languageservice";
import { getLanguageService as getHTMLLanguageService } from "vscode-html-languageservice";

import { htmlTokens } from "./highlights/html.ts";
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
  if (req.method === "tokens") {
    return req.lang === "css" ? cssTokens(req.text) : htmlTokens(req.text);
  }

  const doc = TextDocument.create(`virtual://doc.${req.lang}`, req.lang, 0, req.text);

  if (req.method === "tokens") {
    const raw = req.lang === "css" ? cssTokens(req.text) : htmlTokens(req.text);
    return raw.map(t => {
      const p = doc.positionAt(t.offset);
      return { line: p.line, character: p.character, length: t.length, kind: t.kind };
    });
  }

  if (req.lang === "css") {
    const sheet = css.parseStylesheet(doc);
    switch (req.method) {
      case "diagnose": return css.doValidation(doc, sheet);
      case "complete": return css.doComplete(doc, req.position, sheet);
      case "hover": return css.doHover(doc, req.position, sheet);
    }
  } else {
    const page = html.parseHTMLDocument(doc);
    switch (req.method) {
      // TODO: temparaory no diagnostics on html server.
      case "diagnose": return [];
      case "complete": return html.doComplete(doc, req.position, page);
      case "hover": return html.doHover(doc, req.position, page);
    }
  }
}

(globalThis as any).handle = (req: Req): string => {
  try {
    return JSON.stringify({ result: handle(req) ?? null });
  } catch (e) {
    return JSON.stringify({ error: String(e) });
  }
};

