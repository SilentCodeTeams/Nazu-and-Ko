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

# NKPLGPK 插件格式与防伪

## 目标

- 官方插件以单文件分发
- 普通压缩软件不能直接打开
- 安装前完成完整性和来源验证
- 支持官方插件集合的目录级完整性
- 支持撤销、回滚保护和最小核心版本
- 三方插件可以开发，但不能伪造官方包

## 核心边界

核心安装包不内置大体积或独立更新节奏的能力。以下功能默认插件化：

- Chromium / WebKit Browser Host
- 文件、图片、凭据取证
- 扩展协议包
- 扫描模板和字典
- 可选 tshark 桥接

Npcap 仍是 Windows 平台依赖，不塞进 .nkplgpk。

## 容器布局

.nkplgpk 是单文件容器：

    magic          8 bytes   NKPLGPK\0
    version        2 bytes
    flags          2 bytes
    header_len     4 bytes
    header         header_len bytes (CBOR/JSON, authenticated)
    nonce          24 bytes
    ciphertext     XChaCha20-Poly1305(zstd(tar(payload)))
    signature      64 bytes  Ed25519(header || nonce || ciphertext)

- 载荷使用 zstd 压缩后再加密
- 加密用于防止随意解包和窥视，不承担来源认证
- 来源认证只靠 Ed25519 签名
- 签名覆盖容器头和全部密文
- 解析器必须在解密前检查长度、版本、算法 ID 和上限

不要使用自制的加密算法。XChaCha20-Poly1305、Ed25519、SHA-256 和 zstd 都是成熟原语。

## 载荷

payload 内至少包含：

    identity.nkmeta
    payload/...
    LICENSE
    NOTICE

identity.nkmeta 是 JSON，字段：

    plugin_id
    display_name
    version
    publisher
    publisher_key_id
    min_core_version
    entrypoint
    permissions
    files[] {path, sha256, size}
    catalog_epoch
    catalog_root
    build_id

files[] 列出载荷中每个文件的哈希。安装器解包前和安装后各校验一次，拒绝路径穿越、绝对路径、符号链接和重复路径。

## 第二层：官方目录

不把“其他所有插件的哈希”直接写进每个包，因为新增或删除插件会让所有包失效。使用一个签名的目录文件：

    plugins/index.nkcat

目录包含：

    epoch
    generated_at
    plugins[] {plugin_id, version, bundle_sha256, files_root, publisher_key_id}
    merkle_root
    revocation_list

index.nkcat 由官方目录密钥签名。客户端验证：

1. 插件自身的 Ed25519 签名
2. identity.nkmeta 中每个文件的 SHA-256
3. 插件在 index.nkcat 中的 bundle_sha256
4. index.nkcat 的签名与 epoch
5. 当前 epoch 不低于本地记录，防止回滚

这样第二层是目录级完整性，不是包内互相互引。互相互引会造成循环依赖和无法升级。

## 官方密钥

- 官方签名私钥离线保存，理想状态在 HSM 或硬件令牌中
- 构建机只拿到受限签名服务，不持有私钥
- 客户端固定官方公钥和 key_id
- 支持密钥轮换：新 key_id 由旧 key 签名过渡
- 支持吊销列表，吊销优先于版本号

## 三方插件

三方格式使用：

    .nkplug

三方包使用开发者自己的 Ed25519 密钥。首次安装必须明确显示发布者、权限和信任状态。客户端不会把三方 key 当作官方 key，也不会让三方包生成官方 .nkplgpk。

三方插件优先运行在 WASM 或独立插件宿主进程中；官方原生插件也必须进程隔离，不能直接在主 UI 进程执行。

## 权限与执行

identity.nkmeta 的 permissions 默认拒绝：

    network
    filesystem
    process
    capture
    ui
    secrets

安装器先验证、后解包、再原子替换。运行插件时按权限创建宿主进程，通过受控 IPC 提供能力。插件不能直接访问用户主目录、密钥文件或其他插件数据。

## CLI 与 MCP

规划命令：

    nazuko plugin list
    nazuko plugin verify <file.nkplgpk>
    nazuko plugin install <file.nkplgpk>
    nazuko plugin remove <plugin_id>

MCP 工具可以由插件注册，但注册前必须验证签名和权限。MCP 的错误仍保持英文和短消息。

## 当前状态

这里定义格式和安全边界，尚未实现签名器、验证器和插件宿主。下一步先实现：

1. 只读 verify 命令
2. 官方公钥和 index.nkcat 验证
3. 原子安装/卸载
4. 插件宿主进程

没有经过验证的包，不执行。
