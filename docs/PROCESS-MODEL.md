# Process model

## Why separate processes

- A UI crash must not stop capture, forwarding or a MAC lease.
- The UI must not need elevated privileges.
- The daemon can run headless, as a service, or on a lab machine.
- Clients (Qt, CLI, tests) are interchangeable because they use the same API.

## Processes

    Process        Language   Privilege        Owns
    ------------   --------   --------------   -----------------------------
    Qt client      C++/Qt 6   user             views, dialogs, user input
    nazuko       Rust       user             sessions, proxy, capture, rules
    nettool-macd   Rust       root/admin       MAC leases, interface restore
    plugins        C ABI      daemon's user    dissectors, capture backends

The Qt client never talks to nettool-macd directly. The path is:

    Qt client -> nazuko control API -> nettool-macd helper API

## Crash and restart matrix

    Event                    Capture/proxy     MAC lease        UI
    -----------------------  ---------------   --------------   -------------
    UI crash                 continues         continues        restart
    UI closed by user        continues         continues        closed
    nazuko crash           stops             continues        reconnect
    nazuko restart         new sessions      continues        reconnect
    nettool-macd crash       continues         restored/retry   shows error
    reboot                   stops             OS default       starts fresh

The MAC lease deliberately survives UI and daemon restarts. It ends only by an
explicit release command or an explicit helper shutdown. A lease is never
released just because a client disconnected.

## MAC lease lifecycle

    idle
      -> active        explicit acquire (interface + requested identity)
      -> releasing     explicit release or explicit shutdown
      -> released      restore verified
      -> idle

Rules:

- acquire is idempotent per interface: an existing active lease is returned.
- release requires a lease id and is explicit; no timeout silently restores
  unless the operator configured one.
- helper shutdown releases every active lease and restores the original value.
- every transition is written to an audit log.
- the helper is its own service (launchd, systemd, Windows service) so it can
  outlive the daemon.

## Safety controls

- The helper API is local-only and token authenticated; the token file is 0600.
- The helper requires an explicit privileged start and logs the invoking user.
- The helper refuses to acquire without an original identity to restore.
- A dry-run backend is the default until a real OS backend is selected.
- The UI shows active leases after reconnect; it cannot silently forget them.
- There is no stealth or evasion mode. Operation is limited to systems the user
  owns or is authorized to test.

## Service layout (planned)

    macOS:   launchd daemon/agent for nazuko and nettool-macd
    Linux:   systemd units, CAP_NET_RAW / CAP_NET_ADMIN
    Windows: Windows service for nazuko and nettool-macd

The first implementation keeps both in the foreground for development and adds
service packaging in Phase 3.
