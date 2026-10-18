import type * as monaco from "monaco-editor";

/** Wrap the selection with markers, or insert an empty pair and put the cursor between. */
export function wrap(
  editor: monaco.editor.IStandaloneCodeEditor,
  open: string,
  close: string,
): void {
  const model = editor.getModel();
  const selection = editor.getSelection();
  if (!model || !selection) return;

  const text = model.getValueInRange(selection);

  editor.pushUndoStop();
  editor.executeEdits("toolbar", [
    { range: selection, text: open + text + close, forceMoveMarkers: true },
  ]);
  editor.pushUndoStop();

  if (text === "") {
    editor.setPosition({
      lineNumber: selection.startLineNumber,
      column: selection.startColumn + open.length,
    });
  }
  editor.focus();
}
