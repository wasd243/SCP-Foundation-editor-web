import { ExternalTokenizer } from "@lezer/lr"
import {
  Text,
  UnderlineText,
  StrikeText,
  Hr
} from "./parser.terms.js"

export const myTokenizer = new ExternalTokenizer((input, stack) => {
  const next = input.next
  if (next < 0) return

  // 1. 换行与空格绝对不碰
  if (next == 10 || next == 13 || next == 32 || next == 9) return

  // 2. 核心逻辑：凡是内置 @tokens 里定义的“多字符符号”，JS 必须提前闭嘴
  // [[ 或 [[/
  if (next == 91 && input.peek(1) == 91) return 
  // ]] 必须归还！否则所有 tagEnd 都会坏掉
  if (next == 93 && input.peek(1) == 93) return 
  // **
  if (next == 42 && input.peek(1) == 42) return 
  // // 但要排除 https:// 
  if (next == 47 && input.peek(1) == 47) {
    // 简单判定：如果前面是 ':'，那这就是链接，JS 把它吞了，不当斜体处理
    // 注意：input.peek(-1) 在某些版本的 Lezer 里不可用，我们改用 while 里的排除
  }
  // @@
  if (next == 64 && input.peek(1) == 64) return 
  // {{
  if (next == 123 && input.peek(1) == 123) return 
  // || 表格符号
  if (next == 124 && input.peek(1) == 124) return 

  // 3. 横杠系列 (---- 或 --)
  if (next == 45) {
    let count = 0
    while (input.peek(count) == 45) count++
    if (count >= 4) {
      input.acceptToken(Hr, count)
      return
    }
    if (count == 2) {
      input.acceptToken(StrikeText, 2)
      return
    }
    // 单个 - 留给 Text
  }

  // 4. 下划线系列 (__ )
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 5. 暴力 Text 逻辑：增加更多“禁区”
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      
      // 只要看到这些“可能是标记开头”的符号，Text 必须立刻停止
      if (
        curr == 91 || // [
        curr == 93 || // ]
        curr == 45 || // -
        curr == 95 || // _
        curr == 42 || // *
        curr == 47 || // /
        curr == 43 || // +
        curr == 64 || // @
        curr == 123 || // {
        curr == 124 || // | (解决表格坏掉的关键)
        curr == 126 || // ~ (表格标题)
        curr == 10 || curr == 13 // 换行
      ) break
      len++
    }
    
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 6. 最后一道防线：单字符回退
  // 如果是孤立的 [ 或 ]，由 Text 认领
  input.acceptToken(Text, 1)
})