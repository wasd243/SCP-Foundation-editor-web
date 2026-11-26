import { cp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import { join, resolve } from "node:path";

const extensionRoot = resolve(import.meta.dirname, "..");
const basePath = process.argv[2] ?? "/SCP-Foundation-editor-web";
const output = resolve(extensionRoot, "pages");
const cache = resolve(extensionRoot, ".vscode-pages-cache");
const host = "wasd243.github.io";
const baseUrl = `https://${host}${basePath}`;

async function downloadWorkbench() {
  await rm(cache, { recursive: true, force: true });
  const command = process.platform === "win32" ? "npx.cmd" : "npx";
  const child = spawn(command, ["--yes", "@vscode/test-web@0.0.81", "--browser", "none", "--quality", "stable", "--commit", "1.91.0", "--testRunnerDataDir", cache], { cwd: extensionRoot });
  await new Promise((resolveDownload, reject) => {
    const timer = setInterval(async () => {
      try {
        const entries = await readdir(cache, { withFileTypes: true });
        if (entries.some((entry) => entry.isDirectory())) {
          clearInterval(timer);
          child.kill();
          resolveDownload();
        }
      } catch {}
    }, 500);
    child.once("error", reject);
    child.once("exit", (code) => {
      clearInterval(timer);
      if (code && code !== null) reject(new Error(`test-web exited with ${code}`));
    });
  });
  const entries = await readdir(cache, { withFileTypes: true });
  return join(cache, entries.find((entry) => entry.isDirectory()).name);
}

function escapeAttribute(value) {
  return JSON.stringify(value).replace(/"/g, "&quot;");
}

const workbench = await downloadWorkbench();
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
await cp(workbench, output, { recursive: true, filter: (source) => !source.includes(`${process.platform === "win32" ? "\\" : "/"}node_modules`) });
await cp(extensionRoot, join(output, "extensions", "ftml-analyzer-web"), { recursive: true, filter: (source) => !["node_modules", ".vscode-test-web", ".vscode-pages-cache", "pages", ".gitignore", ".vscodeignore", "ftml-analyzer-web-0.1.0.vsix"].some((ignored) => source.endsWith(ignored) || source.includes(`/${ignored}/`) || source.includes(`\\${ignored}\\`)) });

const main = (await readFile(join(workbench, "out", "browser", "esm", "main.js"), "utf8"))
  .replace("./workbench.api", `${baseUrl}/out/vs/workbench/workbench.web.main.internal.js`);
const configuration = {
  productConfiguration: { enableTelemetry: false },
  workspaceUri: { scheme: "tmp", path: "/default.code-workspace" },
  callbackRoute: "/callback",
  developmentOptions: {
    extensions: [{ scheme: "https", authority: host, path: `${basePath}/extensions/ftml-analyzer-web` }],
  },
};
const index = `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta id="vscode-workbench-web-configuration" data-settings="${escapeAttribute(configuration)}"><link rel="icon" href="${baseUrl}/favicon.ico"><link rel="stylesheet" href="${baseUrl}/out/vs/workbench/workbench.web.main.internal.css"></head><body><script>globalThis._VSCODE_FILE_ROOT=${JSON.stringify(`${baseUrl}/out/`)};</script><script src="${baseUrl}/out/nls.messages.js"></script><script type="module">${main}</script></body></html>`;
await writeFile(join(output, "index.html"), index);
await writeFile(join(output, "404.html"), index);
