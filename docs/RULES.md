# Packet rules

Packet rules are Phase 2 predicates evaluated on every captured chunk. A rule
can allow or drop a chunk; it never rewrites bytes.

## CLI

    nazuko proxy \
      --listen 127.0.0.1:25565 \
      --target 127.0.0.1:25566 \
      --rule 'drop dir=c2s proto=mc contains=hello'

Repeat --rule to add rules. Order matters: the first matching rule decides.

## Syntax

    action key=value [key=value ...]

action is drop or allow.

| Key | Value | Meaning |
| --- | --- | --- |
| name | text | Optional label; default rule-N |
| dir | c2s, s2c | Direction |
| proto | mc, http, tls, dns, ws, quic, raw | Detected protocol |
| src_port | 1-65535 | Source port |
| dst_port | 1-65535 | Destination port |
| min_len | bytes | Minimum chunk length, inclusive |
| max_len | bytes | Maximum chunk length, inclusive |
| contains | text | Payload contains this byte string |
| prefix | text | Payload starts with this byte string |
| suffix | text | Payload ends with this byte string |

Values containing spaces use double quotes:

    --rule 'drop proto=http contains="GET /admin"'

Inside double quotes, the escapes \n, \r, \t, \\ and \" are accepted. A rule
needs at least one predicate; a name alone is not enough.

## Semantics

- Every configured predicate must match.
- Rules are evaluated in order; the first match wins.
- No matching rule means allow.
- An allow rule can whitelist traffic before a later drop rule.
- Evaluation is per captured chunk, not per TCP packet or Minecraft frame.
- --drop-direction still works independently of rules.
- A matched rule is reported in PacketEvent.matched_rule and printed as
  rule=name. Dropped chunks are still written to pcap.
- Rules only allow or drop. There is no rewrite, evasion or stealth mode.

## Examples

    --rule 'drop dir=c2s proto=mc contains=hello'
    --rule 'allow proto=tls'
    --rule 'drop dir=s2c min_len=4096'
    --rule 'drop proto=http prefix=GET name=block-get'

## Safety

Rules operate on systems the user owns or is authorized to test. They do not
provide stealth, evasion or unauthorized remote control.
