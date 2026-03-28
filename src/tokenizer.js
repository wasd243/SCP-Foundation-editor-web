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

  // 1. 【核心】绝对不吃任何空白字符！
  // 32: Space, 10: \n, 13: \r, 9: Tab
  if (next == 32 || next == 10 || next == 13 || next == 9) return

  // 2. 【核心】双发符号预判：如果是内置标签的开头/结尾，JS 直接闭嘴
  // 我们只判断开头，不认领 Token
  const next2 = input.peek(1)
  if (
    (next == 91 && next2 == 91) || // [[
    (next == 93 && next2 == 93) || // ]]
    (next == 42 && next2 == 42) || // **
    (next == 47 && next2 == 47) || // //
    (next == 124 && next2 == 124) || // ||
    (next == 64 && next2 == 64) || // @@
    (next == 123 && next2 == 123) || // {{
    (next == 95 && next2 == 95)    // __ (虽然下面有处理，但这里先避让)
  ) return

  // 3. 处理 Hr (----) 和 删除线 (--)
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
  }

  // 4. 处理下划线 (__ )
  if (next == 95 && next2 == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 5. 【智能 Text 逻辑】
  // 只吸取那些“确定不是标签开头”的字符
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      
      // 遇到空白字符停止（交给 @skip）
      if (curr == 32 || curr == 10 || curr == 13 || curr == 9) break
      
      // 遇到可能是标签开头的符号停止
      let c2 = input.peek(len + 1)
      if (
        (curr == 91 && c2 == 91) || // [[
        (curr == 93 && c2 == 93) || // ]]
        (curr == 42 && c2 == 42) || // **
        (curr == 47 && c2 == 47) || // //
        (curr == 124 && c2 == 124) || // ||
        (curr == 95 && c2 == 95) || // __
        (curr == 123 && c2 == 123) || // {{
        (curr == 64 && c2 == 64)      // @@
      ) break
      
      len++
      // 别吸太长，给 Parser 喘息机会
      if (len >= 50) break
    }
    
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 6. 最后的兜底
  input.acceptToken(Text, 1)
})