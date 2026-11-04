<!--
本文件由 SilentStudio 监管与负责。
本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
版权所有 © SilentStudio。
以 AGPL-3.0-or-later 授权。
This document is supervised and responsible by SilentStudio.
This file is part of the source code of the SilentCodeTeams product Nazu&Ko
SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
Copyright © SilentStudio.
Licensed as AGPL-3.0-or-later.
-->

# 官方插件

官方插件是单文件包，扩展名：

    .nkplgpk

内部名称：NazuKo Plugin Packing（NKPLGPK）。

插件包通过官方签名校验，不依赖文件后缀，也不依赖压缩软件打不开这一点来防伪。普通压缩软件无法直接打开，是因为容器使用了加密和专用布局，不是安全边界。

## 适合做成插件的功能

核心安装包保留：

- 代理、抓包、会话模型
- 请求/响应工作台
- Replay、规则、CLI、MCP
- 基础协议视图

官方可选插件：

- browser-chromium
- browser-webkit（后续）
- forensics-pack
- protocol-pack-extended
- scanner-templates
- tshark-bridge

浏览器内核、取证文件复原、额外协议包、扫描模板和外部 tshark 桥接都体积大、更新节奏不同，适合单独下载。

## 目录

    plugins/
      README.md
      index.nkcat          官方签名目录（后续）
      official/            官方 .nkplgpk 包或 Release 资产（后续）

大型包优先走 GitHub Releases；仓库只保留索引、签名和说明。需要 Git LFS 时再启用。

## 三方插件

第三方开发者使用独立格式：

    .nkplug

三方插件使用开发者自己的签名密钥，不能生成官方 .nkplgpk。官方格式的私钥只存在于 SilentStudio 的离线签名流程中。

详细格式、权限、目录校验和安装流程见：

    ../docs/PLUGINS.md
