# Security policy

## Reporting a vulnerability

Open a private security advisory on the GitHub repository, or contact the
maintainer directly. Do not open a public issue for a vulnerability that could
be exploited before a fix is available.

Please include:

- affected version or commit
- a minimal reproduction
- impact and whether it requires network access
- any suggested fix

## Supported versions

Phase 1 is pre-1.0. Only the current main branch receives fixes.

## Safety boundary

NetTool is a network protocol testing tool for systems you own or are
explicitly authorized to test. It is not an attack service.

The core does not ship:

- stealth or detection-evasion features
- automated ARP flooding, botnet control, or credential theft
- a default-on remote control surface

The proxy binds to 127.0.0.1 by default in the CLI. If you change that, you are
responsible for network exposure and access control.

## Open source is not authorization

An open-source license grants rights to the code. It does not grant permission
to intercept, disrupt, or access a network or computer you do not own. See
docs/AUTHORIZED-USE.md.
