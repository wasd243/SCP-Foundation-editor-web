// ==UserScript==
// @name         H2O2 Wikidot Editor
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      1.2.0
// @description  强制进化！用 CodeMirror 6 接管 Wikidot 原生编辑器
// @author       wasd243
// @match        *://*.wikidot.com/*
// @grant        none
// @run-at       document-idle
// ==/UserScript==

(function () {
    'use strict';

    // 编辑器部署地址：请改成你自己的 GitHub Pages 地址
    // 示例：https://wasd243.github.io/SCP-Foundation-editor-web/
    var EDITOR_URL = 'https://wasd243.github.io/SCP-Foundation-editor-web/';

    var mounted = false;
    var editorOrigin = new URL(EDITOR_URL).origin;
    var observer = null;

    function isLikelyEditPage() {
        if (/\/edit\/true\/?$/.test(location.pathname)) return true;
        return !!document.getElementById('edit-page-textarea');
    }

    function mountEditor() {
        if (mounted || !isLikelyEditPage()) return;
        var nativeTextarea = document.getElementById('edit-page-textarea');
        if (!nativeTextarea || !nativeTextarea.parentNode) return;

        if (document.getElementById('h2o2-editor-frame')) {
            mounted = true;
            return;
        }

        mounted = true;

        // --- 1. 隐藏原生编辑器相关元素 ---
        nativeTextarea.style.display = 'none';
        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.display = 'none';

        // 隐藏调整大小的按钮，但只针对编辑器区域
        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.display = 'none';
        });

        // --- 2. 创建并注入 iframe ---
        var iframe = document.createElement('iframe');
        iframe.id = 'h2o2-editor-frame';
        var iframeUrl = new URL(EDITOR_URL);
        iframeUrl.searchParams.set('parentOrigin', window.location.origin);
        iframe.src = iframeUrl.toString();
        iframe.style.cssText = [
            'width: 100%',
            'height: 70vh',
            'min-height: 520px',
            'border: 1px solid #ccc',
            'border-radius: 4px',
            'display: block',
            'margin-bottom: 8px'
        ].join(';');

        // 插入到 textarea 的前面，保持 DOM 结构
        nativeTextarea.parentNode.insertBefore(iframe, nativeTextarea);

        // --- 3. iframe 加载完成后，把现有内容推送进去 ---
        iframe.addEventListener('load', function () {
            iframe.contentWindow.postMessage({
                type: 'h2o2-init',
                payload: nativeTextarea.value
            }, editorOrigin);
        });

        // --- 4. 接收编辑器广播，同步到原生 textarea ---
        window.addEventListener('message', function (event) {
            if (event.origin !== editorOrigin) return;
            if (!event.data || event.data.type !== 'h2o2-update') return;
            nativeTextarea.value = event.data.payload;
        });
    }

    function boot() {
        mountEditor();
        if (mounted && observer) {
            observer.disconnect();
            observer = null;
        }
    }

    // 首次尝试
    boot();

    // Wikidot 页面异步渲染时兜底
    if (!mounted) {
        observer = new MutationObserver(boot);
        observer.observe(document.documentElement, { childList: true, subtree: true });
        setTimeout(boot, 1000);
        setTimeout(boot, 2500);
    }

    // --- 5. Wikidot 保存/预览按钮无需额外劫持 ---
    // 因为每次编辑器内容变化都已同步到 nativeTextarea，
    // Wikidot 原生的保存/预览按钮读取 textarea.value 时内容已是最新的。
})();
