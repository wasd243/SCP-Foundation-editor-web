<template>
  <div class="editor-pane">
    <div class="pane-header">
      <div class="pane-title">
        <span>源文本</span>
        <span class="pane-badge">FTML (H2O2 Engine)</span>
      </div>
      <span class="pane-stats">
        {{ stats.lines }} 行 · {{ stats.words }} 词 · {{ stats.chars }} 字符
      </span>
    </div>

    <div class="editor-wrap" ref="editorContainerRef"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onBeforeUnmount } from 'vue'
// 在这里引入你打包好的 CM6 和 Lezer 逻辑
import { EditorView } from "@codemirror/view"
import { EditorState } from "@codemirror/state"
// import { yourFtmlSetup } from './your-lezer-setup' 

// 1. 保留 Andy 的数据接口，这样父组件和 previewer 绝对不会报错
const props = defineProps<{
  modelValue: string
  stats: { lines: number; chars: number; words: number; tags: number }
  workerVersion: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'render'): void
  (e: 'save'): void
}>()

const editorContainerRef = ref<HTMLElement | null>(null)
let view: EditorView | null = null

// 2. 在组件挂载时，把你的 CM6 强行塞进 Vue 的 DOM 里
onMounted(() => {
  if (!editorContainerRef.value) return

  view = new EditorView({
    state: EditorState.create({
      doc: props.modelValue, // 初始化时读取父组件传来的文本
      extensions: [
        // ...把你的 lezer 语法高亮、快捷键插件放在这里...

        // 【核心传动轴】：监听 CM6 的变动，向 Vue 外部发送数据！
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            // 把纯净的文本字符串 Emit 出去，Previewer 就能接到了
            emit('update:modelValue', update.state.doc.toString())
          }
        })
      ]
    }),
    parent: editorContainerRef.value // 挂载到 Vue 的 ref 节点上
  })
})

// 3. 监听外部数据变化（比如用户点了"CLEAR"或者加载了云端文档）
watch(
  () => props.modelValue,
  (newVal) => {
    // 防止循环触发：只有当 Vue 传来的值和 CM6 内部的值不一样时才更新
    if (view && view.state.doc.toString() !== newVal) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: newVal }
      })
    }
  }
)

// 4. 优雅退场：防止 Vue 切换路由时内存泄漏
onBeforeUnmount(() => {
  if (view) {
    view.destroy()
    view = null
  }
})
</script>

<style scoped>
/* 把你原生的 CSS 搬过来，稍微适配一下 Vue 的 scoped */
.editor-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #282c34;
}
.editor-wrap {
  flex: 1;
  position: relative;
  overflow: hidden;
}
/* 确保 CM6 占满容器 */
:deep(.cm-editor) {
  height: 100%;
}
</style>
