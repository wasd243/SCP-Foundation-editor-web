<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue";
import * as monaco from "monaco-editor";
import "../ts/setup";
import "../ts/semantic";
import { wrap } from "../ts/commands";

const container = ref<HTMLDivElement>();
// Keep the editor in a plain variable, not a ref: Vue must not proxy it.
let editor: monaco.editor.IStandaloneCodeEditor | undefined;

function wrapSelection(open: string, close: string) {
  if (editor) wrap(editor, open, close);
}

defineExpose({ wrap: wrapSelection });

onMounted(() => {
  editor = monaco.editor.create(container.value!, {
    value: "",
    language: "wikidot",
    theme: "scp-dark",
    lineNumbers: "on",
    automaticLayout: true,
    autoClosingBrackets: "always",
    wordBasedSuggestions: "currentDocument",
    wordWrap: "on",
    "semanticHighlighting.enabled": true,
  });

  editor.getModel()!.setEOL(monaco.editor.EndOfLineSequence.LF);
});

onBeforeUnmount(() => {
  editor?.dispose();
});
</script>

<template>
  <div ref="container" class="editor"></div>
</template>
