//! Cooperative server-worker admission for one Linux network namespace.
//! A control socket path is not network ownership: custom paths and private /run
//! mounts can coexist while sharing the rules swept by NAT startup recovery.
//! Abstract sockets are scoped by the kernel network namespace, released at
//! process death, and never unlinked. They do not claim authority over old binaries
//! or external firewall managers; durable exact-rule recovery is separate.
use std::{
    io,
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixDatagram},
    },
};

const NAME: &[u8] = b"qeli.server.worker";

pub(super) fn acquire() -> anyhow::Result<UnixDatagram> {
    bind(NAME).map_err(|error| anyhow::anyhow!(
        "server worker network namespace already owned or reservation unavailable: {error}; stop the other worker or use a separate network namespace (a different control socket path is not isolation)"
    ))
}

fn bind(name: &[u8]) -> io::Result<UnixDatagram> {
    // Rust's socket constructor sets close-on-exec. No messages or listening
    // backlog are needed: one live bound datagram socket excludes every peer.
    UnixDatagram::bind_addr(&SocketAddr::from_abstract_name(name)?)
}

#[cfg(test)]
#[path = "network_lease_tests.rs"]
mod tests;
