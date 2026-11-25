import * as path from "path";
import * as vscode from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
  const override = vscode.workspace.getConfiguration("ftml").get<string>("serverPath");
  const exe = process.platform === "win32" ? "ftml_analyzer.exe" : "ftml_analyzer";
  const command =
    override || context.asAbsolutePath(path.join("bin", `${process.platform}-${process.arch}`, exe));

  // An Executable without an explicit transport talks over stdio.
  const serverOptions: ServerOptions = { command };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ftml" }],
  };

  client = new LanguageClient("ftml", "FTML Analyzer", serverOptions, clientOptions);
  await client.start();
}

export function deactivate() {
  return client?.stop();
}