import { ExternalTokenizer } from "@lezer/lr"
import { Text, UnderlineText, StrikeText, Hr } from "./parser.terms.js"

export const myTokenizer = new ExternalTokenizer((input, stack) => {
  const { next } = input
  if (next < 0) return

  // 1. 绝对避让区域：空格、引号、等号、括号
  // 这样 @skip { space } 和内置的 Attr 规则才能活
  if (" \t\n\r=\"'[]{}|".includes(String.fromCharCode(next))) return

  // 2. 处理 Hr (----)
  if (next == 45) { // '-'
    let count = 0
    while (input.peek(count) == 45) count++
    if (count >= 4) { input.acceptToken(Hr, count); return }
  }

  // 3. 处理双发符号（只认领，不抢断）
  const next2 = input.peek(1)
  if (next == 95 && next2 == 95) { input.acceptToken(UnderlineText, 2); return }
  if (next == 45 && next2 == 45) { input.acceptToken(StrikeText, 2); return }

  // 4. 极其克制的 Text：一次只吞一个字符
  // 只要不是上述避让字符，就认领为 Text
  input.acceptToken(Text, 1)
})