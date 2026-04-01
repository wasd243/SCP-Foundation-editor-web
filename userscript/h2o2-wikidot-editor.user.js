// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      1.3.1
// @description  解决竞态条件、取消后重进编辑器、以及 Wikidot AJAX 幽灵 DOM 问题
// @author       wasd243
// @match        *://*.wikidot.com/*
// @grant        none
// @run-at       document-start
// ==/UserScript==

(function () {
    'use strict';

    var EDITOR_URL = 'https://wasd243.github.io/SCP-Foundation-editor-web/test.html';
    var IFRAME_ID = 'h2o2-editor-frame';
    var EDITOR_ORIGIN = new URL(EDITOR_URL).origin;

    var observer = null;
    var messageListenerBound = false;

    function getNativeTextarea() {
        return document.getElementById('edit-textarea') || document.getElementById('edit-page-textarea');
    }

    function isLikelyEditPage() {
        return /\/edit\/true\/?$/.test(location.pathname) || !!getNativeTextarea();
    }

    function ensureHostHidden(nativeTextarea) {
        nativeTextarea.style.display = 'none';

        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.display = 'none';

        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.display = 'none';
        });
    }

    function buildIframeUrl() {
        var url = new URL(EDITOR_URL);
        url.searchParams.set('parentOrigin', window.location.origin);
        return url.toString();
    }

    function pushInitContent(iframe, nativeTextarea) {
        if (!iframe || !iframe.contentWindow || !nativeTextarea) return;
        iframe.contentWindow.postMessage({
            type: 'h2o2-init',
            payload: nativeTextarea.value || ''
        }, EDITOR_ORIGIN);
    }

    function bindMessageListener() {
        if (messageListenerBound) return;
        messageListenerBound = true;

        window.addEventListener('message', function (event) {
            if (event.origin !== EDITOR_ORIGIN) return;
            if (!event.data || event.data.type !== 'h2o2-update') return;

            var nativeTextarea = getNativeTextarea();
            if (!nativeTextarea) return;
            nativeTextarea.value = typeof event.data.payload === 'string' ? event.data.payload : '';
        });
    }

    function mountOrRemountEditor() {
        if (!isLikelyEditPage()) return;

        var nativeTextarea = getNativeTextarea();
        if (!nativeTextarea || !nativeTextarea.parentNode) return;

        ensureHostHidden(nativeTextarea);
        bindMessageListener();

        var iframe = document.getElementById(IFRAME_ID);
        if (!iframe) {
            iframe = document.createElement('iframe');
            iframe.id = IFRAME_ID;
            iframe.src = buildIframeUrl();
            iframe.style.cssText = [
                'width: 100%',
                'height: 70vh',
                'min-height: 520px',
                'border: 1px solid #ccc',
                'border-radius: 4px',
                'display: block',
                'margin-bottom: 8px'
            ].join(';');

            nativeTextarea.parentNode.insertBefore(iframe, nativeTextarea);
            iframe.addEventListener('load', function () {
                // 避免编辑器内部尚未完成初始化导致丢失首包
                setTimeout(function () {
                    pushInitContent(iframe, getNativeTextarea());
                }, 350);
            });
        } else {
            iframe.style.display = 'block';

            // 如果 Wikidot 动态替换了 textarea，确保 iframe 仍与当前 textarea 同级
            if (iframe.parentNode !== nativeTextarea.parentNode) {
                nativeTextarea.parentNode.insertBefore(iframe, nativeTextarea);
            }

            // 用户点“取消”再点“编辑”时通常会复用旧 iframe，这里主动再同步一次初始内容
            pushInitContent(iframe, nativeTextarea);
        }
    }

    function boot() {
        if (document.readyState === 'loading') return;
        mountOrRemountEditor();
    }

    document.addEventListener('DOMContentLoaded', mountOrRemountEditor);
    window.addEventListener('load', mountOrRemountEditor);
    window.addEventListener('pageshow', mountOrRemountEditor);

    // Wikidot 可能无整页刷新；用 mutation 持续兜底，不主动断开
    observer = new MutationObserver(function () {
        mountOrRemountEditor();
    });

    observer.observe(document.documentElement, { childList: true, subtree: true });

    // 额外定时兜底：覆盖慢加载和异步渲染
    setTimeout(mountOrRemountEditor, 300);
    setTimeout(mountOrRemountEditor, 1200);
    setTimeout(mountOrRemountEditor, 2500);

    boot();
})();
