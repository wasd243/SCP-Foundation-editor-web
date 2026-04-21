// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      2.2.0
// @description  右侧面板模式，像 DevTools 一样从右侧挤入。包含左上角快捷保存/预览/取消按钮。
// @icon         https://scpsandboxcn.wikidot.com/local--files/peroxide-hyroperoxide/%E6%97%A0%E5%B0%BD%E5%82%AC%E5%8C%96%E5%89%82%EF%BC%88%E7%8E%84%E5%AD%A6%E4%BB%A3%E7%A0%81%E9%95%87%E5%9C%BA%E5%AD%90%EF%BC%89
// @author       wasd243
// @match        *://*.wikidot.com/*
// @license      AGPLv3
// @grant        none
// @run-at       document-end
// ==/UserScript==

(function () {
    'use strict';

    var EDITOR_URL   = 'https://wasd243.github.io/SCP-Foundation-editor-web/index.html';
    var IFRAME_ID    = 'h2o2-editor-frame';
    var PANEL_ID     = 'h2o2-panel';
    var TOGGLE_ID    = 'h2o2-toggle';
    var RESIZER_ID   = 'h2o2-resizer';
    var ACTIONS_ID   = 'h2o2-action-buttons'; // 新增：左上角按钮组ID
    var OBSERVER_KEY = '__h2o2_observer__';

    var PANEL_W      = 520;   // 初始面板宽度 px
    var PANEL_MIN_W  = 300;
    var PANEL_MAX_W  = Math.round(window.innerWidth * 0.85);

    // ── 消息同步：iframe → textarea ───────────────────────────
    window.addEventListener('message', function (event) {
        if (event.origin !== 'https://wasd243.github.io') return;
        if (!event.data || event.data.type !== 'h2o2-update') return;
        var ta = document.getElementById('edit-page-textarea');
        if (!ta) return;
        ta.value = event.data.payload;
        ta.dispatchEvent(new Event('input', { bubbles: true }));
    });

    // ── 面板开关 ──────────────────────────────────────────────
    function getPanelWidth() {
        var p = document.getElementById(PANEL_ID);
        return p ? p.offsetWidth : PANEL_W;
    }

    function openPanel() {
        var panel = document.getElementById(PANEL_ID);
        var toggle = document.getElementById(TOGGLE_ID);
        if (!panel) return;
        var w = getPanelWidth();
        panel.style.right = '0';
        document.body.style.marginRight = w + 'px';
        document.body.style.transition  = 'margin-right 0.25s ease';
        if (toggle) {
            toggle.style.right = w + 'px';
            toggle.title = '关闭编辑器面板';
            toggle.innerHTML = '&#x276F;'; // ❯
        }
        panel._open = true;
    }

    function closePanel() {
        var panel = document.getElementById(PANEL_ID);
        var toggle = document.getElementById(TOGGLE_ID);
        if (!panel) return;
        var w = getPanelWidth();
        panel.style.right = '-' + w + 'px';
        document.body.style.marginRight = '0';
        if (toggle) {
            toggle.style.right = '0';
            toggle.title = '打开编辑器面板';
            toggle.innerHTML = '&#x276E;'; // ❮
        }
        panel._open = false;
    }

    function togglePanel() {
        var panel = document.getElementById(PANEL_ID);
        if (!panel) return;
        panel._open ? closePanel() : openPanel();
    }

    // ── 左上角快捷按钮组 ──────────────────────────────────────
    function createActionButtons() {
        if (document.getElementById(ACTIONS_ID)) return;

        var container = document.createElement('div');
        container.id = ACTIONS_ID;
        container.style.cssText = [
            'position: fixed',
            'top: 15px',
            'left: 15px',
            'z-index: 999999',
            'display: flex',
            'gap: 8px',
            'background: rgba(255, 255, 255, 0.9)',
            'padding: 8px',
            'border-radius: 8px',
            'box-shadow: 0 4px 12px rgba(0,0,0,0.15)',
            'backdrop-filter: blur(4px)'
        ].join('; ');

        // 通用按钮生成函数
        function createBtn(text, color, targetId) {
            var btn = document.createElement('button');
            btn.innerText = text;
            btn.style.cssText = [
                'border: none',
                'border-radius: 4px',
                'padding: 8px 16px',
                'font-size: 14px',
                'font-weight: bold',
                'color: #fff',
                'background: ' + color,
                'cursor: pointer',
                'box-shadow: 0 2px 4px rgba(0,0,0,0.1)',
                'transition: opacity 0.2s, transform 0.1s'
            ].join('; ');

            btn.addEventListener('mouseenter', function() { btn.style.opacity = '0.85'; });
            btn.addEventListener('mouseleave', function() { btn.style.opacity = '1'; });
            btn.addEventListener('mousedown', function() { btn.style.transform = 'scale(0.95)'; });
            btn.addEventListener('mouseup', function() { btn.style.transform = 'scale(1)'; });

            btn.addEventListener('click', function(e) {
                e.preventDefault();
                var target = document.getElementById(targetId);
                if (target) {
                    target.click(); // 劫持并触发Wikidot原生按钮
                } else {
                    console.warn('H2O2: 找不到目标按钮', targetId);
                }
            });
            return btn;
        }

        // 颜色：红=取消(#ef5350)，灰=预览(#78909c)，绿=保存(#66bb6a)
        var btnCancel  = createBtn('取消', '#ef5350', 'edit-cancel-button');
        var btnPreview = createBtn('预览', '#78909c', 'edit-preview-button');
        var btnSave    = createBtn('保存', '#66bb6a', 'edit-save-button');

        container.appendChild(btnCancel);
        container.appendChild(btnPreview);
        container.appendChild(btnSave);

        document.body.appendChild(container);
    }

    // ── 注入逻辑 ──────────────────────────────────────────────
    function inject(ta) {
        if (document.getElementById(PANEL_ID)) return;

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
        if (document.getElementById(PANEL_ID)) return;
        console.log('H2O2: 注入，内容长度', content.length);

        // textarea 隐藏但保留在 DOM（表单提交需要）
        ta.style.setProperty('display', 'none', 'important');
        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.setProperty('display', 'none', 'important');
        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.setProperty('display', 'none', 'important');
        });

        // ── 注入左上角快捷按钮 ──
        createActionButtons();

        // ── 右侧固定面板 ─────────────────────────────────────
        var panel = document.createElement('div');
        panel.id = PANEL_ID;
        panel._open = false;
        panel.style.cssText = [
            'position: fixed',
            'top: 0',
            'right: -' + PANEL_W + 'px',   // 初始藏在右侧屏幕外
            'width: ' + PANEL_W + 'px',
            'height: 100vh',
            'z-index: 999999',
            'background: #fff',
            'box-shadow: -4px 0 16px rgba(0,0,0,0.18)',
            'display: flex',
            'flex-direction: row',         // 左边是 resizer，右边是 iframe
            'transition: right 0.25s ease',
            'box-sizing: border-box',
        ].join('; ');

        // ── 左侧宽度拖拽条 ────────────────────────────────────
        var resizer = document.createElement('div');
        resizer.id = RESIZER_ID;
        resizer.title = '拖拽调整面板宽度';
        resizer.style.cssText = [
            'width: 5px',
            'height: 100%',
            'cursor: ew-resize',
            'flex-shrink: 0',
            'background: #e0e0e0',
            'transition: background 0.15s',
        ].join('; ');
        resizer.addEventListener('mouseenter', function () {
            resizer.style.background = '#b0b0e8';
        });
        resizer.addEventListener('mouseleave', function () {
            resizer.style.background = '#e0e0e0';
        });

        // 拖拽 resizer → 改变面板宽度
        resizer.addEventListener('mousedown', function (e) {
            e.preventDefault();
            var startX = e.clientX;
            var startW = panel.offsetWidth;
            iframe.style.pointerEvents = 'none';

            function onMove(e) {
                var dx   = startX - e.clientX;         // 向左拖 = 变宽
                var newW = Math.min(PANEL_MAX_W, Math.max(PANEL_MIN_W, startW + dx));
                panel.style.width = newW + 'px';
                if (panel._open) {
                    document.body.style.transition = 'none';
                    document.body.style.marginRight = newW + 'px';
                    // 同步 toggle 按钮位置
                    var tog = document.getElementById(TOGGLE_ID);
                    if (tog) tog.style.right = newW + 'px';
                }
            }
            function onUp() {
                iframe.style.pointerEvents = 'auto';
                document.body.style.transition = 'margin-right 0.25s ease';
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup',   onUp);
            }
            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup',   onUp);
        });

        // ── iframe ────────────────────────────────────────────
        var iframe = document.createElement('iframe');
        iframe.id = IFRAME_ID;
        iframe.src = EDITOR_URL;
        iframe.style.cssText = [
            'flex: 1',
            'min-width: 0',
            'height: 100%',
            'border: none',
            'display: block',
        ].join('; ');

        panel.appendChild(resizer);
        panel.appendChild(iframe);
        document.body.appendChild(panel);

        // ── 开关按钮（竖排，固定在右侧边缘）─────────────────
        var toggle = document.createElement('div');
        toggle.id = TOGGLE_ID;
        toggle.innerHTML = '&#x276E;'; // ❮
        toggle.title = '打开编辑器面板';
        toggle.style.cssText = [
            'position: fixed',
            'right: 0',
            'top: 50%',
            'transform: translateY(-50%)',
            'z-index: 1000000',
            'width: 24px',
            'height: 64px',
            'background: #5c6bc0',
            'color: #fff',
            'display: flex',
            'align-items: center',
            'justify-content: center',
            'cursor: pointer',
            'border-radius: 6px 0 0 6px',
            'font-size: 16px',
            'box-shadow: -2px 0 8px rgba(0,0,0,0.2)',
            'user-select: none',
            'transition: right 0.25s ease, background 0.15s',
        ].join('; ');
        toggle.addEventListener('mouseenter', function () {
            toggle.style.background = '#3949ab';
        });
        toggle.addEventListener('mouseleave', function () {
            toggle.style.background = '#5c6bc0';
        });
        toggle.addEventListener('click', togglePanel);
        document.body.appendChild(toggle);

        // ── 推送内容到 iframe ─────────────────────────────────
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

        // 注入后自动打开面板
        setTimeout(openPanel, 50);
    }

    // ── 清理面板 ──────────────────────────────────────────────
    function cleanupPanel() {
        var panel   = document.getElementById(PANEL_ID);
        var toggle  = document.getElementById(TOGGLE_ID);
        var actions = document.getElementById(ACTIONS_ID); // 清理左上角按钮
        if (panel)   panel.remove();
        if (toggle)  toggle.remove();
        if (actions) actions.remove();
        document.body.style.marginRight = '';
        console.log('H2O2: 面板已清理');
    }

    // ── MutationObserver ──────────────────────────────────────
    function startObserver() {
        if (window[OBSERVER_KEY]) {
            try { window[OBSERVER_KEY].disconnect(); } catch (e) {}
        }
        var obs = new MutationObserver(function () {
            var ta     = document.getElementById('edit-page-textarea');
            var panel  = document.getElementById(PANEL_ID);

            if (!ta && panel) {
                console.log('H2O2: 编辑结束，清理面板');
                cleanupPanel();
                return;
            }
            if (ta && !panel) {
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

    // ── 看门狗 ────────────────────────────────────────────────
    setInterval(function () {
        if (!window[OBSERVER_KEY]) {
            console.warn('H2O2: Observer 丢失，重建中...');
            startObserver();
            return;
        }
        var ta    = document.getElementById('edit-page-textarea');
        var panel = document.getElementById(PANEL_ID);

        if (ta && !panel) {
            console.warn('H2O2: 发现未注入的 textarea，补注入');
            inject(ta);
        }
        if (ta) {
            ta.style.setProperty('display', 'none', 'important');
        }
        if (!ta && panel) {
            cleanupPanel();
        }
    }, 1000);

    // ── 启动 ──────────────────────────────────────────────────
    startObserver();
    var ta = document.getElementById('edit-page-textarea');
    if (ta) inject(ta);

    console.log('H2O2: v2.2.0 已启动');
})();
