//! Linux profile DNS listeners. Packet parsing, cache policy and upstream exchanges
//! live in the portable resolver engine so host tests exercise the production code.

use crate::config::server::DnsConfig;
use crate::dns_resolver::{apply_udp_size_limit, resolve};
pub(crate) use crate::dns_resolver::{compile_blocklist, new_cache};
pub use crate::dns_resolver::{DnsBlocklist, DnsCache};
use crate::server::ServerState;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Upper bound on in-flight query TASKS. The permit is taken in the accept loop
/// BEFORE spawning (see the loop below), so a flood is bounded by refusing to
/// start work rather than by parking an unbounded number of started tasks.
const MAX_INFLIGHT: usize = 512;

/// Bind the proxy's listen socket, SEPARATELY from serving on it.
///
/// The bind used to happen inside the detached serve task, so a port already taken — the
/// common case, a host resolver on `0.0.0.0:53` covering the TUN address — surfaced as one
/// ERROR line while the profile came up regardless and handed every client the address of a
/// resolver that does not exist. Names then simply stopped resolving with nothing pointing at
/// the cause. Binding here lets the caller fail the profile BEFORE it advertises a resolver it
/// cannot provide. (Audit 2026-08-01, §4.)
pub async fn bind_dns_proxy(dns_cfg: &DnsConfig) -> anyhow::Result<UdpSocket> {
    bind_dns_proxy_at(dns_cfg, &dns_cfg.listen).await
}

pub async fn bind_dns_proxy_at(dns_cfg: &DnsConfig, listen: &str) -> anyhow::Result<UdpSocket> {
    let bind_addr = crate::util::join_host_port(listen, dns_cfg.port);
    UdpSocket::bind(&bind_addr)
        .await
        .map_err(|e| anyhow::anyhow!("DNS proxy cannot bind {bind_addr}: {e}"))
}

/// Bind the TCP half of the resolver, on the same address and port as the UDP one.
///
/// DNS over TCP is not an optional extra: RFC 7766 makes it a REQUIREMENT for every resolver,
/// and it is what a client does after receiving a truncated answer. Without it this proxy could
/// not honestly set TC — telling a client "ask again over TCP" while listening on UDP alone
/// would have turned every oversized answer into a failed lookup, which is why the UDP path
/// used to forward answers whole regardless of size. Binding it is what makes the TC path in
/// `apply_udp_size_limit` truthful. (Audit 2026-08-01, §10.)
pub async fn bind_dns_proxy_tcp(dns_cfg: &DnsConfig) -> anyhow::Result<TcpListener> {
    bind_dns_proxy_tcp_at(dns_cfg, &dns_cfg.listen).await
}

pub async fn bind_dns_proxy_tcp_at(
    dns_cfg: &DnsConfig,
    listen: &str,
) -> anyhow::Result<TcpListener> {
    let bind_addr = crate::util::join_host_port(listen, dns_cfg.port);
    TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| anyhow::anyhow!("DNS proxy cannot bind {bind_addr}/tcp: {e}"))
}

/// Serve DNS over TCP: length-prefixed messages (RFC 1035 §4.2.2), one task per connection,
/// sharing the blocklist, cache and upstream policy with the UDP path via [`resolve`].
pub(crate) async fn run_dns_proxy_tcp(
    dns_cfg: DnsConfig,
    listener: TcpListener,
    cache: DnsCache,
    pref: Arc<AtomicUsize>,
    blocklist: DnsBlocklist,
    tasks: super::ProfileTasks,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let cfg = Arc::new(dns_cfg);
    // The same in-flight bound as UDP, for the same reason: a flood must be refused rather
    // than parked. TCP additionally bounds itself by the accept queue.
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT));
    loop {
        let (mut stream, _peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("DNS proxy (tcp) accept error: {} — continuing", e);
                continue;
            }
        };
        let Ok(permit) = sem.clone().try_acquire_owned() else {
            continue; // at capacity: dropping the connection is the bound
        };
        let cache = cache.clone();
        let cfg = cfg.clone();
        let pref = pref.clone();
        let blocklist = blocklist.clone();
        tasks.spawn(async move {
            let _permit = permit;
            // The timeout bounds IDLE time, not the connection.
            //
            // It used to wrap the whole exchange, so a perfectly healthy persistent connection
            // — which is the entire point of RFC 7766 — was cut off `timeout_secs` after
            // accept, mid-query, however busy it was. What needs bounding is a peer that
            // connects and then says nothing; a peer that keeps asking questions is a peer the
            // service exists for. Applying it per read gives that, and still releases the
            // in-flight slot from a stalled client. (Audit 2026-08-01, §10.)
            let idle = Duration::from_secs(
                cfg.timeout_secs
                    .clamp(1, crate::config::server::DNS_MAX_TIMEOUT_SECS),
            );
            // RFC 7766 allows several queries per connection; serve until the peer closes.
            loop {
                let mut len_buf = [0u8; 2];
                match tokio::time::timeout(idle, stream.read_exact(&mut len_buf)).await {
                    Ok(Ok(_)) => {}
                    // Clean close, broken peer or an idle timeout — either way, done.
                    _ => return,
                }
                let qlen = u16::from_be_bytes(len_buf) as usize;
                if qlen < 12 {
                    return; // shorter than a DNS header
                }
                let mut query = vec![0u8; qlen];
                // The BODY gets its own deadline too: a peer that announces a length and then
                // dribbles could otherwise hold the slot indefinitely between reads.
                match tokio::time::timeout(idle, stream.read_exact(&mut query)).await {
                    Ok(Ok(_)) => {}
                    _ => return,
                }
                let Some(resp) = resolve(
                    cache.clone(),
                    cfg.clone(),
                    pref.clone(),
                    blocklist.clone(),
                    &query,
                )
                .await
                else {
                    return;
                };
                // TCP carries the full answer without the downstream UDP size cap. If no
                // upstream supplied a full answer, preserve the last-resort TC response.
                let Ok(len) = u16::try_from(resp.len()) else {
                    return;
                };
                let mut framed = Vec::with_capacity(2 + resp.len());
                framed.extend_from_slice(&len.to_be_bytes());
                framed.extend_from_slice(&resp);
                match tokio::time::timeout(idle, stream.write_all(&framed)).await {
                    Ok(Ok(())) => {}
                    _ => return,
                }
            }
        });
    }
}

