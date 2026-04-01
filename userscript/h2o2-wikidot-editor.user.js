// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      1.3.0
// @description  解决竞态条件与 Wikidot AJAX 幽灵 DOM 问题
// @author       wasd243
// @match        *://*.wikidot.com/*
// @grant        none
// @run-at       document-start
// ==/UserScript==

(function () {
    'use strict';

    var EDITOR_URL = 'https://wasd243.github.io/SCP-Foundation-editor-web/test.html';

    function injectEditor() {
        var nativeTextarea = document.getElementById('edit-textarea') || document.getElementById('edit-page-textarea');

        // 如果页面上目前没有原生框，直接退出
        if (!nativeTextarea) return;

        // 【关键修改1】给具体的 textarea 打上专属标记，而不是去找全局的 iframe
        if (nativeTextarea.dataset.h2o2Injected === 'true') {
            // 如果点过取消再点编辑，确保我们的 iframe 也跟着显示出来
            var existingIframe = document.getElementById('h2o2-editor-frame');
            if (existingIframe && nativeTextarea.style.display !== 'none') {
                nativeTextarea.style.display = 'none';
                existingIframe.style.display = 'block';
            }
            return;
        }

        console.log("H2O2: 发现全新的原生编辑框，开始接管...");
        nativeTextarea.dataset.h2o2Injected = 'true'; // 打上专属烙印

        // 隐藏原生组件
        nativeTextarea.style.display = 'none';
        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.display = 'none';
        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.display = 'none';
        });

        // 斩草除根：如果有上一次遗留的幽灵 iframe，先干掉它
        var oldIframe = document.getElementById('h2o2-editor-frame');
        if (oldIframe) oldIframe.remove();

        // 创建新的 iframe
        var iframe = document.createElement('iframe');
        iframe.id = 'h2o2-editor-frame';
        iframe.src = EDITOR_URL;
        iframe.style.cssText = 'width: 100%; height: 70vh; min-height: 520px; border: 1px solid #ccc; border-radius: 4px; display: block; margin-bottom: 8px;';

        nativeTextarea.parentNode.insertBefore(iframe, nativeTextarea);

        // 【关键修改2】延迟发送数据，等 CodeMirror 完全启动
        iframe.addEventListener('load', function () {
            console.log("H2O2: iframe 骨架加载完毕，等待内部 JS 启动...");

            // 延迟 800 毫秒发送数据（如果还是没同步，可以把 800 改成 1500 试试）
            setTimeout(function() {
                console.log("H2O2: 向 iframe 发送沙盒原生内容!");
                iframe.contentWindow.postMessage({
                    type: 'h2o2-init',
                    payload: nativeTextarea.value
                }, '*');
            }, 800);
        });

        // 接收编辑器更新，同步回原生框
        window.addEventListener('message', function (event) {
            if (event.data && event.data.type === 'h2o2-update') {
                nativeTextarea.value = event.data.payload;
            }
        });
    }

    const observer = new MutationObserver(injectEditor);
    observer.observe(document.documentElement, { childList: true, subtree: true });
})();
