import * as vscode from "vscode";
import {
  BrowserMessageReader,
  BrowserMessageWriter,
  LanguageClient,
} from "vscode-languageclient/browser";

let client: LanguageClient | undefined;
let worker: Worker | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const languageWorker = new Worker(
    vscode.Uri.joinPath(context.extensionUri, "dist", "wasm.worker.js").toString(),
  );
  languageWorker.postMessage({
    type: "ftml/initialize",
    wasmUrl: vscode.Uri.joinPath(
      context.extensionUri,
      "wasm",
      "ftml_lsp_wasm_bg.wasm",
    ).toString(),
  });
  worker = languageWorker;

  client = new LanguageClient(
    "ftml",
    "FTML Analyzer",
    async () => ({
      reader: new BrowserMessageReader(languageWorker),
      writer: new BrowserMessageWriter(languageWorker),
    }),
    {
      documentSelector: [{ language: "ftml" }],
    },
  );
  await client.start();
}

export async function deactivate(): Promise<void> {
  await client?.stop();
  worker?.terminate();
  client = undefined;
  worker = undefined;
}
