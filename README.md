# SCP Foundation Editor (Web Edition)

![License](https://img.shields.io/badge/license-AGPL--3.0-red.svg)
![Version](https://img.shields.io/badge/version-alpha-orange.svg)
![Platform](https://img.shields.io/badge/platform-Web%20%7C%20Windows%20%7C%20macOS-blue.svg)
![Language](https://img.shields.io/badge/language-JavaScript-yellow.svg)
![Language](https://img.shields.io/badge/language-CSS-purple.svg)

一个专为 SCP 基金会中文站设计的 Wikidot 语法编辑器，旨在提供流畅的实时编写体验。

## 🚀 特性
- **全平台覆盖**：支持浏览器端及桌面端（Windows/macOS/Linux）。
- **组件集成**：内置对主流社区组件 ACS 和 AIM 的完美支持。
- **现代交互**：基于 CodeMirror 6，集成 One Dark 主题及实时字符统计。

## 📚 引用与鸣谢
本项目集成并致敬以下社区组件：

* **AIM (Advanced Information Methodology)** — By *Dr Moned*
    * [查看组件页面](https://scp-wiki.wikidot.com/component:advanced-information-methodology)
* **ACS Animation (Anomaly Classification System)** — By *EstrellaYoshte*
    * [查看组件页面](https://scp-wiki.wikidot.com/component:acs-animation)

## 🛠 构建指南
本项目前端部分使用 `esbuild` 进行模块打包。

```bash
# 安装依赖
npm install

# 执行打包
npm run build
