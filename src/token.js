import { ExternalTokenizer } from "@lezer/lr"
import * as Terms from "./parser.terms.js"

// 引入 Hr
const { UnderlineText, StrikeText, Hr } = Terms

const _ = 95, 
      dash = 45, 
      newline = 10, 
      carriageReturn = 13;

export const inlineTokenizer = new ExternalTokenizer((input, stack) => {
  let next = input.next;

  // 1. 下划线逻辑: __...__
  if (next == _) {
    if (input.peek(1) == _) {
      let offset = 2;
      let hasContent = false;
      while (true) {
        let curr = input.peek(offset);
        if (curr == -1 || curr == newline || curr == carriageReturn) break;
        if (curr == _) {
          if (input.peek(offset + 1) == _) {
            if (hasContent) {
              for (let i = 0; i < offset + 2; i++) input.advance();
              input.acceptToken(UnderlineText);
              return;
            }
            break; 
          }
        }
        hasContent = true;
        offset++;
      }
    }
  }

  // 2. 连字号逻辑: 处理 Hr (分割线) 和 StrikeText (删除线)
  if (next == dash) {
    let count = 0;
    // 先数一数有多少个连续的 -
    while (input.peek(count) == dash) {
      count++;
    }

    // --- 情况 A: 水平分割线 (4个或更多 -) ---
    if (count >= 4) {
      for (let i = 0; i < count; i++) input.advance();
      input.acceptToken(Hr);
      return;
    }

    // --- 情况 B: 删除线 (--内容--) ---
    if (count == 2) {
      let offset = 2;
      let hasContent = false;
      
      while (true) {
        let curr = input.peek(offset);
        if (curr == -1 || curr == newline || curr == carriageReturn) break;
        
        if (curr == dash) {
          if (input.peek(offset + 1) == dash) {
            // 确保结尾恰好是两个 -，排除 --- 这种干扰
            if (input.peek(offset + 2) != dash) {
              if (hasContent) {
                for (let i = 0; i < offset + 2; i++) input.advance();
                input.acceptToken(StrikeText);
                return;
              }
            }
            break;
          }
        }
        hasContent = true;
        offset++;
      }
    }
    // 如果是 1个 或 3个 -，或者没有正确闭合，直接跳过，交给 grammar 处理
  }
});