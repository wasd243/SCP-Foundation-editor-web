import {ExternalTokenizer} from "@lezer/lr"
// 注意：一定要引入 .terms.js，这里存的是 Token 的数字 ID
import {UnderlineText, StrikeText, Text} from "./parser.terms.js"

const _ = 95, 
      dash = 45, 
      newline = 10, 
      carriageReturn = 13,
      bracketOpen = 91, // [
      star = 42;       // *

export const inlineTokenizer = new ExternalTokenizer((input, stack) => {
  let {next} = input;

  // 1. 下划线逻辑: __...__
  if (next == _) {
    let start = input.pos;
    input.advance();
    if (input.next == _) {
      input.advance();
      let hasContent = false;
      while (input.next != -1 && input.next != newline && input.next != carriageReturn) {
        if (input.next == _) {
          input.advance();
          if (input.next == _) {
            input.advance();
            if (hasContent) {
              input.acceptToken(UnderlineText);
              return;
            }
            break; 
          }
        } else {
          hasContent = true;
          input.advance();
        }
      }
    }
    input.goto(start); // 匹配失败，探头瞬移回起点
    next = input.next; // 重置当前看到的字符
  }

  // 2. 删除线逻辑: --...--
  if (next == dash) {
    let start = input.pos;
    input.advance();
    if (input.next == dash) {
      input.advance();
      // 排除 ---- (Hr)
      if (input.next == dash) {
        input.goto(start);
      } else {
        let hasContent = false;
        while (input.next != -1 && input.next != newline && input.next != carriageReturn) {
          if (input.next == dash) {
            input.advance();
            if (input.next == dash) {
              input.advance();
              if (hasContent) {
                input.acceptToken(StrikeText);
                return;
              }
              break;
            }
          } else {
            hasContent = true;
            input.advance();
          }
        }
        input.goto(start);
      }
    } else {
      input.goto(start);
    }
    next = input.next;
  }

  // 3. Text 兜底逻辑 (吞噬一切普通文本)
  if (next != -1) {
    while (next != -1 && next != newline && next != carriageReturn) {
      // 只要看到可能是标记开头的符号，立刻停下，把控制权还给解析器
      if (next == _ || next == dash || next == bracketOpen || next == star) {
        break;
      }
      input.advance();
      next = input.next;
    }
    
    // 如果探头移动过了，说明有内容，接受为 Text
    if (input.pos > stack.pos) {
      input.acceptToken(Text);
    }
  }
});