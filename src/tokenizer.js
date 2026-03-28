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

  // 1. 【彻底放手】换行、空格、制表符。
  // 只有这样，你的 @skip { space } 才能在两个 Token 之间生效
  if (next == 10 || next == 13 || next == 32 || next == 9) return

  // 2. 【双符号紧急避让】
  // 只要当前位置可能是内置标签的开头或结尾，JS 必须立刻 return
  if (next == 91 && input.peek(1) == 91) return // [[
  if (next == 93 && input.peek(1) == 93) return // ]]
  if (next == 42 && input.peek(1) == 42) return // **
  if (next == 47 && input.peek(1) == 47) return // //
  if (next == 124 && input.peek(1) == 124) return // ||
  if (next == 64 && input.peek(1) == 64) return // @@
  if (next == 123 && input.peek(1) == 123) return // {{

  // 3. 【JS 自有逻辑】Hr 和 删除线
  if (next == 45) { // '-'
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
  }

  // 4. 下划线
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 5. 【Text 逻辑优化】
  // 我们不再用 while 循环猛吸，而是“看一步走一步”
  // 如果当前字符不是上述任何特殊符号的开头，我们就吃掉它
  
  // 判定：如果当前是 [ 但后面不是 [，或者是 ] 但后面不是 ]，可以吃
  // 否则，为了保险，我们只吃掉非特殊字符
  if (
    next !== 91 && // [
    next !== 93 && // ]
    next !== 42 && // *
    next !== 47 && // /
    next !== 95 && // _
    next !== 45 && // -
    next !== 124 && // |
    next !== 64 && // @
    next !== 123    // {
  ) {
    let len = 1
    while (true) {
      let c = input.peek(len)
      // 碰到任何潜在的特殊符号或空白，立刻停止
      if (c <= 32 || c == 91 || c == 93 || c == 42 || c == 47 || c == 95 || c == 45 || c == 124 || c == 64 || c == 123 || c == -1) break
      len++
    }
    input.acceptToken(Text, len)
    return
  }

  // 6. 最后的兜底：如果是单发的 [ 或 ]，以单字符 Text 形式发出去
  input.acceptToken(Text, 1)
})