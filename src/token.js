import { ExternalTokenizer } from "@lezer/lr"
import * as Terms from "./parser.terms.js"

// 注意：这里不再引入 Text
const { UnderlineText, StrikeText } = Terms

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
    // 如果不是合法的 __...__，什么都不做，交还给 grammar 处理（它会被识别为 Punctuation）
  }

  // 2. 删除线逻辑: --...--
  if (next == dash) {
    // 排除 ---- 分割线 (确保第三个字符不是 -)
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
});