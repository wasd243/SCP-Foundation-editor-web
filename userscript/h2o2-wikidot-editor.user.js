// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      2.0.3
// @description  可以自由修改窗口大小
// @icon         https://scpsandboxcn.wikidot.com/local--files/peroxide-hyroperoxide/%E6%97%A0%E5%B0%BD%E5%82%AC%E5%8C%96%E5%89%82%EF%BC%88%E7%8E%84%E5%AD%A6%E4%BB%A3%E7%A0%81%E9%95%87%E5%9C%BA%E5%AD%90%EF%BC%89
// @author       wasd243
// @match        *://*.wikidot.com/*
// @license      AGPLv3
// @grant        none
// @run-at       document-end
// ==/UserScript==

(function () {
    'use strict';

    var EDITOR_URL = 'https://wasd243.github.io/SCP-Foundation-editor-web/index.html';
    var IFRAME_ID = 'h2o2-editor-frame';
    var OBSERVER_KEY = '__h2o2_observer__';

    // ── 消息同步：iframe → textarea ───────────────────────────
    window.addEventListener('message', function (event) {
        if (event.origin !== 'https://wasd243.github.io') return;
        if (!event.data || event.data.type !== 'h2o2-update') return;
        var ta = document.getElementById('edit-page-textarea');
        if (!ta) return;
        ta.value = event.data.payload;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
    });

    // ── 注入逻辑 ──────────────────────────────────────────────
    function inject(ta) {
        if (document.getElementById(IFRAME_ID)) return;

        var waited = 0;
        var poller = setInterval(function () {
            waited += 100;
            var content = ta.value;
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

        // ── 外层可拖拽容器 ──────────────────────────────────────
        var wrapper = document.createElement('div');
        wrapper.id = 'h2o2-editor-wrapper';
        wrapper.style.cssText = [
            'width: 100%',
            'height: 70vh',
            'min-width: 300px',
            'min-height: 200px',
            'resize: both',// 允许横竖自由拖拽
            'overflow: hidden',// resize 生效必须加这个
            'display: block',
            'margin-bottom: 8px',
            'box-sizing: border-box',
            'border: 1px solid #ccc',
            'border-radius: 4px',
        ].join('; ');

        var iframe = document.createElement('iframe');
        iframe.id = IFRAME_ID;
        iframe.src = EDITOR_URL;
        iframe.style.cssText = [
            'width: 100%',
            'height: 100%',// 撑满外层容器
            'border: none',
            'display: block',
        ].join('; ');
        // ── 左下角手柄 ──────────────────────────────────────────
        var handleBL = document.createElement('div');
        handleBL.style.cssText = [
            'position: absolute',
            'left: 0',
            'bottom: 0',
            'width: 18px',
            'height: 18px',
            'cursor: nesw-resize',
            'background: linear-gradient(225deg, transparent 50%, #888 50%, #888 60%, transparent 60%, transparent 70%, #888 70%, #888 80%, transparent 80%)',
            'z-index: 9999',
            'border-radius: 0 0 0 4px',
        ].join('; ');

        // ── 左侧拖拽条 ───────────────────────────────────────────
        var handleLeft = document.createElement('div');
        handleLeft.style.cssText = [
            'position: absolute',
            'left: -4px',
            'top: 0',
            'width: 8px',
            'height: calc(100% - 18px)',
            'cursor: ew-resize',
            'z-index: 9998',
        ].join('; ');

        // ── 左侧专用拖拽逻辑（向左拖 = 宽度增加 + wrapper 左移）──
        function makeDraggableLeft(el) {
            el.addEventListener('mousedown', function (e) {
                e.preventDefault();
                var startX = e.clientX;
                var startW = wrapper.offsetWidth;
                // 记录初始 left 偏移（如果有的话）
                var startLeft = wrapper.offsetLeft;

                iframe.style.pointerEvents = 'none';

                function onMove(e) {
                    var dx = startX - e.clientX;// 向左为正
                    var newW = Math.max(300, startW + dx);
                    wrapper.style.width = newW + 'px';
                    // 同步往左偏移，保持右边界不动
                    wrapper.style.marginLeft = (startLeft - dx) + 'px';
                }
                function onUp() {
                    iframe.style.pointerEvents = 'auto';
                    document.removeEventListener('mousemove', onMove);
                    document.removeEventListener('mouseup', onUp);
                }
                document.addEventListener('mousemove', onMove);
                document.addEventListener('mouseup', onUp);
            });
        }

        // 左下角 = 同时改高度 + 左侧宽度
        handleBL.addEventListener('mousedown', function (e) {
            e.preventDefault();
            var startX = e.clientX;
            var startY = e.clientY;
            var startW = wrapper.offsetWidth;
            var startH = wrapper.offsetHeight;
            var startLeft = wrapper.offsetLeft;

            iframe.style.pointerEvents = 'none';

            function onMove(e) {
                var dx = startX - e.clientX;
                var newW = Math.max(300, startW + dx);
                var newH = Math.max(200, startH + (e.clientY - startY));
                wrapper.style.width = newW + 'px';
                wrapper.style.height = newH + 'px';
                wrapper.style.marginLeft = (startLeft - dx) + 'px';
            }
            function onUp() {
                iframe.style.pointerEvents = 'auto';
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
            }
            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        });

        makeDraggableLeft(handleLeft);

        wrapper.appendChild(iframe);
        wrapper.appendChild(handleBL); // 左下角（新增）
        wrapper.appendChild(handleLeft); // 左侧（新增）

        ta.parentNode.insertBefore(wrapper, ta);

        iframe.addEventListener('load', function () {
            setTimeout(function () {
                try {
                    iframe.contentWindow.postMessage(
                        { type: 'h2o2-init', payload: content }, '*'
                    );
                    console.log('H2O2: 内容已推送，长度', content.length);
                } catch (e) {
                    console.warn('H2O2: postMessage 失败', e);
                }
            }, 500);
        });
    }
    // ── 启动一个 MutationObserver 实例 ───────────────────────
    function startObserver() {
        // 断开旧的
        if (window[OBSERVER_KEY]) {
            try { window[OBSERVER_KEY].disconnect(); } catch (e) {}
        }

        var obs = new MutationObserver(function () {
            var ta = document.getElementById('edit-page-textarea');
            var iframe = document.getElementById(IFRAME_ID);

            if (!ta && iframe) {
                console.log('H2O2: 编辑结束，清理 iframe');
                iframe.remove();
                return;
            }
            if (ta && !iframe) {
                inject(ta);
            }
            if (ta) {
                ta.style.setProperty('display', 'none', 'important');
            }
        });

        obs.observe(document.documentElement, { childList: true, subtree: true });
        window[OBSERVER_KEY] = obs;
        console.log('H2O2: Observer 已(重)启动');
    }

    // ── 看门狗：每秒检查 Observer 是否还活着 ─────────────────
    // Warning: Wikidot AJAX is a monster. Do NOT remove this watchdog unless you want to lose your soul (and your drafts).
    // 原理：给 Observer 实例打一个存活标记，
    // 如果标记消失或 Observer 对象丢失，立刻重建。
    setInterval(function () {

        // 检查 Observer 是否还挂在 window 上
        if (!window[OBSERVER_KEY]) {
            console.warn('H2O2: Observer 丢失，重建中...');
            startObserver();
            return;
        }

        // 顺带检查：有 textarea 但没有 iframe，说明注入失败了，补注入
        var ta = document.getElementById('edit-page-textarea');
        var iframe = document.getElementById(IFRAME_ID);
        if (ta && !iframe) {
            console.warn('H2O2: 发现未注入的 textarea，补注入');
            inject(ta);
        }

        // 有 textarea 就持续压制
        if (ta) {
            ta.style.setProperty('display', 'none', 'important');
        }

        // 没有 textarea 就清理僵尸 iframe
        if (!ta && iframe) {
            iframe.remove();
            var wrapper = document.getElementById('h2o2-editor-wrapper');
            if (wrapper) wrapper.remove();
        }

    }, 1000);

    // ── 启动 ──────────────────────────────────────────────────
    startObserver();

    // 页面加载时立即检查一次
    var ta = document.getElementById('edit-page-textarea');
    if (ta) inject(ta);

    console.log('H2O2: v2.0.2 已启动');
})();
