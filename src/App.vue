<script setup lang="ts">
import { ref } from "vue";
import Editor from "./components/Editor.vue";
import EditorToolBar from "./components/EditorToolBar.vue";

const editorRef = ref<InstanceType<typeof Editor>>();
</script>

<template>
  <div class="app">
    <EditorToolBar @wrap="(open, close) => editorRef?.wrap(open, close)" />
    <Editor ref="editorRef" />
  </div>
</template>
