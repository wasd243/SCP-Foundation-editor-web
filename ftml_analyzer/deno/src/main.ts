import { createInterface } from "node:readline";
import process from "node:process";
import { TextDocument } from "vscode-languageserver-textdocument";
import { getCSSLanguageService } from "vscode-css-languageservice";
import { getLanguageService as getHTMLLanguageService } from "vscode-html-languageservice";

type Req = {
  id: number;
  lang: "css" | "html";
  method: "diagnose" | "complete" | "hover";
  text: string;
  position: { line: number; character: number };
};

const css = getCSSLanguageService();
const html = getHTMLLanguageService();

function handle(req: Req): unknown {
  const doc = TextDocument.create(`virtual://doc.${req.lang}`, req.lang, 0, req.text);

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
      case "diagnose": return []; // the HTML service has no validation API
      case "complete": return html.doComplete(doc, req.position, page);
      case "hover": return html.doHover(doc, req.position, page);
    }
  }
}

(globalThis as any).handle = (json: string): string => {
  const req = JSON.parse(json);
  try {
    return JSON.stringify({ id: req.id, result: handle(req) ?? null });
  } catch (e) {
    return JSON.stringify({ id: req.id, error: String(e) });
  }
};
