//! qeli library crate.
//!
//! The modules live here (rather than in `main.rs`) so the realtls core can be
//! built as a `cdylib` for Android/Windows via [`protocol::realtls::ffi`]. The
//! server/client/TUN/web modules are Linux-only; the cross-platform pieces
//! (config, crypto, protocol — including the realtls FFI) build everywhere.

pub mod config;
pub mod crypto;
pub mod protocol;
// Cross-platform whole-client lifecycle and platform-plan boundary. The current Linux
// client is migrated onto this incrementally; keeping the module platform-neutral lets
// every GUI client consume the same state machine through its optional C ABI.
pub mod transport_core;
// Cross-platform helpers (atomic file writes etc.); builds everywhere, including
// the realtls FFI cdylib for Android/Windows/macOS.
pub mod util;

// Resolver wire/cache/upstream code has no Linux dependencies. Compile it in host tests
// too; GUI native release libraries keep it excluded and Linux owns the socket lifecycle.
#[cfg(any(test, all(target_os = "linux", feature = "server")))]
#[path = "server/dns/resolver.rs"]
mod dns_resolver;

// Profile ownership is platform-neutral and exercised without privileged network setup.
#[cfg(any(test, all(target_os = "linux", feature = "server")))]
#[path = "server/tasks.rs"]
mod profile_tasks;

// Delivery admission/lifetime is tested without making external HTTP requests.
#[cfg(any(test, all(target_os = "linux", feature = "server")))]
#[path = "server/notify_tasks.rs"]
mod notify_tasks;

#[cfg(all(test, feature = "server", not(target_os = "linux")))]
#[allow(dead_code)] // Host tests exercise helpers; server entry points run only on Linux.
#[path = "server/notify.rs"]
mod server_notify;

// Control protocol bounds are shared by the Unix server and its CLI client.
#[cfg(any(test, all(target_os = "linux", feature = "server")))]
#[path = "server/control_io.rs"]
mod control_io;

// Process ownership/retry logic is tested with isolated child processes on the host.
#[cfg(any(test, all(target_os = "linux", feature = "server")))]
#[path = "server/supervisor.rs"]
mod server_supervisor;

#[cfg(test)]
#[path = "server/dns/test_support.rs"]
mod dns_test_support;

// Compile the actual listeners in host tests too; Linux already includes them via server.
#[cfg(all(test, not(all(target_os = "linux", feature = "server"))))]
#[path = "server/dns.rs"]
mod dns_listeners;

// Accounting uses portable counters and atomic file writes; exercise it on the host too.
#[cfg(all(test, not(all(target_os = "linux", feature = "server"))))]
#[path = "server/usage.rs"]
mod server_usage;

// One cross-process ownership journal for every Linux component that changes host-wide
// forwarding sysctls. The full daemon can run server profiles and panel-managed outbound
// clients at the same time, so separate server/client snapshots would race on teardown.
#[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
#[path = "client/sysctl.rs"]
pub(crate) mod sysctl;

// Linux daemon socket-option helpers and transport constants. The cross-platform client
// carrier itself lives in `transport_core`; these helpers remain for the Linux server/CLI
// path. `ring`-free, so they cross-compile to mipsel/aarch64.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub mod transport;

// Exercise bounded hook process I/O with real host child fixtures. Linux clients and
// servers use this same runner; native GUI release libraries do not include it.
#[cfg(any(
    test,
    all(target_os = "linux", any(feature = "client", feature = "server"))
))]
#[path = "hooks/process.rs"]
mod hook_process;

// Headless credential I/O, shutdown ordering and cleanup policy have portable host tests.
#[cfg(any(test, all(target_os = "linux", feature = "client")))]
mod client_cleanup;
#[cfg(any(test, all(target_os = "linux", feature = "client")))]
mod client_tasks;
#[cfg(any(test, all(target_os = "linux", feature = "client")))]
mod credential_file;
#[cfg(any(test, all(target_os = "linux", feature = "client")))]
mod dns_backup;
#[cfg(any(test, all(target_os = "linux", feature = "client")))]
mod secret_buffer;

// Bind command authorization to the exact descriptor supplying Linux runtime INI bytes.
#[cfg(any(
    test,
    all(target_os = "linux", any(feature = "client", feature = "server"))
))]
mod config_source;

// Lifecycle hooks (post_up/post_down); used by both the client and server, Linux-only.
// Command trust is supplied by config_source above.
#[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
pub mod hooks;

// Opt-in packet timeline (`QELI_TRACE`). Gated like `hooks`: it instruments the
// client/server data planes and pulls in tokio's signal handling, neither of which
// belongs in the realtls cdylib.
#[cfg(all(target_os = "linux", any(feature = "client", feature = "server")))]
pub mod trace;

// `client` builds under feature = "client"; `server`/`web` under feature = "server".
// Linux TUN is shared by either feature. Default features enable both, so a normal build is
// unchanged. A router (Keenetic) build uses `--no-default-features --features
// client-bin` to drop the server/web stack (and its MIPS-incompatible `ring`).
#[cfg(any(
    all(target_os = "linux", feature = "client"),
    all(
        any(
            target_os = "android",
            target_os = "windows",
            target_os = "macos",
            target_os = "ios"
        ),
        feature = "transport-core-ffi"
    )
))]
pub mod client;
#[cfg(all(target_os = "linux", feature = "server"))]
pub mod server;
// `tun::tap` contains the platform-neutral Ethernet framing helpers consumed by the
// fd-backed Android/macOS core. The actual TUN device implementation remains Linux-only
// inside `tun::iface`.
#[cfg(any(
    all(target_os = "linux", any(feature = "client", feature = "server")),
    all(
        any(target_os = "android", target_os = "macos"),
        feature = "transport-core-ffi"
    )
))]
pub mod tun;
#[cfg(all(target_os = "linux", feature = "server"))]
pub mod web;
