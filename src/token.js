import { ExternalTokenizer } from "@lezer/lr"
import * as Terms from "./parser.terms.js"

const { UnderlineText, StrikeText, Text } = Terms

const _ = 95, 
      dash = 45, 
      newline = 10, 
      carriageReturn = 13,
      bracketOpen = 91,
      star = 42;

export const inlineTokenizer = new ExternalTokenizer((input, stack) => {
  // 注意：不要在一开始就 advance
  let next = input.next;

  // 1. 下划线逻辑: __...__
  if (next == _) {
    // 探测第二个字符，但不提交
    if (input.peek(1) == _) {
      let offset = 2;
      let hasContent = false;
      
      // 使用 peek 来向后观察，不移动指针
      while (true) {
        let curr = input.peek(offset);
        if (curr == -1 || curr == newline || curr == carriageReturn) break;
        
        if (curr == _) {
          if (input.peek(offset + 1) == _) {
            if (hasContent) {
              // 只有确定匹配成功了，才一口气 advance 到底并接受
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
    // 如果没匹配成功，这里什么都不写，直接进入后面的逻辑
  }

  // 2. 删除线逻辑: --...--
  if (next == dash) {
    if (input.peek(1) == dash && input.peek(2) != dash) {
      let offset = 2;
      let hasContent = false;
      while (true) {
        let curr = input.peek(offset);
        if (curr == -1 || curr == newline || curr == carriageReturn) break;
        if (curr == dash) {
          if (input.peek(offset + 1) == dash) {
            if (hasContent) {
              for (let i = 0; i < offset + 2; i++) input.advance();
              input.acceptToken(StrikeText);
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

  // 3. Text 兜底逻辑
  if (next != -1) {
    // Text 需要吞噬字符，所以它用 advance
    while (next != -1 && next != newline && next != carriageReturn) {
      // 遇到可能是其他标记开头的，停下
      if (next == _ || next == dash || next == bracketOpen || next == star) break;
      input.advance();
      next = input.next;
    }
    
    // 只要探头动了，就说明有普通文本
    if (input.pos > stack.pos) {
      input.acceptToken(Text);
    }
  }
});