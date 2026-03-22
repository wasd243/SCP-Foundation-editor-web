/**
 * @file editor.js
 * @description SCP Foundation Wikidot Editor - Core Module
 * * --- LICENSE INFORMATION ---
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 * * --- ATTRIBUTION & THIRD-PARTY COMPONENTS ---
 * 1. Editor Engine: CodeMirror 6 (MIT License) - © Marijn Haverbeke
 * 2. Syntax Patterns: ACS & AIM logic patterns are derived from the 
 * SCP Foundation community under CC BY-SA 3.0.
 * -------------------------------------------------------------------
 */
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { StreamLanguage, syntaxHighlighting, HighlightStyle } from "@codemirror/language";
import { oneDark } from "@codemirror/theme-one-dark";
import { autocompletion } from "@codemirror/autocomplete";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { Tag } from "@lezer/highlight";
// 导入颜色预览扩展和事件处理函数
import { colorPreviewExtension, setupColorPickerHandler } from "./color_preview.js";
import { wikidotColorExtension } from "./color_widgets.js";

// 初始化内容
const EXAMPLE_CODE = `[[include :scp-wiki-cn:theme:peroxide]]

[[div class="xxx"]]
++ **SCP-CN-WEB-EDITOR**
//测试版 v1.0.0 //
[[/div]]

+ 点几下试试？这里是可以编辑的，顺便说一句，这不是Google docs，你编辑的内容别人应该看不到的（当然啊我不保证100%没有低克入侵）

------

> **编辑器功能演示：**
**高亮测试**：###ff4d4d|这行文本应该是红色的##。
> 但是在引用里面会默认为绿色
> * **高亮测试**：###ff4d4d|这行文本应该是红色的##。
> * **代码块**：{{monospace}}。
> * **折叠块**：
[[collapsible show="+ 展开技术细节" hide="- 隐藏内容"]]
当前编辑器基于 **CodeMirror 6** 核心，支持：
1. **自动补全**：尝试在下方输入 \`[[\` 或 \`@@\`。
2. **快捷工具栏**：点击上方 **CODE TOOLS** 标签试试看！
3. **安全初始化**：初始化不如CLEAR一根，在这里只是为了展示说明
[[/collapsible]]

-------

||~ 语法项目 ||~ 高亮状态 ||
|| 加粗 || **完成** ||
|| 斜体 || //完成// ||
|| 下划线 || __完成__ ||
|| 删除线 || --完成-- ||

[[footnote]]xxx[[/footnote]]

[[footnoteblock]]

@@@@
[[div class="footer"]]
[[/div]]
[[/div]]

[[module CSS]]
/* 未来将支持 CSS 高亮补全 */
.container {
    border: 1px solid #fff;
}
[[/module]]`

// 1. 定义自定义高亮标签，防止 "Unknown highlighting tag" 报错
const customTags = {
    header: Tag.define(),
    strong: Tag.define(),
    em: Tag.define(),
    underline: Tag.define(),
    strikethrough: Tag.define(),
    wikiTag: Tag.define(),
    link: Tag.define(),
    hr: Tag.define(),
    rate: Tag.define(),
    right: Tag.define(),
    left: Tag.define(),
    center: Tag.define(),
    sup: Tag.define(),
    sub: Tag.define(),
    newline: Tag.define(),
    monosapace: Tag.define(),
    list1: Tag.define(),
    list2: Tag.define(),
    list3: Tag.define(),
    list4: Tag.define(),
    quote: Tag.define(),
    table: Tag.define(),
    table_header: Tag.define(),
    table_cell: Tag.define(),
    original_text: Tag.define(), // 用于原始文本
    image: Tag.define(), // 用于图片
    footnote: Tag.define(),
    footnote_block: Tag.define(), // 用于脚注块
    color: Tag.define(), 
    include: Tag.define(), // 用于 [[include ...]] 标签
    scp_wiki: Tag.define(), // 用于特定的主题标签
    div: Tag.define(), // 用于 [[div ...]] 标签
    tabview: Tag.define(), // 用于 [[tabview]] 标签
    tab: Tag.define(), // tabview增强
    acs: Tag.define(), // 用于ACS
    components: Tag.define(), // Components
    equal: Tag.define(), // 用于 = 号
    line_up: Tag.define(), // 用于|
    size: Tag.define(), // 用于字体大小标签
    aim: Tag.define(), // 用于AIM
    collapsible: Tag.define(), // 用于可折叠内容
    license: Tag.define(), // LICENSE
    note: Tag.define(), // note
    user: Tag.define(), // user
    Highlight: Tag.define(), // CSS and html temporary highlight
};

