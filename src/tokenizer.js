export const myTokenizer = new ExternalTokenizer((input, stack) => {
  let {next} = input;
  if (next < 0) return;

  // 1. 处理横杠系列 (- / -- / ----)
  if (next == 45) { // '-'
    let count = 0;
    while (input.peek(count) == 45) {
      count++;
    }
    
    // 判定优先级：长者优先
    if (count >= 4) {
      input.acceptToken(Hr, count);
      return;
    }
    if (count >= 2) {
      // 这里的逻辑可以更细：Wikidot 的删除线通常不跨行
      input.acceptToken(StrikeText, 2); 
      return;
    }
    // 只是一个普通的连字符
    input.acceptToken(Dash, 1);
    return;
  }

  // 2. 处理下划线 (__ )
  if (next == 95) { // '_'
    if (input.peek(1) == 95) {
      input.acceptToken(UnderlineText, 2);
      return;
    }
  }

  // 3. 增强版 Text 逻辑 (吸尘器模式)
  // 核心思想：只要不是特殊功能的开关符，就一口气吞掉
  if (next != -1) {
    let len = 0;
    while (true) {
      let curr = input.peek(len);
      if (curr == -1) break; // EOF
      
      // 碰到这些符号，Text 必须立刻止步，让位给其他 Tokenizer
      if (curr == 91 || // '[' (标签开始)
          curr == 45 || // '-' (横杠系列)
          curr == 95 || // '_' (下划线)
          curr == 42 || // '*' (粗体)
          curr == 47 || // '/' (斜体)
          curr == 10 || // '\n'
          curr == 13    // '\r'
      ) break;
      
      len++;
    }
    
    if (len > 0) {
      input.acceptToken(Text, len);
      return;
    }
  }
  
  // 4. 换行
  if (next == 10 || next == 13) {
    input.acceptToken(newline, 1);
    return;
  }
});