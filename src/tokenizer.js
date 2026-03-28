// 确保这一行在最顶端
import { ExternalTokenizer } from "@lezer/lr"

// 这里导入你生成的 terms
import {
  Text,
  UnderlineText,
  StrikeText,
  Hr,
  newline
} from "./parser.terms.js"

export const myTokenizer = new ExternalTokenizer((input, stack) => {
  const next = input.next
  if (next < 0) return

  // 换行逻辑
  if (next == 10 || next == 13) {
    input.acceptToken(newline, 1)
    return
  }

  // 横杠逻辑 (Hr: ----, Strike: --)
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

  // 下划线逻辑 (__ )
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // Text 批量吸尘器
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      // 遇到特殊符号就停
      if (
        curr == 91 || // [
        curr == 45 || // -
        curr == 95 || // _
        curr == 42 || // *
        curr == 47 || // /
        curr == 43 || // +
        curr == 124 || // |
        curr == 10 || curr == 13 // newline
      ) break
      len++
    }
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 兜底一个字符
  input.acceptToken(Text, 1)
})