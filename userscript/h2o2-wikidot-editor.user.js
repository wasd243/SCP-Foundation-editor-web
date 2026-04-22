// ==UserScript==
// @name         H2O2 Wikidot Editor (Universal)
// @namespace    https://github.com/wasd243/SCP-Foundation-editor-web
// @version      2.4.7
// @description  右侧面板模式，像 DevTools 一样从右侧挤入。包含左上角快捷保存/预览/取消/实时预览按钮，并禁用 CSS 动画。
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
    var ACTIONS_ID   = 'h2o2-action-buttons';
    var OBSERVER_KEY = '__h2o2_observer__';

    var PANEL_W      = 520;
    var PANEL_MIN_W  = 300;
    var PANEL_MAX_W  = window.innerWidth - 20;

    window.addEventListener('resize', function () {
        PANEL_MAX_W = window.innerWidth - 20;
    });

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
            toggle.innerHTML = '&#x276F;';
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
            toggle.innerHTML = '&#x276E;';
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

        function createBtn(text, color, action) {
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
                if (typeof action === 'function') {
                    action(e, btn);
                } else if (typeof action === 'string') {
                    var target = document.getElementById(action);
                    if (target) {
                        target.click();
                    } else {
                        console.warn('H2O2: 找不到目标按钮', action);
                    }
                }
            });
            return btn;
        }

        var btnCancel  = createBtn('取消', '#ef5350', 'edit-cancel-button');
        var btnPreview = createBtn('预览', '#78909c', 'edit-preview-button');
        var btnSave    = createBtn('保存', '#66bb6a', 'edit-save-button');

        // ── 实时预览逻辑 ──
        var isLivePreview = false;
        var liveTimer = null;

        function stopLivePreview() {
            isLivePreview = false;
            var btn = document.getElementById('h2o2-btn-live-preview');
            if (btn) {
                btn.innerText = '实时预览: 关';
                btn.style.background = '#ab47bc';
            }
            var styleEl = document.getElementById('h2o2-live-preview-style');
            if (styleEl) styleEl.remove();

            var s2 = document.createElement('script');
            s2.innerHTML = 'if (window._h2o2_orig_scrollTo) window.scrollTo = window._h2o2_orig_scrollTo;';
            document.head.appendChild(s2);
            setTimeout(function(){ s2.remove(); }, 100);

            var sb = document.getElementById('h2o2-scroll-block');
            if (sb) sb.remove();

            if (liveTimer) {
                clearInterval(liveTimer);
                liveTimer = null;
            }
        }
        window._h2o2_stopLivePreview = stopLivePreview;

        var btnLivePreview = createBtn('实时预览: 关', '#ab47bc', function(e, btn) {
            if (isLivePreview) {
                stopLivePreview();
            } else {
                isLivePreview = true;
                btn.innerText = '实时预览: 开';
                btn.style.background = '#8e24aa';

                // 1. 注入 CSS：屏蔽加载动画、锁屏掩罩、以及全局所有 CSS 动画和过渡效果
                var styleEl = document.createElement('style');
                styleEl.id = 'h2o2-live-preview-style';
                styleEl.innerHTML = `
                    /* 屏蔽 Wikidot 原生加载遮罩与提示 */
                    #lock-screen, #lock-info, #saving-message, #indicator, .owindow.wait, .ajax-loader {
                        display: none !important;
                        opacity: 0 !important;
                        pointer-events: none !important;
                        visibility: hidden !important;
                    }
                    /* 屏蔽页面所有版式自带的动画（如文字浮出、渐变、缩放等） */
                    * {
                        animation: none !important;
                        -webkit-animation: none !important;
                        transition: none !important;
                        -webkit-transition: none !important;
                    }
                `;
                document.head.appendChild(styleEl);

                // 2. 注入 JS 到页面环境：屏蔽滚动
                var s1 = document.createElement('script');
                s1.id = 'h2o2-scroll-block';
                s1.innerHTML = `
                    if (!window._h2o2_orig_scrollTo) window._h2o2_orig_scrollTo = window.scrollTo;
                    window.scrollTo = function(){};
                `;
                document.head.appendChild(s1);

                // 3. 开启定时点击（每 2.5 秒触发一次原生预览）
                liveTimer = setInterval(function() {
                    var target = document.getElementById('edit-preview-button');
                    if (target) target.click();
                }, 1000);
            }
        });
        btnLivePreview.id = 'h2o2-btn-live-preview';

        container.appendChild(btnCancel);
        container.appendChild(btnPreview);
        container.appendChild(btnSave);
        container.appendChild(btnLivePreview);

        document.body.appendChild(container);
    }

    // ── 全局样式注入：屏蔽弹窗 + 屏蔽动画 ───────────────────
    function injectGlobalStyles() {
        if (document.getElementById('h2o2-global-style')) return;

        // ── 1. 屏蔽原生 JS 弹窗 ──────────────────────────────
        // alert 直接吞掉；confirm 自动返回 true（相当于"确定"）；prompt 返回空字符串
        window.alert   = function (msg) { console.info('[H2O2] alert 已屏蔽:', msg); };
        window.confirm = function (msg) { console.info('[H2O2] confirm 已屏蔽，返回 true:', msg); return true; };
        window.prompt  = function (msg) { console.info('[H2O2] prompt 已屏蔽，返回空:', msg);  return ''; };

        // ── 2. 全局 CSS 屏蔽 Wikidot 自定义弹窗 + 全站动画 ──
        var style = document.createElement('style');
        style.id = 'h2o2-global-style';
        style.innerHTML = [
            /* Wikidot 各类弹窗/遮罩/加载层 */
            '#lock-screen, #lock-info, #saving-message,',
            '.owindow.wait, .ajax-loader, .modal-blocker,',
            'div.blocker, #ud-ui-dialog, .ui-dialog-overlay {',
            '    display: none !important;',
            '    opacity: 0 !important;',
            '    pointer-events: none !important;',
            '    visibility: hidden !important;',
            '}',

            /* 全站动画清零（h2o2 自身用 inline style，优先级更高，不受影响）*/
            'body *:not([id^="h2o2"]):not([id^="h2o2"] *) {',
            '    animation-duration:        0.001ms !important;',
            '    animation-delay:           0.001ms !important;',
            '    animation-iteration-count: 1       !important;',
            '    transition-duration:       0.001ms !important;',
            '    transition-delay:          0.001ms !important;',
            '}',
        ].join('\n');
        document.head.appendChild(style);

        // ── 3. MutationObserver 补漏：动态插入的弹窗也一并干掉 ──
        var dialogKiller = new MutationObserver(function (mutations) {
            mutations.forEach(function (m) {
                m.addedNodes.forEach(function (node) {
                    if (node.nodeType !== 1) return;
                    var id  = node.id  || '';
                    var cls = node.className || '';
                    var kill = (
                        id.indexOf('lock')   >= 0 ||
                        id.indexOf('saving') >= 0 ||
                        cls.indexOf('owindow') >= 0 ||
                        cls.indexOf('blocker') >= 0 ||
                        cls.indexOf('modal')   >= 0 ||
                        cls.indexOf('ajax-loader') >= 0
                    );
                    if (kill) {
                        node.style.setProperty('display',    'none',    'important');
                        node.style.setProperty('opacity',    '0',       'important');
                        node.style.setProperty('visibility', 'hidden',  'important');
                    }
                });
            });
        });
        dialogKiller.observe(document.documentElement, { childList: true, subtree: true });

        console.log('H2O2: 全局样式/弹窗屏蔽已启动');
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

        ta.style.setProperty('display', 'none', 'important');
        var toolbar = document.getElementById('wd-editor-toolbar-panel');
        if (toolbar) toolbar.style.setProperty('display', 'none', 'important');
        document.querySelectorAll('.change-textarea-size').forEach(function (el) {
            el.style.setProperty('display', 'none', 'important');
        });

        createActionButtons();

        var panel = document.createElement('div');
        panel.id = PANEL_ID;
        panel._open = false;
        panel.style.cssText = [
            'position: fixed',
            'top: 0',
            'right: -' + PANEL_W + 'px',
            'width: ' + PANEL_W + 'px',
            'height: 100vh',
            'z-index: 999999',
            'background: #fff',
            'box-shadow: -4px 0 16px rgba(0,0,0,0.18)',
            'display: flex',
            'flex-direction: row',
            'transition: right 0.25s ease',
            'box-sizing: border-box',
        ].join('; ');

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

        resizer.addEventListener('mousedown', function (e) {
            e.preventDefault();
            var startX = e.clientX;
            var startW = panel.offsetWidth;
            iframe.style.pointerEvents = 'none';

            function onMove(e) {
                var dx   = startX - e.clientX;
                var newW = Math.min(PANEL_MAX_W, Math.max(PANEL_MIN_W, startW + dx));
                panel.style.width = newW + 'px';
                if (panel._open) {
                    document.body.style.transition = 'none';
                    document.body.style.marginRight = newW + 'px';
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

        var toggle = document.createElement('div');
        toggle.id = TOGGLE_ID;
        toggle.innerHTML = '&#x276E;';
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

        setTimeout(openPanel, 50);
    }

    function cleanupPanel() {
        if (typeof window._h2o2_stopLivePreview === 'function') {
            window._h2o2_stopLivePreview();
        }

        var panel   = document.getElementById(PANEL_ID);
        var toggle  = document.getElementById(TOGGLE_ID);
        var actions = document.getElementById(ACTIONS_ID);
        if (panel)   panel.remove();
        if (toggle)  toggle.remove();
        if (actions) actions.remove();
        document.body.style.marginRight = '';
        console.log('H2O2: 面板已清理');
    }

    function startObserver() {
        if (window[OBSERVER_KEY]) {
            try { window[OBSERVER_KEY].disconnect(); } catch (e) {}
        }
        var obs = new MutationObserver(function () {
            var ta     = document.getElementById('edit-page-textarea');
            var panel  = document.getElementById(PANEL_ID);

            if (!ta && panel) {
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
    }

    setInterval(function () {
        if (!window[OBSERVER_KEY]) {
            startObserver();
            return;
        }
        var ta    = document.getElementById('edit-page-textarea');
        var panel = document.getElementById(PANEL_ID);

        if (ta && !panel) {
            inject(ta);
        }
        if (ta) {
            ta.style.setProperty('display', 'none', 'important');
        }
        if (!ta && panel) {
            cleanupPanel();
        }
    }, 1000);

    startObserver();
    var ta = document.getElementById('edit-page-textarea');
    if (ta) inject(ta);

    console.log('H2O2: v2.4.0 已启动');
    console.log("%c 翻译环境检测完成：一切正常运行。", "color: #a7b8c8;");
    console.log("%c 愿每一次预定都被尊重。", "color: #ff0000;");
})();
