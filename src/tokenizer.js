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

  // 1. 换行符避让：绝对不要在这里 accept 换行，直接还给内置引擎
  if (next == 10 || next == 13) return

  // 2. 横杠逻辑 (---- 或 --) 
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
    // 注意：如果是单个 '-'，不要在这里认领，往下走让 Text 吞掉或者还给内置
  }

  // 3. 下划线逻辑 (__ )
  if (next == 95 && input.peek(1) == 95) {
    input.acceptToken(UnderlineText, 2)
    return
  }

  // 4. 【关键修正】内置标签避让逻辑
  // 如果当前是 [ (91), * (42), / (47), + (43), @ (64), { (123) 等
  // 我们要看看它是不是双发的符号（[[ , **, // 等）
  if (next == 91 && input.peek(1) == 91) return // 发现 [[，还给内置
  if (next == 42 && input.peek(1) == 42) return // 发现 **，还给内置
  if (next == 47 && input.peek(1) == 47) return // 发现 //，还给内置
  if (next == 64 && input.peek(1) == 64) return // 发现 @@，还给内置
  if (next == 123 && input.peek(1) == 123) return // 发现 {{，还给内置

  // 5. 暴力 Text 逻辑 (吸尘器模式)
  if (next != -1) {
    let len = 0
    while (true) {
      let curr = input.peek(len)
      if (curr == -1) break
      
      // 这里的“刹车”条件必须和上面的避让逻辑一致
      if (
        curr == 91 || // [
        curr == 45 || // -
        curr == 95 || // _
        curr == 42 || // *
        curr == 47 || // /
        curr == 43 || // +
        curr == 64 || // @
        curr == 123 || // {
        curr == 10 || curr == 13 // 换行
      ) break
      len++
    }
    
    if (len > 0) {
      input.acceptToken(Text, len)
      return
    }
  }

  // 6. 如果是孤立的单符号（比如单个 [ 或 单个 *），由这里认领
  input.acceptToken(Text, 1)
})