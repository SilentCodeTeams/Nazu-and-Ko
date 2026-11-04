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

# Contributing to Nazu&Ko

Thanks for your interest. Nazu&Ko welcomes issues and pull requests.

## Ground rules

- Open an issue before large features or refactors.
- Keep each pull request focused on one change.
- Do not rewrite, delete, or move unrelated files.
- Do not overwrite or revert another contributor's work. If you need to change
  someone else's code, explain why in the pull request.
- Never force-push to the main branch.
- Run the quality gates before opening a pull request:
      cargo fmt --all -- --check
      cargo clippy --workspace --all-targets -- -D warnings
      cargo test --workspace
- Security-sensitive changes need prior discussion. Nazu&Ko is for authorized
  testing only; do not add stealth, evasion, or unauthorized control features.
- By contributing you agree your changes are licensed under AGPL-3.0-or-later.

## Workflow

1. Open an issue for non-trivial work.
2. Fork the repository and create a branch.
3. Add tests for behavior changes.
4. Open a pull request and fill in the template.
5. Wait for CI and review.

## Scope

We welcome protocol dissectors, capture backends, tests, documentation and
cross-platform fixes. We do not accept features that enable unauthorized
access, evasion or harm.

The main branch is protected against force pushes and deletion.
