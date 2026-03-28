import { ExternalTokenizer } from "@lezer/lr"
// 1. 这里删掉 newline，只留下你真正需要的
import {
  Text,
  UnderlineText,
  StrikeText,
  Hr
} from "./parser.terms.js"

export const myTokenizer = new ExternalTokenizer((input, stack) => {
  const next = input.next
  if (next < 0) return

  // 2. 直接用数字判断换行，不使用变量
  // 10 = \n, 13 = \r
  if (next == 10 || next == 13) {
    // 这里我们不 acceptToken，让 Lezer 的内置 @tokens newline 去处理
    // 直接返回，tokenizer 就会跳过这个字符
    return 
  }

  // 3. 横杠逻辑 (---- 或 --)
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

  // 4. 下划线逻辑 (__ )
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 5. 暴力 Text 逻辑
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      // 遇到这些符号就“刹车”
      if (
        curr == 91 || // [
        curr == 45 || // -
        curr == 95 || // _
        curr == 42 || // *
        curr == 47 || // /
        curr == 43 || // +
        curr == 124 || // |
        curr == 10 || curr == 13 // 换行符
      ) break
      len++
    }
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 6. 兜底吞掉一个字符，防止死循环
  input.acceptToken(Text, 1)
})