<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue";
import * as monaco from "monaco-editor";
import "../ts/sugg";
import "../ts/semantic";
import "../ts/semantic/foldProvider";
import { updateSubHighlights } from "../ts/semantic";
import { wrap } from "../ts/commands";

const container = ref<HTMLDivElement>();
// Keep the editor in a plain variable, not a ref: Vue must not proxy it.
let editor: monaco.editor.IStandaloneCodeEditor | undefined;
let timer: ReturnType<typeof setTimeout> | undefined;

function wrapSelection(open: string, close: string) {
  if (editor) wrap(editor, open, close);
}

defineExpose({ wrap: wrapSelection });

onMounted(() => {
  editor = monaco.editor.create(container.value!, {
    value                         : "",
    language                      : "wikidot",
    theme                         : "scp-dark",
    lineNumbers                   : "on",
    automaticLayout               :  true,
    autoClosingBrackets           : "always",
    autoClosingDelete             : "always",
    wordBasedSuggestions          : "currentDocument",
    wordWrap                      : "on",
    "semanticHighlighting.enabled": true,
    unicodeHighlight: {
      ambiguousCharacters         : false,
      invisibleCharacters         : false,
      nonBasicASCII               : false,
    },
    bracketPairColorization: { 
      enabled                     : false, 
    },
    guides: {
      bracketPairs                : false,
    },
    matchBrackets                 : "never",
  });

  const model = editor.getModel()!;
  model.setEOL(monaco.editor.EndOfLineSequence.LF);

  updateSubHighlights(model);
  editor.onDidChangeModelContent(() => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => updateSubHighlights(model), 100);
  });
});

onBeforeUnmount(() => {
  if (timer !== undefined) clearTimeout(timer);
  editor?.dispose();
});
</script>

<template>
  <div ref="container" class="editor"></div>
</template>