pub(crate) async fn run_dns_proxy(
    _state: Arc<ServerState>,
    dns_cfg: DnsConfig,
    bound: UdpSocket,
    cache: DnsCache,
    pref: Arc<AtomicUsize>,
    blocklist: DnsBlocklist,
    tasks: super::ProfileTasks,
) -> anyhow::Result<()> {
    let bind_addr = crate::util::join_host_port(&dns_cfg.listen, dns_cfg.port);
    // Shared listen socket: query tasks send their answers back through it.
    let socket = Arc::new(bound);
    log::info!("DNS proxy listening on {}", bind_addr);

    let cfg = Arc::new(dns_cfg);
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT));
    // Count of queries refused because the in-flight gate was full (for rate-limited logging).
    let dropped = Arc::new(AtomicU64::new(0));
    // The full UDP payload maximum, not a guess at what clients "should" send. `recv_from`
    // discards whatever does not fit, silently, so any bound below 65535 turns a legal
    // datagram — a large TSIG or an EDNS0 query from a client that advertised
    // room for it — into a truncated message forwarded upstream as if it were whole. This is
    // ONE buffer for the whole accept loop, so the ceiling costs 64 KiB per profile, once.
    // (Audit 2026-08-01, §10.)
    let mut buf = vec![0u8; 65_535];
    loop {
        let (n, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                // A transient recv error must not tear down the whole DNS proxy for the
                // profile (mirrors the UDP data-plane worker's log-and-continue).
                log::warn!("DNS proxy recv error: {} — continuing", e);
                continue;
            }
        };
        // A valid DNS message has at least the 12-byte header.
        if n < 12 {
            continue;
        }
        // Take the in-flight permit HERE, before spawning. Acquiring it inside the task
        // (as this did) bounds only the upstream work: the spawn itself always succeeds,
        // so a flood piles up an unbounded number of tasks, each parked on the semaphore
        // while holding its own copy of the datagram — memory grows without limit even
        // though "in-flight" looks capped. Refusing to start the task is the actual bound;
        // a dropped UDP query is retried by the client, an OOM is not. (S-02)
        let permit = match sem.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                // Rate-limited: under a flood this fires on every packet otherwise.
                let n = dropped.fetch_add(1, Ordering::Relaxed) + 1;
                if n % 1000 == 1 {
                    log::warn!(
                        "DNS proxy: {} in-flight queries — dropping (total dropped: {})",
                        MAX_INFLIGHT,
                        n
                    );
                }
                continue;
            }
        };
        let query = buf[..n].to_vec();
        // Each query is handled on its own task so a slow/unreachable upstream
        // can't stall every other client's lookup (the old single-socket loop
        // blocked the whole proxy on each query — head-of-line blocking).
        let socket = socket.clone();
        let cache = cache.clone();
        let cfg = cfg.clone();
        let pref = pref.clone();
        let blocklist = blocklist.clone();
        tasks.spawn(async move {
            handle_query(socket, cache, cfg, permit, pref, blocklist, query, src).await;
        });
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_query(
    socket: Arc<UdpSocket>,
    cache: DnsCache,
    cfg: Arc<DnsConfig>,
    // Held for the whole task and released on return — the caller acquired it before
    // spawning us, so the number of live tasks is what MAX_INFLIGHT actually bounds. (S-02)
    _permit: OwnedSemaphorePermit,
    pref: Arc<AtomicUsize>,
    blocklist: Arc<HashSet<String>>,
    query: Vec<u8>,
    src: SocketAddr,
) {
    let Some(resp) = resolve(cache, cfg, pref, blocklist, &query).await else {
        return;
    };
    // Only the UDP path has a size limit to respect; over TCP the answer goes out whole.
    let Some(out) = apply_udp_size_limit(&query, resp) else {
        return;
    };
    let _ = socket.send_to(&out, src).await;
}
