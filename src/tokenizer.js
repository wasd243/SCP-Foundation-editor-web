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

  // 1. 换行与空白字符归还内置
  if (next == 10 || next == 13 || next == 32 || next == 9) return

  // 2. 【核心避让】只要是双发符号，JS 绝对不碰，让给 @tokens
  // 这里的 peek(1) 检查非常关键
  if (next == 91 && input.peek(1) == 91) return // [[
  if (next == 93 && input.peek(1) == 93) return // ]] (修复 TagEnd 的关键)
  if (next == 42 && input.peek(1) == 42) return // **
  if (next == 47 && input.peek(1) == 47) return // //
  if (next == 124 && input.peek(1) == 124) return // ||
  if (next == 64 && input.peek(1) == 64) return // @@
  if (next == 123 && input.peek(1) == 123) return // {{
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 3. 横杠逻辑 (Hr: ----, Strike: --)
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
    // 单个 - 交给 Text
  }

  // 4. 吸尘器 Text 逻辑
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      
      // 只要看到这些“双发预警符号”，立刻刹车
      // 这样能保证 [[, ]], **, //, || 等永远能被内置识别
      if (
        (curr == 91 && input.peek(len + 1) == 91) || // [[
        (curr == 93 && input.peek(len + 1) == 93) || // ]]
        (curr == 42 && input.peek(len + 1) == 42) || // **
        (curr == 47 && input.peek(len + 1) == 47) || // //
        (curr == 124 && input.peek(len + 1) == 124) || // ||
        (curr == 64 && input.peek(len + 1) == 64) || // @@
        (curr == 123 && input.peek(len + 1) == 123) || // {{
        (curr == 95 && input.peek(len + 1) == 95) || // __
        curr == 10 || curr == 13 // 换行
      ) break

      // 如果是单字符（不是双发的开头），就吸走
      len++
      
      // 限制单次 Text 长度，防止它跑太远
      if (len > 100) break 
    }
    
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 5. 兜底逻辑：如果不是双发符号，就吃掉这一个字符作为 Text
  input.acceptToken(Text, 1)
})