/**
 * 自定义 Wikidot 语法解析器
 */
const wikidotLanguage = StreamLanguage.define({
    // ====== 新增：1. 初始化状态 ======
    startState() {
        return { inCSS: false, inHTML: false };
    },
    // ====== 新增：2. 状态拷贝（处理撤销/重做必须） ======
    copyState(state) {
        return { inCSS: state.inCSS, inHTML: state.inHTML };
    },
    token(stream, state) {
        // ====== 新增：3. 范围判定与拦截 ======

        // ================================================================
        
        // 检测进入特定模块
        if (!state.inCSS && !state.inHTML) {
            if (stream.match(/\[\[module css\]\]/i)) {
                state.inCSS = true;
                return "wikiTag"; 
            }
            if (stream.match(/\[\[html\]\]/i)) {
                state.inHTML = true;
                return "wikiTag";
            }
        }

        // CSS 模块内部解析逻辑（完美复用你现有的Tag）
        if (state.inCSS) {
            if (stream.match(/\[\[\/module\]\]/i)) {
                state.inCSS = false;
                return "wikiTag";
            }
            if (stream.match(/[\{\}\:\;]/)) return "equal"; // 符号复用等号颜色
            if (stream.match(/\.[a-zA-Z0-9_-]+/)) return "components"; // 类名
            if (stream.match(/#[a-zA-Z0-9_-]+/)) return "Highlight"; // #ID 或 颜色代码
            if (stream.match(/[a-zA-Z-]+(?=\s*:)/)) return "aim"; // 属性名 (如 color:)
            stream.next();
            return "original_text";
        }

        // HTML 模块内部解析逻辑（完美复用你现有的Tag）
        if (state.inHTML) {
            if (stream.match(/\[\[\/html\]\]/i)) {
                state.inHTML = false;
                return "wikiTag";
            }
            if (stream.match(/<\/?[a-zA-Z0-9-]+\s?>?/)) {
                return "components"; 
            }

            // 如果上面没匹配到，单独匹配剩下的尖括号或等号
            if (stream.match(/[\<\>\=\/]/)) {
                return "equal"; // 借用定义的等号颜色 (#d19a66 橙色) 
            }
            if (stream.match(/[a-zA-Z-]+(?=\=)/)) return "aim"; // 属性名
            if (stream.match(/".*?"/)) return "Highlight"; // 引号内字符串
            stream.next();
            return "original_text";
        }

        // ================================================================

        // 标题
        if (stream.sol() && stream.match(/\++ /)) {
            stream.skipToEnd();
            return "header";
        }

        // 改进的格式匹配，支持基本嵌套
        // 加粗：允许内部包含单个*字符
        if (stream.match(/\*\*(?:[^*]|\*[^*])*\*\*/)) return "strong";
        
        // 斜体：允许内部包含单个/字符
        if (stream.match(/\/\/(?:[^/]|\/[^/])*\/\//)) return "em";
        
        // 下划线：允许内部包含单个_字符
        if (stream.match(/__(?:[^_]|_[^_])*__/)) return "underline";
        
        // 删除线：允许内部包含单个-字符
        if (stream.match(/--(?:[^-]|-[^-])*--/)) return "strikethrough";
        
        // 上标：允许内部包含单个^字符
        if (stream.match(/\^\^(?:[^\^]|\^[^\^])*\^\^/)) return "sup";
        
        // 下标：允许内部包含单个,字符
        if (stream.match(/,,(?:[^,]|,[^,])*,,/)) return "sub";

        // 链接
        if (stream.match(/\[https?:\/\/.*?\]/)) return "link";

        // 强制换行符
        if (stream.match(/\@\@\@\@/)) return "newline";

        // 英文等宽字体
        if (stream.match(/\{\{.*?\}\}/)) return "monosapace";

        // size 标签
        if (stream.match(/\[\[size.*?\]\]/) || stream.match(/\[\[\/size\]\]/)) return "size";

        // user 标签
        if (stream.match(/\[\[user .*?\]\]/i) || stream.match(/\[\[\*user .*?\]\]/i)) return "user";

        // 在wikidotLanguage的token函数中修改颜色匹配部分：
        // Wikidot颜色标签：###ffffff|文字##
        if (stream.match(/###([0-9a-fA-F]{6})\|/)) {
            // 匹配了 ###ffffff| 部分
            // 现在查找文字内容直到 ##（注意是2个#，不是3个）
            let content = "";
            while (!stream.eol()) {
                // 检查是否遇到 ##
                if (stream.peek() === "#") {
                    stream.next(); // 跳过第一个#
                    if (stream.peek() === "#") {
                        stream.next(); // 跳过第二个#
                        // 找到了结束标记 ##
                        return "color";
                    }
                } else {
                    content += stream.next();
                }
            }
            // 如果没有找到结束标记，返回null
            return null;
        }

        // 16进制颜色代码（用于普通颜色预览）
        if (stream.match(/#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b/)) {
            return "color";
        }

        // 原始文本
        if (stream.match(/\@\@.*?\@\@/)) {
            stream.skipToEnd();
            return "original_text";
        }

        // 无序列表
        if (stream.sol() && stream.match(/\*+ /)) {
            return "list1";
        }

        // 有序列表
        if (stream.sol() && stream.match(/#+ /)) {
            return "list2";
        }

        // 定义列表
        if (stream.sol() && stream.match(/\:.*?\:/)) {
            return "list3";
        }

        // 高级列表
        if (stream.match(/\[\[ul\]\]/) || stream.match(/\[\[li\]\]/) || stream.match(/\[\[ol\]\]/)){
            return "list4";
        }

        // 引用
        if (stream.sol() && stream.match(/>+ /)) {
            stream.skipToEnd();
            return "quote";
        }

        // = 号
        if (stream.match(/=/)) {
            return "equal";
        }

        // 表格
        // ================================================================
        if (stream.match(/\|\|/)) {
            // 高亮竖线
            return "table";
        }
        if (stream.match(/\~/)) {
            return "table_header";
        }
        // 匹配普通表格行
        if (stream.sol() && stream.match(/\|\|(?!~)/)) {
            stream.skipToEnd();
            return "table_cell";
        }
        // ================================================================

        // | 竖线
        if (stream.match(/\|/)) {
            return "line_up";
        }

        // 带有[[]]的注意需要放在标签前面，因为它也是以 [[ 开头的
        // ================================================================
        // 评分
        if (stream.match(/\[\[module rate\]\]/i)) return "rate";

        // 右对齐
        if (stream.match(/\[\[\>?\]\]/)) return "right";
        if (stream.match(/\[\[\/\>?\]\]/)) return "right";

        // 左对齐
        if (stream.match(/\[\[\<?\]\]/)) return "left";
        if (stream.match(/\[\[\/\<?\]\]/)) return "left";

        // 居中
        if (stream.match(/\[\[\=?\]\]/)) return "center";
        if (stream.match(/\[\[\/\=?\]\]/)) return "center";

        // 图片
        if (stream.match(/\[\[.*?image.*?\]\]/i) || stream.match(/include component:image-block/i)) return "image";

        // 脚注
        if (stream.match(/\[\[footnote\]\]/) || stream.match(/\[\[\/footnote\]\]/)) return "footnote";

        // 脚注块
        if (stream.match(/\[\[footnoteblock\]\]/i)) return "footnote_block";

        // div
        if (stream.match(/\[\[div.*?\]\]/i) || stream.match(/\[\[\/div\]\]/i)) {
            return "div";
        }

        // collapsible
        if (stream.match(/\[\[collapsible.*?\]\]/i) || stream.match(/\[\[\/collapsible\]\]/i)) {
            return "collapsible";
        }

        // LICENSE
        if (stream.match(/include :scp-wiki-cn:component:license-box/i) || stream.match(/include :scp-wiki-cn:component:license-end/i)) {
            return "license";
        }

        // note
        if (stream.match(/\[\[note\]\]/i) || stream.match(/\[\[\/note\]\]/i)) {
            return "note";
        }

        // ACS AIM
        // ================================================================
        if (stream.match(/include :scp-wiki-cn:component:anomaly-class-bar-source *?/i)) {
            return "acs";
        }
        if (stream.match(/include :scp-wiki-cn:component:advanced-information-methodaology *?/i)){
            return "aim";
        }
        // ================================================================
        // Components
        // ================================================================
        if (stream.match(/lang.*?/)){
            return "components";
        }
        if (stream.match(/item-number.*?/)){
            return "components";
        }
        if (stream.match(/clearance.*?/)){
            return "components";
        }
        if (stream.match(/container-class.*?/)){
            return "components";
        }
        if (stream.match(/disruption.*?/)){
            return "components";
        }
        if (stream.match(/risk-class.*?/)){
            return "components";
        }
        if (stream.match(/lv/)){
            return "components";
        }
        if (stream.match(/cc/)){
            return "components";
        }
        if (stream.match(/dc/)){
            return "components";
        }
        if (stream.match(/site/)){
            return "components";
        }
        if (stream.match(/dir/)){
            return "components";
        }
        if (stream.match(/head/)){
            return "components";
        }
        if (stream.match(/mtf/)){
            return "components";
        }
        if (stream.match(/XXXX/)){
            return "components";
        }
        if (stream.match(/blocks/)){
            return "components";
        }
        if (stream.match(/author/)){
            return "components";
        }
        if (stream.match(/translator/)){
            return "components";
        }
        if (stream.match(/name/) || stream.match(/caption/) || stream.match(/width/) || stream.match(/height/) || stream.match(/align/)) {
            return "components";
        }
        // ================================================================
        // Tabview
        // ================================================================
        if (stream.match(/\[\[tabview.*?\]\]/) || stream.match(/\[\[\/tabview\]\]/)) {
            return "tabview";
        }
        if (stream.match(/\[\[tab.*?\]\]/) || stream.match(/\[\[\/tab\]\]/)) {
            return "tab";
        }
        // ================================================================
        if (stream.match(/\[\[include[^\]]*\]\]/)) {
            // 匹配整个 include 标签
            // 我们可以检查里面是否有分会
            const match = stream.current();
            if (match.includes(':scp-wiki')) {
                return "scp_wiki";  // 如果有分会，整个标签都高亮为 scp_wiki
            }
            return "include";  // 否则高亮为 include
        }
        // ================================================================
        // Wikidot 标签
        if (stream.match(/\]\]/) || stream.match(/\[\[/)) return "wikiTag";

        // 分割线
        if (stream.sol() && stream.match(/^-{6,}$/)) return "hr";

        stream.next();
        return null;
    },
    // 关键：建立字符串标记与 Tag 对象的映射
    tokenTable: {
        "header": customTags.header,
        "strong": customTags.strong,
        "em": customTags.em,
        "underline": customTags.underline,
        "strikethrough": customTags.strikethrough,
        "wikiTag": customTags.wikiTag,
        "sup": customTags.sup,
        "sub": customTags.sub,
        "link": customTags.link,
        "hr": customTags.hr,
        "rate": customTags.rate,
        "right": customTags.right,
        "left": customTags.left,
        "center": customTags.center,
        "newline": customTags.newline,
        "monosapace": customTags.monosapace,
        "list1": customTags.list1,
        "list2": customTags.list2,
        "list3": customTags.list3,
        "list4": customTags.list4,
        "quote": customTags.quote,
        "table": customTags.table,
        "table_header": customTags.table_header,
        "table_cell": customTags.table_cell,
        "original_text": customTags.original_text,
        "image": customTags.image,
        "footnote": customTags.footnote,
        "footnote_block": customTags.footnote_block,
        "color": customTags.color,
        "include": customTags.include,
        "scp_wiki": customTags.scp_wiki,
        "div": customTags.div,
        "tabview": customTags.tabview,
        "tab": customTags.tab,
        "acs": customTags.acs,
        "components": customTags.components,
        "equal": customTags.equal,
        "line_up": customTags.line_up,
        "size": customTags.size,
        "aim": customTags.aim,
        "collapsible": customTags.collapsible,
        "note": customTags.note,
        "user": customTags.user,
        "Highlight": customTags.Highlight,
    }
});

// 2. 创建高亮样式映射，将标签转换为具体的 CSS 类名
const wikidotHighlightStyle = HighlightStyle.define([
    { tag: customTags.header, class: "cm-header" },
    { tag: customTags.strong, class: "cm-strong" },
    { tag: customTags.em, class: "cm-em" },
    { tag: customTags.underline, class: "cm-underline" },
    { tag: customTags.strikethrough, class: "cm-strikethrough" },
    { tag: customTags.wikiTag, class: "cm-wikiTag" },
    { tag: customTags.link, class: "cm-link" },
    { tag: customTags.hr, class: "cm-hr" },
    { tag: customTags.rate, class: "cm-rate" },
    { tag: customTags.right, class: "cm-right" },
    { tag: customTags.left, class: "cm-left" },
    { tag: customTags.center, class: "cm-center" },
    { tag: customTags.sup, class: "cm-sup" },
    { tag: customTags.sub, class: "cm-sub" },
    { tag: customTags.newline, class: "cm-newline" },
    { tag: customTags.monosapace, class: "cm-monosapace" },
    { tag: customTags.list1, class: "cm-list1" },
    { tag: customTags.list2, class: "cm-list2" },
    { tag: customTags.list3, class: "cm-list3" },
    { tag: customTags.list4, class: "cm-list4" },
    { tag: customTags.quote, class: "cm-quote" },
    { tag: customTags.table, class: "cm-table" },
    { tag: customTags.table_header, class: "cm-table-header" },
    { tag: customTags.table_cell, class: "cm-table-cell" },
    { tag: customTags.original_text, class: "cm-original-text" },
    { tag: customTags.image, class: "cm-image" },
    { tag: customTags.footnote, class: "cm-footnote" },
    { tag: customTags.footnote_block, class: "cm-footnote-block" },
    { tag: customTags.color, class: "cm-color" },
    { tag: customTags.include, class: "cm-include" },
    { tag: customTags.scp_wiki, class: "cm-scp-wiki" },
    { tag: customTags.div, class: "cm-div" },
    { tag: customTags.tabview, class: "cm-tabview" },
    { tag: customTags.tab, class: "cm-tab" },
    { tag: customTags.acs, class: "cm-acs" },
    { tag: customTags.components, class: "cm-components" },
    { tag: customTags.equal, class: "cm-equal" },
    { tag: customTags.line_up, class: "cm-line-up" },
    { tag: customTags.size, class: "cm-size" },
    { tag: customTags.aim, class: "cm-aim" },
    { tag: customTags.collapsible, class: "cm-collapsible" },
    { tag: customTags.note, class: "cm-note" },
    { tag: customTags.user, class: "cm-user" },
    { tag: customTags.Highlight, class: "cm-Highlight"},
]);

/**
 * 改进的自动延续列表逻辑 - 针对Wikidot语法
 * 使用更高优先级确保覆盖默认行为
 */
const customKeymap = keymap.of([
    {
        key: "Enter",
        run: (view) => {
            const state = view.state;
            const selection = state.selection.main;
            
            if (!selection.empty) return false;
            
            const line = state.doc.lineAt(selection.head);
            const cursorPos = selection.head - line.from;
            
            // 检查光标是否在行末（或者接近行末）
            const isAtEndOfLine = cursorPos >= line.text.length - 1;
            
            if (!isAtEndOfLine) {
                // 如果光标不在行末，让默认行为处理（比如在行中间换行）
                return false;
            }
            
            // Wikidot列表语法：单个*表示无序列表，单个#表示有序列表
            // 匹配行首的 * 或 #，后面必须跟空格
            const listMatch = line.text.match(/^([*#])\s+/);
            const list3Match = line.text.match(/^(:.*?:)\s+/); // 定义列表匹配
            
            if (listMatch || list3Match) {
                const listMarker = listMatch ? listMatch[1] : list3Match[1]; // * 或 # 或 :
                
                // 检查是否在空列表项上按回车
                const contentAfterMarker = line.text.substring(listMarker.length + 1).trim();
                const isListItemEmpty = contentAfterMarker === '';
                
                if (isListItemEmpty) {
                    // 空列表项：删除当前行的列表标记
                    view.dispatch({
                        changes: { 
                            from: line.from, 
                            to: line.to, 
                            insert: "" 
                        },
                        selection: { anchor: line.from }
                    });
                    return true;
                } else {
                    // 非空列表项：插入新行并延续列表标记
                    const newLineContent = `\n${listMarker} `;
                    
                    view.dispatch({
                        changes: { 
                            from: selection.head, 
                            to: selection.head, 
                            insert: newLineContent 
                        },
                        selection: { anchor: selection.head + newLineContent.length }
                    });
                    return true;
                }
            }
            
            return false;
        }
    },
    indentWithTab
]);

/**
 * 自动补全配置
 */
import { wikidotCompletionSource } from "./completion.js";

// 3. 初始化编辑器
const startEditor = () => {
    const state = EditorState.create({
        doc: EXAMPLE_CODE,
        extensions: [
            // 将 customKeymap 放在 basicSetup 之前，确保优先级
            customKeymap,
            basicSetup,
            oneDark,
            wikidotLanguage,
            syntaxHighlighting(wikidotHighlightStyle),
            // 先添加Wikidot颜色标签扩展
            wikidotColorExtension,
            // 再添加普通颜色预览扩展
            colorPreviewExtension,
            autocompletion({ override: [wikidotCompletionSource], selectOnOpen: true }),
            
            // Web版本：添加本地存储自动保存功能
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    try {
                        // 保存到localStorage
                        const content = update.state.doc.toString();
                        localStorage.setItem('wikidot-editor-content', content);
                        
                        // 触发自定义事件，供其他组件使用
                        window.dispatchEvent(new CustomEvent('editorContentChanged', {
                            detail: { content }
                        }));
                    } catch (error) {
                        console.warn('无法保存到本地存储:', error);
                    }
                }
            }),

            EditorView.theme({
                "&": { height: "100%" },
                "&.cm-focused": { outline: "none" },
                ".cm-scroller": { 
                    fontFamily: "'Cascadia Code', 'Consolas', 'Monaco', 'Courier New', monospace",
                    lineHeight: "1.6",
                    fontSize: "14px"
                },
                // 为颜色预览添加一些基本样式
                ".cm-color-preview": {
                    display: "inline-block",
                    width: "12px",
                    height: "12px",
                    margin: "0 4px 0 0",
                    border: "1px solid #555",
                    borderRadius: "2px",
                    backgroundColor: "var(--color-value, #ccc)",
                    verticalAlign: "middle",
                    cursor: "pointer",
                    transition: "transform 0.2s ease, border-color 0.2s ease"
                },
                ".cm-color-preview:hover": {
                    transform: "scale(1.1)",
                    borderColor: "#888"
                },
                // Wikidot颜色文本样式
                ".cm-wikidot-colored-text": {
                    fontWeight: "normal!important",
                }
            })
        ]
    });

    const editorView = new EditorView({
        state,
        parent: document.getElementById("editor-container")
    });
    
    // 设置颜色选择器事件处理
    setupColorPickerHandler(editorView);
    
    // 尝试从localStorage加载之前的内容
    try {
        const savedContent = localStorage.getItem('wikidot-editor-content');
        if (savedContent) {
            // 延迟一点时间设置内容，确保编辑器完全初始化
            setTimeout(() => {
                editorView.dispatch({
                    changes: {
                        from: 0,
                        to: editorView.state.doc.length,
                        insert: savedContent
                    }
                });
            }, 100);
        }
    } catch (error) {
        console.warn('无法从本地存储加载内容:', error);
    }
    
    // 将实例挂载到全局，方便 index.html 的按钮调用
    window.editorInstance = editorView;

    return editorView;
};

// 导出编辑器实例供其他脚本使用
window.WikidotEditor = {
    startEditor,
    // 可以添加其他公共API
};

// 当DOM加载完成后启动编辑器
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', startEditor);
} else {
    startEditor();
}
