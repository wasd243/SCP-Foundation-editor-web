// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      1.1.0
// @description  等待内容填充后再注入
// @author       wasd243
// @match        *://*.wikidot.com/*
// @grant        none
// @run-at       document-end
// ==/UserScript==

(function () {
    'use strict';
    // 这里填入引用网页的url
    var EDITOR_URL = 'https://wasd243.github.io/SCP-Foundation-editor-web/test.html';
    var IFRAME_ID = 'h2o2-editor-frame';

    // ── 消息同步 ──────────────────────────────────────────────
    window.addEventListener('message', function (event) {
        if (!event.data || event.data.type !== 'h2o2-update') return;
        var ta = document.getElementById('edit-page-textarea');
        if (!ta) return;
        ta.value = event.data.payload;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
    });

    // ── 注入 ──────────────────────────────────────────────────
    function inject(ta) {
        // 已经注入过了
        if (document.getElementById(IFRAME_ID)) return;

        // 等待 Wikidot 填充内容（最多等 3 秒，每 100ms 检查一次）
        var waited = 0;
        var poller = setInterval(function () {
            waited += 100;
            var content = ta.value;

            // 内容已填充，或者等够 3 秒强制注入
            if (content.length > 0 || waited >= 3000) {
                clearInterval(poller);
                doInject(ta, content);
            }
        }, 100);
    }

    function doInject(ta, content) {
        if (document.getElementById(IFRAME_ID)) return;
        console.log('H2O2: 注入，内容长度', content.length);

        ta.style.setProperty('display', 'none', 'important');

        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.setProperty('display', 'none', 'important');
        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.setProperty('display', 'none', 'important');
        });

        var iframe = document.createElement('iframe');
        iframe.id = IFRAME_ID;
        iframe.src = EDITOR_URL;
        iframe.style.cssText = 'width:100%;height:70vh;min-height:520px;border:none;border-radius:4px;display:block;margin-bottom:8px;';
        ta.parentNode.insertBefore(iframe, ta);

        iframe.addEventListener('load', function () {
            setTimeout(function () {
                try {
                    iframe.contentWindow.postMessage(
                        { type: 'h2o2-init', payload: content }, '*'
                    );
                } catch (e) {
                    console.warn('H2O2: postMessage 失败', e);
                }
            }, 500);
        });
    }

    // ── 监听 textarea 出现 ────────────────────────────────────
    var observer = new MutationObserver(function () {
        var ta = document.getElementById('edit-page-textarea');
        var iframe = document.getElementById(IFRAME_ID);

        // textarea 消失：清理 iframe
        if (!ta && iframe) {
            console.log('H2O2: 编辑结束，清理');
            iframe.remove();
            return;
        }

        // textarea 出现且还没注入
        if (ta && !iframe) {
            inject(ta);
        }

        // textarea 存在：持续压制
        if (ta) {
            ta.style.setProperty('display', 'none', 'important');
        }
    });

    observer.observe(document.documentElement, {
        childList: true,
        subtree: true
    });

    // 页面加载时检查一次（处理直接进入编辑页的情况）
    var ta = document.getElementById('edit-page-textarea');
    if (ta) inject(ta);

    console.log('H2O2: v1.1.0 已启动');
})();