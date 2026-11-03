# Contributing to NetTool ζ

Thanks for your interest. NetTool welcomes issues and pull requests.

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
- Security-sensitive changes need prior discussion. NetTool is for authorized
  testing only; do not add stealth, evasion, or unauthorized control features.
- By contributing you agree your changes are licensed under Apache-2.0.

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
