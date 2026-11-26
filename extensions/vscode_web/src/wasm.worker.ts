import initialize, {
  handle_message,
  reset,
} from "../wasm/ftml_lsp_wasm.js";

let initialized: Promise<void> | undefined;

self.addEventListener("message", async (event: MessageEvent<unknown>) => {
  const data = event.data as { type?: string; wasmUrl?: string };
  if (data.type === "ftml/initialize" && data.wasmUrl) {
    initialized = initialize(data.wasmUrl).then(reset);
    return;
  }

  if (!initialized) {
    throw new Error("WASM worker received an LSP message before initialization");
  }

  await initialized;
  const replies: unknown[] = JSON.parse(handle_message(JSON.stringify(event.data)));
  for (const reply of replies) {
    self.postMessage(reply);
  }
});
