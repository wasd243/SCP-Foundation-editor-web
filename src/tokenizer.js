import { ExternalTokenizer } from "@lezer/lr"
import { Text, UnderlineText, StrikeText, Hr } from "./parser.terms.js"

export const myTokenizer = new ExternalTokenizer((input) => {
  const { next } = input
  if (next < 0) return

  // 【核心改变】增加对路径常用字符的避让：冒号(:)、斜杠(/)、点(.)、连字符(-)
  // 这样 IncludePath 和 LinkURL 才能作为一个整体被 Parser 捕获
  if (" \t\n\r=\"'[]{}|:./#@+*".includes(String.fromCharCode(next))) return

  // 同时也避开所有的字母和数字，把它们留给 attrPathToken 和 includePathToken
  if ((next >= 48 && next <= 57) || (next >= 65 && next <= 90) || (next >= 97 && next <= 122)) {
    return
  }

  // 处理 Hr (----) 和 Strike (--)
  if (next == 45) { // '-'
    let count = 0
    while (input.peek(count) == 45) count++
    if (count >= 4) { input.acceptToken(Hr, count); return }
    if (count == 2) { input.acceptToken(StrikeText, 2); return }
  }

  // 只有真正没用的“杂碎”字符（比如中文字符、特殊标点）才认领为 Text
  // 而且我们尝试一次性多吞一点，直到遇到上述保留字符
  let size = 1
  while (true) {
    let n = input.peek(size)
    if (n < 0 || " \t\n\r=\"'[]{}|:./#@+*-".includes(String.fromCharCode(n))) break
    if ((n >= 48 && n <= 57) || (n >= 65 && n <= 90) || (n >= 97 && n <= 122)) break
    size++
  }
  input.acceptToken(Text, size)
})