// ==UserScript==
// @name         SCP Wiki Logo Replacer
// @namespace    http://tampermonkey.net/
// @version      1.2
// @description  强制替换 SCP Wiki 顶部 logo 图标
// @author       wasd243
// @match        *://scp-wiki.wikidot.com/*
// @match        *://www.scp-wiki.wikidot.com/*
// @icon         https://scpsandboxcn.wikidot.com/local--files/peroxide-hyroperoxide/%E5%8F%8D%E5%90%91%E7%9A%84%E6%97%A0%E5%B0%BD%E5%82%AC%E5%8C%96%E5%89%82%EF%BC%88%E9%95%87%E5%9C%BA%E5%AD%90%E7%94%A8%E7%9A%84%EF%BC%89
// @grant        none
// ==/UserScript==

(function () {
    'use strict';

    const NEW_LOGO_URL = 'https://scp-wiki.wikidot.com/local--files/theme:basalt/basalt_scp_logo-for_lightmode.svg';

    // 排除账户头像
    const selectors = [
        '#header h1 img',
        '#header h2 img',
        '#header .logo img',
    ];

    function replaceLogo() {
        selectors.forEach(sel => {
            document.querySelectorAll(sel).forEach(img => {
                if (img.src !== NEW_LOGO_URL) {
                    img.src = NEW_LOGO_URL;
                    img.srcset = '';
                }
            });
        });
    }

    replaceLogo();

    const observer = new MutationObserver(() => replaceLogo());
    observer.observe(document.documentElement, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ['src'],
    });
})();
