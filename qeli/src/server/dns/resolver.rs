//! Portable DNS resolver engine, shared by UDP/TCP listeners and host unit tests.

use crate::config::server::DnsConfig;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

/// (response_bytes, inserted_at, ttl), keyed by the txid-normalised query.
///
/// The TTL is PER ENTRY, taken from the record itself (S-14). It used to be one global
/// `dns.timeout_secs` for everything, which is not a caching policy at all: a record the
/// zone says is valid for 5 s was served stale for the whole timeout, and a record valid
/// for a day was re-queried just as often. `timeout_secs` is a network timeout; reusing it
/// as a cache lifetime conflated two unrelated settings.
pub type DnsCache = Arc<RwLock<HashMap<Vec<u8>, (Vec<u8>, Instant, Duration)>>>;
pub type DnsBlocklist = Arc<HashSet<String>>;

/// Ceiling on a record-derived cache lifetime. Short authoritative TTLs are honoured exactly;
/// raising 1..4 seconds to a local floor serves records beyond the lifetime chosen by the zone.
const MAX_CACHE_TTL: Duration = Duration::from_secs(3600);

/// Walk a compressed DNS name once for both framing and blocklist decoding.
/// Return the end of its on-wire representation, but validate the entire expanded name.
/// Pointers refer backwards to names, never into the header; labels and pointer chains
/// have bounded work and the expanded name includes its terminating root in the 255 limit.
fn walk_name<'a>(msg: &'a [u8], mut pos: usize, mut label: impl FnMut(&'a [u8])) -> Option<usize> {
    let mut end = None;
    let mut wire_len = 1usize;
    // A 255-byte name may have 127 labels. Compression adds hops but no expanded
    // bytes, so a 128-step total would reject its otherwise valid compressed form.
    for _ in 0..256 {
        let len = *msg.get(pos)?;
        if len & 0xC0 == 0xC0 {
            let low = *msg.get(pos.checked_add(1)?)?;
            let target = (((len & 0x3F) as usize) << 8) | low as usize;
            if target < 12 || target >= pos {
                return None;
            }
            end.get_or_insert(pos.checked_add(2)?);
            pos = target;
            continue;
        }
        if len & 0xC0 != 0 {
            return None;
        }
        if len == 0 {
            return end.or_else(|| pos.checked_add(1));
        }
        wire_len = wire_len.checked_add(1 + usize::from(len))?;
        if wire_len > 255 {
            return None;
        }
        let start = pos.checked_add(1)?;
        pos = start
            .checked_add(usize::from(len))
            .filter(|end| *end <= msg.len())?;
        label(&msg[start..pos]);
    }
    None
}

fn skip_name(msg: &[u8], pos: usize) -> Option<usize> {
    walk_name(msg, pos, |_| {})
}

/// Validate the framing of every declared DNS section. A response can have a perfectly
/// readable ANSWER TTL while its last RDATA, authority or additional record is truncated;
/// such a datagram must not be forwarded as a complete answer or become a reusable
/// cache entry.
fn dns_message_is_complete(msg: &[u8]) -> bool {
    if msg.len() < 12 {
        return false;
    }
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let counts = [
        u16::from_be_bytes([msg[6], msg[7]]) as usize,
        u16::from_be_bytes([msg[8], msg[9]]) as usize,
        u16::from_be_bytes([msg[10], msg[11]]) as usize,
    ];
    let mut pos = 12usize;
    for _ in 0..qdcount {
        let Some(next) = skip_name(msg, pos)
            .and_then(|after_name| after_name.checked_add(4))
            .filter(|end| *end <= msg.len())
        else {
            return false;
        };
        pos = next;
    }
    for count in counts {
        for _ in 0..count {
            let Some(after_name) = skip_name(msg, pos) else {
                return false;
            };
            let Some(header_end) = after_name.checked_add(10).filter(|end| *end <= msg.len())
            else {
                return false;
            };
            let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]) as usize;
            let Some(next) = header_end
                .checked_add(rdlen)
                .filter(|end| *end <= msg.len())
            else {
                return false;
            };
            pos = next;
        }
    }
    // DNS padding belongs inside an OPT RDATA. Bytes outside the counted sections are
    // unframed trailing data and are not safe to replay from a shared cache.
    pos == msg.len()
}

/// Subtract `age` seconds from every resource record's TTL, in place, saturating at zero.
///
/// Walks ANSWER + AUTHORITY + ADDITIONAL, because all three carry cacheable records and a stub
/// resolver honours the TTLs it sees in each. The OPT pseudo-record (type 41) is SKIPPED: its
/// TTL field is not a lifetime at all but the extended RCODE and EDNS flags, so decrementing it
/// would corrupt the answer's error code and DO bit rather than age anything.
///
/// A malformed message stops the walk where it stops — the records already adjusted stay
/// adjusted, which is strictly better than serving the whole thing with stale lifetimes, and no
/// index is ever written without having been bounds-checked first.
fn decrement_ttls(msg: &mut [u8], age: u32) {
    if age == 0 || msg.len() < 12 {
        return;
    }
    let counts = [
        u16::from_be_bytes([msg[6], msg[7]]) as usize, // ANCOUNT
        u16::from_be_bytes([msg[8], msg[9]]) as usize, // NSCOUNT
        u16::from_be_bytes([msg[10], msg[11]]) as usize, // ARCOUNT
    ];
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        pos = match skip_name(msg, pos).and_then(|p| p.checked_add(4)) {
            Some(p) => p,
            None => return,
        };
    }
    for count in counts {
        for _ in 0..count {
            let after_name = match skip_name(msg, pos) {
                Some(p) => p,
                None => return,
            };
            if after_name + 10 > msg.len() {
                return;
            }
            let rtype = u16::from_be_bytes([msg[after_name], msg[after_name + 1]]);
            let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]) as usize;
            let Some(record_end) = after_name
                .checked_add(10)
                .and_then(|header_end| header_end.checked_add(rdlen))
                .filter(|end| *end <= msg.len())
            else {
                return;
            };
            if rtype != 41 {
                let t = after_name + 4;
                let ttl = u32::from_be_bytes([msg[t], msg[t + 1], msg[t + 2], msg[t + 3]]);
                msg[t..t + 4].copy_from_slice(&ttl.saturating_sub(age).to_be_bytes());
            }
            pos = record_end;
        }
    }
}

/// Smallest TTL across the ANSWER section, or `None` when the message carries no answers
/// (NXDOMAIN / NODATA) or is malformed.
///
/// Walks names rather than assuming a fixed offset: DNS names are label sequences that may
/// end in a compression pointer, so the record header is not at a predictable position.
/// Only the ANSWER section is read — the OPT pseudo-record in ADDITIONAL stores extended
/// flags in its TTL field, so including it would produce a nonsense lifetime.
fn answer_min_ttl(msg: &[u8]) -> Option<u32> {
    if msg.len() < 12 {
        return None;
    }
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let ancount = u16::from_be_bytes([msg[6], msg[7]]) as usize;
    if ancount == 0 {
        return None;
    }
    let mut pos = 12;
    for _ in 0..qdcount {
        pos = skip_name(msg, pos)?.checked_add(4)?; // QTYPE + QCLASS
    }
    let mut min = u32::MAX;
    for _ in 0..ancount {
        pos = skip_name(msg, pos)?;
        if pos.checked_add(10)? > msg.len() {
            return None;
        }
        let ttl = u32::from_be_bytes([msg[pos + 4], msg[pos + 5], msg[pos + 6], msg[pos + 7]]);
        let rdlen = u16::from_be_bytes([msg[pos + 8], msg[pos + 9]]) as usize;
        min = min.min(ttl);
        pos = pos
            .checked_add(10)?
            .checked_add(rdlen)
            .filter(|end| *end <= msg.len())?;
    }
    Some(min)
}

/// RFC 2308 negative-cache lifetime from an SOA in the AUTHORITY section:
/// `min(SOA RR TTL, SOA.MINIMUM)`. A negative answer without SOA is not reusable.
fn negative_cache_ttl(msg: &[u8]) -> Option<u32> {
    if msg.len() < 12 {
        return None;
    }
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let ancount = u16::from_be_bytes([msg[6], msg[7]]) as usize;
    let nscount = u16::from_be_bytes([msg[8], msg[9]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        pos = skip_name(msg, pos)?
            .checked_add(4)
            .filter(|end| *end <= msg.len())?;
    }
    for _ in 0..ancount {
        let after_name = skip_name(msg, pos)?;
        let header_end = after_name.checked_add(10).filter(|end| *end <= msg.len())?;
        let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]) as usize;
        pos = header_end
            .checked_add(rdlen)
            .filter(|end| *end <= msg.len())?;
    }

    let mut minimum_ttl = None;
    for _ in 0..nscount {
        let after_name = skip_name(msg, pos)?;
        let header_end = after_name.checked_add(10).filter(|end| *end <= msg.len())?;
        let rtype = u16::from_be_bytes([msg[after_name], msg[after_name + 1]]);
        let rr_ttl = u32::from_be_bytes([
            msg[after_name + 4],
            msg[after_name + 5],
            msg[after_name + 6],
            msg[after_name + 7],
        ]);
        let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]) as usize;
        let record_end = header_end
            .checked_add(rdlen)
            .filter(|end| *end <= msg.len())?;
        if rtype == 6 {
            // SOA RDATA = MNAME, RNAME, SERIAL, REFRESH, RETRY, EXPIRE, MINIMUM.
            let after_mname = skip_name(msg, header_end).filter(|p| *p <= record_end)?;
            let numeric = skip_name(msg, after_mname).filter(|p| *p <= record_end)?;
            if numeric.checked_add(20)? != record_end {
                return None;
            }
            let minimum = u32::from_be_bytes([
                msg[numeric + 16],
                msg[numeric + 17],
                msg[numeric + 18],
                msg[numeric + 19],
            ]);
            let candidate = rr_ttl.min(minimum);
            minimum_ttl =
                Some(minimum_ttl.map_or(candidate, |current: u32| current.min(candidate)));
        }
        pos = record_end;
    }
    minimum_ttl
}

/// CNAME/DNAME chains may be present in a NOERROR NODATA response. ANCOUNT alone
/// does not establish that the requested RRset exists. With a whole-message cache,
/// an unresolved chain/referral needs a negative SOA lifetime or cannot be reused.
fn answer_contains_question_type(msg: &[u8]) -> Option<bool> {
    if u16::from_be_bytes([*msg.get(4)?, *msg.get(5)?]) != 1 {
        return None;
    }
    let question_end = question_section_end(msg)?;
    let qtype = u16::from_be_bytes([msg[question_end - 4], msg[question_end - 3]]);
    let ancount = u16::from_be_bytes([msg[6], msg[7]]);
    let mut pos = question_end;
    for _ in 0..ancount {
        let after_name = skip_name(msg, pos)?;
        let header_end = after_name.checked_add(10).filter(|end| *end <= msg.len())?;
        let rtype = u16::from_be_bytes([msg[after_name], msg[after_name + 1]]);
        if rtype == qtype || qtype == 255 {
            return Some(true);
        }
        let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]);
        pos = header_end
            .checked_add(usize::from(rdlen))
            .filter(|end| *end <= msg.len())?;
    }
    Some(false)
}

fn response_cache_ttl(msg: &[u8]) -> Option<Duration> {
    if !dns_message_is_complete(msg) || msg[2] & 0x02 != 0 {
        return None; // TC is a retry instruction, never a reusable answer.
    }
    // Cache only successful and standard negative outcomes. SERVFAIL, REFUSED,
    // FORMERR and other transient/policy errors must be retried upstream on the next query,
    // not amplified to every client for `dns.timeout_secs`.
    let rcode = msg[3] & 0x0F;
    if (rcode != 0 && rcode != 3) || dns_extended_rcode(msg)? != 0 {
        return None;
    }
    let answer_ttl = answer_min_ttl(msg);
    let is_negative = rcode == 3 || !answer_contains_question_type(msg)?;
    let negative_ttl = if is_negative {
        Some(negative_cache_ttl(msg)?)
    } else {
        None
    };
    let seconds = match (answer_ttl, negative_ttl) {
        // Both NXDOMAIN and NOERROR/NODATA may carry an alias chain in ANSWER.
        (Some(answer), Some(negative)) => answer.min(negative),
        (Some(answer), None) => answer,
        (None, Some(negative)) => negative,
        (None, None) => return None,
    };
    if seconds == 0 {
        return None;
    }
    Some(Duration::from_secs(seconds as u64).min(MAX_CACHE_TTL))
}

fn response_is_retryable_error(msg: &[u8]) -> bool {
    let base_retryable = msg
        .get(3)
        .is_some_and(|flags| matches!(flags & 0x0F, 1 | 2 | 4 | 5));
    base_retryable || dns_extended_rcode(msg).is_some_and(|extended| extended != 0)
}

/// One DNS query over TCP (RFC 1035 §4.2.2: each message is prefixed with its 2-byte
/// big-endian length). Used both when `dns.upstream_protocol = tcp` and as the retry
/// path when a UDP answer comes back truncated. Returns the raw response message, or
/// `None` on any timeout/IO/protocol error — the caller then falls back. (S-14)
///
/// The whole exchange shares one deadline, so a resolver that accepts the connection and
/// then stalls cannot hold the task (and its in-flight permit) open.
async fn query_tcp(addr: &str, query: &[u8], timeout: Duration) -> Option<Vec<u8>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let deadline = tokio::time::Instant::now() + timeout;
    let remaining =
        |d: tokio::time::Instant| d.saturating_duration_since(tokio::time::Instant::now());

    let mut stream =
        match tokio::time::timeout(remaining(deadline), tokio::net::TcpStream::connect(addr)).await
        {
            Ok(Ok(s)) => s,
            _ => return None,
        };

    // Length-prefixed request.
    let len = u16::try_from(query.len()).ok()?;
    let mut framed = Vec::with_capacity(2 + query.len());
    framed.extend_from_slice(&len.to_be_bytes());
    framed.extend_from_slice(query);
    if tokio::time::timeout(remaining(deadline), stream.write_all(&framed))
        .await
        .ok()?
        .is_err()
    {
        return None;
    }

    // Length-prefixed response. The 2-byte prefix is what makes the >512-byte answers
    // that triggered the TCP retry readable in the first place.
    let mut len_buf = [0u8; 2];
    if tokio::time::timeout(remaining(deadline), stream.read_exact(&mut len_buf))
        .await
        .ok()?
        .is_err()
    {
        return None;
    }
    let resp_len = u16::from_be_bytes(len_buf) as usize;
    if resp_len < 12 {
        return None; // shorter than a DNS header — not a usable message
    }
    let mut resp = vec![0u8; resp_len];
    if tokio::time::timeout(remaining(deadline), stream.read_exact(&mut resp))
        .await
        .ok()?
        .is_err()
    {
        return None;
    }
    Some(resp)
}

/// Answer one query, from the blocklist, the cache or an upstream — transport-agnostic.
///
/// Split out of `handle_query` so the TCP listener can share every bit of policy (blocklist,
/// cache, upstream selection and failover, cache admission). The alternative — a second copy
/// for TCP — is how two paths end up disagreeing about which names are blocked.
pub(crate) async fn resolve(
    cache: DnsCache,
    cfg: Arc<DnsConfig>,
    pref: Arc<AtomicUsize>,
    blocklist: Arc<HashSet<String>>,
    query: &[u8],
) -> Option<Vec<u8>> {
    let upstreams: Vec<_> = cfg
        .upstream
        .iter()
        .take(crate::config::server::DNS_MAX_UPSTREAMS)
        .filter_map(|address| address.parse().ok().map(|ip| SocketAddr::new(ip, 53)))
        .collect();
    resolve_with_upstreams(cache, cfg, pref, blocklist, query, &upstreams).await
}

// The same engine is exercised against ephemeral loopback resolvers in tests. Production
// always supplies validated configuration addresses on port 53; no new config syntax.
async fn resolve_with_upstreams(
    cache: DnsCache,
    cfg: Arc<DnsConfig>,
    pref: Arc<AtomicUsize>,
    blocklist: DnsBlocklist,
    query: &[u8],
    upstreams: &[SocketAddr],
) -> Option<Vec<u8>> {
    if !dns_message_is_complete(query) {
        return None;
    }
    let query = query.to_vec();
    let query_txid = [query[0], query[1]];

    if is_blocked(&query, &blocklist) {
        return blocked_response(&query);
    }

    // Cache key ignores the per-query transaction ID (bytes 0..2) so the same
    // question shares one entry regardless of txid.
    let mut cache_key = query.clone();
    cache_key[0] = 0;
    cache_key[1] = 0;

    let upstream_timeout = Duration::from_secs(
        cfg.timeout_secs
            .clamp(1, crate::config::server::DNS_MAX_TIMEOUT_SECS),
    );
    let cached = {
        let cache_read = cache.read().await;
        cache_read
            .get(&cache_key)
            .and_then(|(resp, time, entry_ttl)| {
                // Per-entry lifetime from the record, not the global network timeout. (S-14)
                let age = time.elapsed();
                if age < *entry_ttl {
                    Some((resp.clone(), age.as_secs()))
                } else {
                    None
                }
            })
    };
    if let Some((mut response, age_secs)) = cached {
        if response.len() >= 2 {
            response[0] = query_txid[0];
            response[1] = query_txid[1];
        }
        // Age the record TTLs by how long the answer has sat here.
        //
        // Only the transaction ID used to be rewritten, so a reply served one second before its
        // cache entry expired still claimed the FULL original TTL. Downstream resolvers and
        // stub clients cache on that number, which turns a 60-second record into up to 120
        // seconds of staleness — and the error compounds through every layer that caches. RFC
        // 2181 §5.2: a cached record is served with the remaining lifetime, not the original.
        // (Audit 2026-08-01, §10.)
        decrement_ttls(&mut response, age_secs.min(u32::MAX as u64) as u32);
        return Some(response);
    }

    if upstreams.is_empty() {
        return None;
    }

    // (The in-flight permit is already held — acquired by the accept loop before spawn.)
    // A fresh ephemeral socket per query: no cross-query demux, so one slow
    // resolver only delays its own task.
    // `dns.upstream_protocol` was parsed, serialized back out and shown in the panel, but
    // NOTHING read it — every query went out over UDP regardless. An operator who set
    // `tcp` (e.g. because the network mangles UDP/53) got silent UDP anyway. (S-14)
    let force_tcp = cfg.upstream_protocol.eq_ignore_ascii_case("tcp");

    // Re-randomise the transaction ID for the UPSTREAM query.
    //
    // The client's txid used to travel upstream verbatim, and the only anti-spoof checks
    // were that same txid plus the upstream socket address. Against an off-path attacker
    // that is 32 bits (txid + ephemeral port). Against a VPN CLIENT it is ~15: the client
    // chooses the txid, knows the exact instant the query goes out, and — because the cache
    // is shared per profile and keyed on the question with the txid zeroed — a single
    // successful spoof poisons the answer for every other user of that profile. Only the
    // ephemeral source port remains unknown, and Linux's default range is ~28k.
    //
    // A fresh random txid restores the full 32 bits, and matching the QUESTION section
    // (below) closes the birthday variant where an attacker sprays answers for a name it
    // did not ask about. The client's txid is put back before the answer is cached or
    // returned. (Audit 2026-08-04, M-10.)
    let upstream_txid: [u8; 2] = {
        let mut t = [0u8; 2];
        rand::Rng::fill_bytes(&mut rand::rng(), &mut t);
        t
    };
    let mut query = query;
    query[0] = upstream_txid[0];
    query[1] = upstream_txid[1];
    let query = query;

    let start = pref.load(Ordering::Relaxed) % upstreams.len();
    let mut response = None;
    let mut last_upstream_error = None;
    let mut truncated_fallback = None;
    let query_deadline = tokio::time::Instant::now() + upstream_timeout;
    for attempt in 0..upstreams.len() {
        let now = tokio::time::Instant::now();
        let remaining = query_deadline.saturating_duration_since(now);
        if remaining.is_zero() {
            break;
        }
        // One configured timeout bounds the complete query, not each upstream in turn. Split
        // the remaining time so a dead preferred resolver cannot consume the whole budget
        // without giving the fallback resolver a chance.
        let attempts_left = u32::try_from(upstreams.len() - attempt)
            .unwrap_or(u32::MAX)
            .max(1);
        let attempt_timeout = remaining / attempts_left;
        if attempt_timeout.is_zero() {
            break;
        }
        let attempt_deadline = now + attempt_timeout;
        let idx = (start + attempt) % upstreams.len();
        let upstream_sa = upstreams[idx];
        let upstream_ip = upstream_sa.ip();
        let upstream_addr = upstream_sa.to_string();
        if force_tcp {
            if let Some(full) = query_tcp(&upstream_addr, &query, attempt_timeout).await {
                // Same anti-spoof check as the UDP path (TCP is connection-bound, so
                // the source is implicitly the resolver we dialled).
                if response_matches(&full, upstream_txid, &query) && dns_message_is_complete(&full)
                {
                    if response_is_retryable_error(&full) {
                        last_upstream_error = Some(full);
                        continue;
                    }
                    if full[2] & 0x02 != 0 {
                        truncated_fallback = Some(full);
                        continue; // TCP framing does not imply an untruncated DNS answer.
                    }
                    response = Some(full);
                    pref.store(idx, Ordering::Relaxed);
                    break;
                }
            }
            continue;
        }
        let bind_addr = if upstream_ip.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let upstream_sock = match UdpSocket::bind(bind_addr).await {
            Ok(socket) => socket,
            Err(error) => {
                log::debug!("DNS: cannot open {bind_addr} upstream socket: {error}");
                continue;
            }
        };
        if upstream_sock.send_to(&query, upstream_sa).await.is_err() {
            continue;
        }
        // Accept only a reply that (a) came from the resolver we queried and (b)
        // carries the matching transaction ID — otherwise an off-/on-path spoof
        // could poison the cache. Bound the total wait by the configured timeout.
        // Sized from what the CLIENT advertised, with a 4 KiB floor.
        //
        // `recv_from` DISCARDS whatever does not fit, silently and with no short-read signal,
        // so the buffer has to be at least as large as the answer the upstream is entitled to
        // send — and that entitlement comes from the OPT record in the query, which is the
        // client's, forwarded verbatim. A fixed 4096 was right for the common advertisements
        // (1232/4096) and wrong for a client that asked for more: its answer came back chopped
        // mid-record and was forwarded as a malformed message. (S-14, extended 2026-08-01 §10.)
        let want = upstream_buf_size(&query);
        let mut resp_buf = vec![0u8; want];
        loop {
            let remaining = attempt_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, upstream_sock.recv_from(&mut resp_buf)).await {
                Ok(Ok((m, from))) => {
                    // Compare the FULL socket address, not just the IP. A reply is only ours
                    // if it came back from the port we sent to; accepting any port on the
                    // right host lets anything else on that machine — or anything that can
                    // spoof its address — answer for the resolver, and the txid below is only
                    // 16 bits of defence.
                    if from != upstream_sa {
                        continue; // not from the queried resolver — ignore
                    }
                    if !response_matches(&resp_buf[..m], upstream_txid, &query) {
                        continue; // wrong txid/question — spoof or stale, ignore
                    }
                    // TC is an explicit retry instruction. An exact buffer fill is the one
                    // case where `recv_from` cannot distinguish a complete datagram from one
                    // it cut off. Either condition needs one (not two) TCP retry.
                    let truncated = resp_buf[2] & 0x02 != 0;
                    let possibly_cut_off = m == resp_buf.len();
                    if truncated || possibly_cut_off {
                        if truncated {
                            log::debug!(
                                "DNS: truncated reply from {} — retrying over TCP",
                                upstream_sa
                            );
                        } else {
                            log::debug!(
                                "DNS: reply from {} exactly filled the {}-byte buffer — may \
                                 have been cut off, retrying over TCP",
                                upstream_sa,
                                resp_buf.len()
                            );
                        }
                        let tcp_budget =
                            attempt_deadline.saturating_duration_since(tokio::time::Instant::now());
                        if let Some(full) = query_tcp(&upstream_addr, &query, tcp_budget).await {
                            if response_matches(&full, upstream_txid, &query)
                                && dns_message_is_complete(&full)
                            {
                                if response_is_retryable_error(&full) {
                                    last_upstream_error = Some(full);
                                    break;
                                }
                                if full[2] & 0x02 != 0 {
                                    truncated_fallback = Some(full);
                                    break; // Give other upstreams the remaining query budget.
                                }
                                response = Some(full);
                                pref.store(idx, Ordering::Relaxed);
                                break;
                            }
                        }
                        // Give the remaining resolvers a chance before falling back to TC.
                        // An exact buffer fill without TC may have been cut mid-record and is
                        // never safe to forward. An explicit TC response is structurally useful:
                        // keep it only as a last resort so the downstream client can retry TCP.
                        if truncated && dns_message_is_complete(&resp_buf[..m]) {
                            truncated_fallback = Some(resp_buf[..m].to_vec());
                        }
                        break;
                    }
                    if !dns_message_is_complete(&resp_buf[..m]) {
                        break; // corrupt framing: try another resolver
                    }
                    if response_is_retryable_error(&resp_buf[..m]) {
                        // Return the last real DNS error if every resolver fails, but first
                        // try the next upstream instead of making this one the preferred
                        // source of SERVFAIL/REFUSED for every subsequent client.
                        last_upstream_error = Some(resp_buf[..m].to_vec());
                        break;
                    }
                    response = Some(resp_buf[..m].to_vec());
                    pref.store(idx, Ordering::Relaxed);
                    break;
                }
                _ => break, // timeout or socket error → try next upstream
            }
        }
        if response.is_some() {
            break;
        }
    }

    if response.is_none() {
        response = last_upstream_error;
    }
    if response.is_none() {
        if let Some(fallback) = truncated_fallback {
            response = Some(fallback);
        }
    }
    if let Some(mut resp) = response {
        // Put the CLIENT's transaction ID back — everything above spoke to the upstream
        // under a freshly randomised one. Done before the cache insert so a cache hit and a
        // fresh answer are byte-identical apart from the txid the hit path rewrites anyway.
        // (Audit 2026-08-04, M-10.)
        resp[0] = query_txid[0];
        resp[1] = query_txid[1];

        // Cache lifetime from the record itself, clamped. A TTL of 0 means "do not cache"
        // (RFC 2181 §8) and is honoured by skipping the insert entirely — it is used for
        // things like round-robin load balancing, where caching defeats the point. Negative
        // NXDOMAIN/NODATA responses use RFC 2308's min(SOA TTL, SOA.MINIMUM); without a valid
        // authority SOA they are not reusable. (S-14)
        let Some(entry_ttl) = response_cache_ttl(&resp) else {
            return Some(resp);
        };

        let cache_limit = cfg
            .cache_size
            .min(crate::config::server::DNS_MAX_CACHE_ENTRIES);
        if cache_limit == 0 {
            return Some(resp);
        }
        let mut cache_write = cache.write().await;
        if cache_write.len() >= cache_limit {
            // Drop expired entries first (cheap win). If the cache is still full of
            // FRESH entries, evict a batch of arbitrary keys so we make real room —
            // otherwise every insert at steady-state saturation would re-scan the
            // whole map (O(n)) and free nothing, stalling all DNS tasks. Batching
            // amortizes the scan over ~cache_size/10 inserts.
            let now = Instant::now();
            cache_write.retain(|_, (_, time, entry_ttl)| now.duration_since(*time) < *entry_ttl);
            if cache_write.len() >= cache_limit {
                let evict = (cache_limit / 10).max(1);
                let victims: Vec<_> = cache_write.keys().take(evict).cloned().collect();
                for k in victims {
                    cache_write.remove(&k);
                }
            }
        }
        if cache_write.len() < cache_limit {
            cache_write.insert(cache_key, (resp.clone(), Instant::now(), entry_ttl));
        }
        return Some(resp);
    }
    None
}

/// How large a reply the UPSTREAM may legitimately send us, which is what the client asked
/// for — its OPT record travels upstream verbatim — with a 4 KiB floor so the common case
/// keeps its old headroom, and the UDP maximum as the ceiling.
fn upstream_buf_size(query: &[u8]) -> usize {
    advertised_udp_size(query).clamp(4096, 65_535)
}

/// The UDP payload size the client said it can accept: its EDNS0 OPT record, or the 512-byte
/// floor from RFC 1035 §4.2.1 when it sent no OPT at all.
fn advertised_udp_size(query: &[u8]) -> usize {
    const FLOOR: usize = 512;
    // OPT lives in the ADDITIONAL section, and its CLASS field carries the payload size
    // (RFC 6891 §6.1.2) rather than a class. Walking there means stepping over every earlier
    // section, so a malformed query simply falls back to the floor.
    let Some(pos) = additional_section_start(query) else {
        return FLOOR;
    };
    let arcount = u16::from_be_bytes([query[10], query[11]]) as usize;
    let mut pos = pos;
    for _ in 0..arcount {
        // An OPT record's NAME is always root (a single 0 byte), but skip_name handles the
        // general case and keeps this honest against a compressed pointer.
        let after_name = match skip_name(query, pos) {
            Some(p) => p,
            None => return FLOOR,
        };
        if after_name + 10 > query.len() {
            return FLOOR;
        }
        let rtype = u16::from_be_bytes([query[after_name], query[after_name + 1]]);
        let class = u16::from_be_bytes([query[after_name + 2], query[after_name + 3]]);
        let rdlen = u16::from_be_bytes([query[after_name + 8], query[after_name + 9]]) as usize;
        let Some(record_end) = after_name
            .checked_add(10)
            .and_then(|header_end| header_end.checked_add(rdlen))
            .filter(|end| *end <= query.len())
        else {
            return FLOOR;
        };
        if rtype == 41 && query.get(pos) == Some(&0) {
            // Clamp: a peer may advertise anything, and a UDP datagram cannot exceed 65535
            // however large a number it writes here.
            return (class as usize).clamp(FLOOR, 65_535);
        }
        pos = record_end;
    }
    FLOOR
}

/// Offset of the first ADDITIONAL record, stepping over the question, answer and authority
/// sections. `None` on anything malformed.
fn additional_section_start(msg: &[u8]) -> Option<usize> {
    if msg.len() < 12 {
        return None;
    }
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let ancount = u16::from_be_bytes([msg[6], msg[7]]) as usize;
    let nscount = u16::from_be_bytes([msg[8], msg[9]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        pos = skip_name(msg, pos)?
            .checked_add(4)
            .filter(|end| *end <= msg.len())?; // QTYPE + QCLASS
    }
    for _ in 0..(ancount + nscount) {
        pos = skip_name(msg, pos)?;
        if pos.checked_add(10)? > msg.len() {
            return None;
        }
        let rdlen = u16::from_be_bytes([msg[pos + 8], msg[pos + 9]]) as usize;
        pos = pos
            .checked_add(10)?
            .checked_add(rdlen)
            .filter(|end| *end <= msg.len())?;
    }
    Some(pos)
}

/// Extended DNS RCODE from the single well-formed OPT pseudo-record, or zero when EDNS is
/// absent. Multiple OPT records and a non-root OPT owner name are malformed and reject cache
/// admission. The ordinary low four RCODE bits live in the base header.
fn dns_extended_rcode(msg: &[u8]) -> Option<u8> {
    let mut pos = additional_section_start(msg)?;
    let arcount = u16::from_be_bytes([msg[10], msg[11]]) as usize;
    let mut extended = None;
    for _ in 0..arcount {
        let name_start = pos;
        let after_name = skip_name(msg, pos)?;
        let header_end = after_name.checked_add(10).filter(|end| *end <= msg.len())?;
        let rtype = u16::from_be_bytes([msg[after_name], msg[after_name + 1]]);
        let rdlen = u16::from_be_bytes([msg[after_name + 8], msg[after_name + 9]]) as usize;
        pos = header_end
            .checked_add(rdlen)
            .filter(|end| *end <= msg.len())?;
        if rtype == 41 {
            if msg.get(name_start) != Some(&0) || extended.is_some() {
                return None;
            }
            extended = Some(msg[after_name + 4]);
        }
    }
    Some(extended.unwrap_or(0))
}

/// Cut a UDP reply down to what the client said it can take, setting TC so it knows to ask
/// again over TCP.
///
/// This proxy used to forward the answer WHOLE however large it was, because setting TC without
/// a TCP listener would have sent the client to a port where nothing answers — a working lookup
/// turned into a failing one. Now that the listener exists, TC means what it says.
///
/// The truncated message is header + question with all three record counts zeroed, not the
/// original bytes cut short: chopping mid-record leaves counts promising records that are not
/// there, which a resolver reads as a malformed message rather than as "retry over TCP".
pub(crate) fn apply_udp_size_limit(query: &[u8], resp: Vec<u8>) -> Vec<u8> {
    let limit = advertised_udp_size(query);
    if resp.len() <= limit {
        return resp;
    }
    // Question section only; if it cannot be located, fall back to a bare header.
    let q_end = question_section_end(query).unwrap_or(12).min(query.len());
    let mut out = Vec::with_capacity(q_end);
    out.extend_from_slice(&query[..q_end]);
    // The FLAGS come from the real answer, not from the query.
    //
    // Building them from the query and forcing NOERROR discarded what the resolver actually
    // said: an oversized NXDOMAIN went out as NOERROR+TC, and RA (recursion available), AA and
    // AD were lost with it. A stub that trusts the truncated header — many do, before deciding
    // whether the TCP retry is even worth it — was told the wrong thing about the answer it
    // was about to fetch. Copy bytes 2..4 from the response and set TC on top; only QR is
    // forced, because a response is what this is. (Audit 2026-08-01, §10.)
    if resp.len() >= 4 {
        out[2] = resp[2];
        out[3] = resp[3];
    }
    out[2] |= 0x80; // QR: this is a response
    out[2] |= 0x02; // TC: truncated — ask again over TCP
                    // QDCOUNT is left alone on purpose — the question IS carried, and a resolver matches the
                    // truncated reply to its outstanding query by it.
    out[6..8].copy_from_slice(&0u16.to_be_bytes()); // ANCOUNT
    out[8..10].copy_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    out[10..12].copy_from_slice(&0u16.to_be_bytes()); // ARCOUNT (the OPT is dropped with it)
    out
}

/// Offset just past the question section.
/// Is this upstream reply actually the answer to the query we sent?
///
/// Checks the transaction ID AND that the QUESTION section comes back byte-identical.
/// The txid alone is 16 bits; without the question match an attacker spraying answers can
/// win the birthday race with a reply about a name we never asked for and still have it
/// cached under our key. Both are cheap and neither is optional. (Audit 2026-08-04, M-10.)
fn response_matches(resp: &[u8], txid: [u8; 2], query: &[u8]) -> bool {
    if resp.len() < 12 || resp[0] != txid[0] || resp[1] != txid[1] {
        return false;
    }
    // QR bit must be set — a reply, not a reflected query.
    if resp[2] & 0x80 == 0 {
        return false;
    }
    match (question_section_end(query), question_section_end(resp)) {
        (Some(qe), Some(re)) => {
            // QDCOUNT must agree too, else "no question" trivially matches "no question".
            query[4..6] == resp[4..6] && qe == re && query[12..qe] == resp[12..re]
        }
        _ => false,
    }
}

fn question_section_end(msg: &[u8]) -> Option<usize> {
    if msg.len() < 12 {
        return None;
    }
    let qdcount = u16::from_be_bytes([msg[4], msg[5]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        pos = skip_name(msg, pos)?
            .checked_add(4)
            .filter(|end| *end <= msg.len())?;
    }
    Some(pos)
}

/// Construct a self-contained NXDOMAIN response for a blocked name.
///
/// Copying the whole request and only changing FLAGS also copied any client-supplied
/// ANSWER/AUTHORITY/ADDITIONAL records while their counts still claimed they were part of
/// our response. Besides reflecting arbitrary payload, that produced contradictory NXDOMAIN
/// packets. Keep only the validated question and explicitly clear every resource-record count.
fn blocked_response(query: &[u8]) -> Option<Vec<u8>> {
    let q_end = question_section_end(query)?;
    let mut response = query[..q_end].to_vec();
    // Preserve OPCODE and RD from the request. This proxy provides recursion (RA), does not
    // authenticate the synthetic answer (AD=0), and returns NXDOMAIN.
    response[2] = 0x80 | (query[2] & 0x79);
    response[3] = 0x80 | (query[3] & 0x10) | 0x03;
    response[6..8].copy_from_slice(&0u16.to_be_bytes());
    response[8..10].copy_from_slice(&0u16.to_be_bytes());
    response[10..12].copy_from_slice(&0u16.to_be_bytes());
    Some(response)
}

pub(crate) fn compile_blocklist(raw: &[String]) -> DnsBlocklist {
    Arc::new(
        raw.iter()
            .filter_map(|domain| crate::config::server::normalize_blocklist_domain(domain))
            .collect(),
    )
}

fn is_blocked(query: &[u8], blocklist: &HashSet<String>) -> bool {
    if blocklist.is_empty() {
        return false;
    }

    let Some(domain) = first_question_name(query) else {
        return false;
    };
    let mut candidate = domain.as_str();
    loop {
        if blocklist.contains(candidate) {
            return true;
        }
        let Some(dot) = candidate.find('.') else {
            return false;
        };
        candidate = &candidate[dot + 1..];
    }
}

/// Decode the first question name, including legal compression pointers, with strict loop and
/// length bounds. Blocklist matching must not interpret bytes from QTYPE, EDNS or another
/// section as labels when a client sends malformed framing.
fn first_question_name(query: &[u8]) -> Option<String> {
    if query.len() < 12 || u16::from_be_bytes([query[4], query[5]]) == 0 {
        return None;
    }
    let mut labels = Vec::new();
    walk_name(query, 12, |label| labels.push(label))?;
    let mut domain = String::from_utf8(labels.join(&b"."[..])).ok()?;
    domain.make_ascii_lowercase();
    Some(domain)
}

#[cfg(test)]
mod tests {
    //! Coverage for the answer-TTL parser (S-14). It walks attacker-influenced bytes —
    //! an upstream reply is untrusted input — so the cases that matter are the malformed
    //! ones: it must return None, never panic or loop.
    use super::*;

    fn cname_response(ttl: u32) -> Vec<u8> {
        let mut msg = response(&[], false);
        msg[7] = 1;
        msg.extend_from_slice(&[0xc0, 0x0c, 0, 5, 0, 1]);
        msg.extend_from_slice(&ttl.to_be_bytes());
        msg.extend_from_slice(&[0, 8, 5, b'a', b'l', b'i', b'a', b's', 0xc0, 0x0c]);
        msg
    }

    #[test]
    fn audit_cname_nodata_cannot_outlive_the_negative_soa() {
        for (alias, soa, minimum, expected) in [
            (300, 120, 5, Some(5)),
            (3, 120, 5, Some(3)),
            (300, 120, 0, None),
        ] {
            let msg = with_soa(cname_response(alias), soa, minimum);
            assert_eq!(response_cache_ttl(&msg), expected.map(Duration::from_secs));
        }
    }

    #[test]
    fn audit_compression_cycles_are_not_complete_dns_messages() {
        let mut msg = response(&[60], true);
        let owner = question_section_end(&msg).unwrap();
        msg[owner..owner + 2].copy_from_slice(&[0xc0, owner as u8]);
        assert!(!dns_message_is_complete(&msg));
        assert_eq!(response_cache_ttl(&msg), None);
    }

    #[test]
    fn audit_dns_names_cannot_expand_beyond_255_bytes() {
        let mut msg = query(None)[..12].to_vec();
        for _ in 0..4 {
            msg.push(63);
            msg.extend_from_slice(&[b'a'; 63]);
        }
        msg.extend_from_slice(&[0, 0, 1, 0, 1]);
        assert!(!dns_message_is_complete(&msg));
    }

    #[test]
    fn aliases_require_negative_proof_unless_the_requested_rrset_is_present() {
        let alias = cname_response(60);
        assert_eq!(
            response_cache_ttl(&alias),
            None,
            "unresolved A lookup has no negative SOA"
        );
        let mut cname_query = alias.clone();
        let question_end = question_section_end(&alias).unwrap();
        cname_query[question_end - 4..question_end - 2].copy_from_slice(&5u16.to_be_bytes());
        assert_eq!(
            response_cache_ttl(&cname_query),
            Some(Duration::from_secs(60))
        );
        let mut resolved = alias;
        resolved[7] = 2;
        resolved.extend_from_slice(&[0xc0, (question_end + 12) as u8, 0, 1, 0, 1]);
        resolved.extend_from_slice(&30u32.to_be_bytes());
        resolved.extend_from_slice(&[0, 4, 192, 0, 2, 80]);
        assert_eq!(response_cache_ttl(&resolved), Some(Duration::from_secs(30)));
    }

    #[test]
    fn name_walker_accepts_boundaries_and_nested_backward_pointers() {
        let mut boundary = query(None)[..12].to_vec();
        for length in [63, 63, 63, 61] {
            boundary.push(length);
            boundary.extend(std::iter::repeat_n(b'a', usize::from(length)));
        }
        boundary.extend_from_slice(&[0, 0, 1, 0, 1]);
        assert!(dns_message_is_complete(&boundary));
        assert!(first_question_name(&boundary).is_some());
        // 127 one-byte labels plus root is also a legal 255-byte name. Following
        // the answer's compression pointer adds a hop without adding name bytes.
        let mut many_labels = query(None)[..12].to_vec();
        for _ in 0..127 {
            many_labels.extend_from_slice(&[1, b'a']);
        }
        many_labels.extend_from_slice(&[0, 0, 1, 0, 1]);
        many_labels[2] = 0x81;
        many_labels[3] = 0x80;
        many_labels[7] = 1;
        many_labels.extend_from_slice(&[0xc0, 12, 0, 1, 0, 1, 0, 0, 0, 60, 0, 4, 192, 0, 2, 80]);
        assert_eq!(
            response_cache_ttl(&many_labels),
            Some(Duration::from_secs(60))
        );
        let mut nested = response(&[60, 30], true);
        let first = question_section_end(&nested).unwrap();
        nested[first + 16..first + 18].copy_from_slice(&[0xc0, first as u8]);
        assert!(dns_message_is_complete(&nested));
        assert_eq!(response_cache_ttl(&nested), Some(Duration::from_secs(30)));
        for target in [0, 11, first + 16] {
            let mut bad = nested.clone();
            bad[first..first + 2].copy_from_slice(&[0xc0, target as u8]);
            assert!(!dns_message_is_complete(&bad));
        }
    }

    #[tokio::test]
    async fn cached_queries_age_ttls_restore_ids_and_obey_the_current_blocklist() {
        let cfg = Arc::new(DnsConfig {
            upstream: Vec::new(),
            ..serde_json::from_str("{}").unwrap()
        });
        let cache: DnsCache = Arc::new(RwLock::new(HashMap::new()));
        let pref = Arc::new(AtomicUsize::new(0));
        let mut request = query(None);
        request[..2].copy_from_slice(&[0x12, 0x34]);
        let mut key = request.clone();
        key[..2].fill(0);
        cache.write().await.insert(
            key.clone(),
            (
                response(&[60], true),
                Instant::now() - Duration::from_secs(10),
                Duration::from_secs(60),
            ),
        );
        let hit = resolve(
            cache.clone(),
            cfg.clone(),
            pref.clone(),
            Arc::new(HashSet::new()),
            &request,
        )
        .await
        .unwrap();
        assert_eq!(&hit[..2], &[0x12, 0x34]);
        assert!(answer_min_ttl(&hit).is_some_and(|ttl| ttl <= 50));
        let blocked = resolve(
            cache.clone(),
            cfg.clone(),
            pref.clone(),
            compile_blocklist(&["example.com".into()]),
            &request,
        )
        .await
        .unwrap();
        assert_eq!(blocked[3] & 15, 3);
        cache.write().await.get_mut(&key).unwrap().1 = Instant::now() - Duration::from_secs(61);
        assert!(
            resolve(cache, cfg, pref, Arc::new(HashSet::new()), &request)
                .await
                .is_none()
        );
    }

    struct TestUpstream {
        address: SocketAddr,
        task: tokio::task::JoinHandle<()>,
    }

    impl Drop for TestUpstream {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn test_upstream(udp: Option<Vec<u8>>, tcp: Option<Vec<u8>>) -> TestUpstream {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let socket = UdpSocket::bind(address).await.unwrap();
        let task = tokio::spawn(async move {
            if let Some(mut reply) = udp {
                let mut input = vec![0; 65535];
                let (n, peer) = socket.recv_from(&mut input).await.unwrap();
                assert!(n >= 12);
                reply[..2].copy_from_slice(&input[..2]);
                socket.send_to(&reply, peer).await.unwrap();
            }
            if let Some(mut reply) = tcp {
                let (mut stream, _) = listener.accept().await.unwrap();
                let length = stream.read_u16().await.unwrap();
                let mut input = vec![0; usize::from(length)];
                stream.read_exact(&mut input).await.unwrap();
                reply[..2].copy_from_slice(&input[..2]);
                stream
                    .write_u16(u16::try_from(reply.len()).unwrap())
                    .await
                    .unwrap();
                stream.write_all(&reply).await.unwrap();
            }
        });
        TestUpstream { address, task }
    }

    async fn check_truncated_tcp_failover(force_tcp: bool) {
        let mut truncated = response(&[], true);
        truncated[2] |= 2;
        let first = test_upstream((!force_tcp).then(|| truncated.clone()), Some(truncated)).await;
        let good = response(&[60], true);
        let second = test_upstream(
            (!force_tcp).then(|| good.clone()),
            force_tcp.then_some(good.clone()),
        )
        .await;
        let cfg = Arc::new(DnsConfig {
            upstream_protocol: if force_tcp { "tcp" } else { "udp" }.into(),
            timeout_secs: 2,
            cache_size: 8,
            ..serde_json::from_str("{}").unwrap()
        });
        let cache: DnsCache = Arc::new(RwLock::new(HashMap::new()));
        let pref = Arc::new(AtomicUsize::new(0));
        let mut request = query(None);
        request[..2].copy_from_slice(&[0x12, 0x34]);
        let answer = tokio::time::timeout(
            Duration::from_secs(3),
            resolve_with_upstreams(
                cache.clone(),
                cfg,
                pref.clone(),
                Arc::new(HashSet::new()),
                &request,
                &[first.address, second.address],
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            answer[2] & 2,
            0,
            "truncated TCP answer must not suppress a healthy fallback"
        );
        assert_eq!(&answer[..2], &[0x12, 0x34]);
        assert_eq!(&answer[2..], &good[2..]);
        assert_eq!(pref.load(Ordering::Relaxed), 1);
        assert_eq!(cache.read().await.len(), 1);
    }

    #[tokio::test]
    async fn audit_forced_tcp_tries_next_upstream_after_tc() {
        check_truncated_tcp_failover(true).await;
    }

    #[tokio::test]
    async fn audit_udp_retry_tries_next_upstream_after_tcp_tc() {
        check_truncated_tcp_failover(false).await;
    }

    #[tokio::test]
    async fn a_silent_preferred_upstream_leaves_time_for_the_fallback() {
        let silent = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let good = test_upstream(Some(response(&[60], true)), None).await;
        let cfg = Arc::new(DnsConfig {
            timeout_secs: 1,
            upstream_protocol: "udp".into(),
            ..serde_json::from_str("{}").unwrap()
        });
        let cache: DnsCache = Arc::new(RwLock::new(HashMap::new()));
        let pref = Arc::new(AtomicUsize::new(0));
        let answer = tokio::time::timeout(
            Duration::from_secs(3),
            resolve_with_upstreams(
                cache,
                cfg,
                pref.clone(),
                Arc::new(HashSet::new()),
                &query(None),
                &[silent.local_addr().unwrap(), good.address],
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(answer[7], 1);
        assert_eq!(pref.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn a_partial_tcp_frame_is_bounded_by_the_exchange_deadline() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let length = stream.read_u16().await.unwrap();
            let mut input = vec![0; usize::from(length)];
            stream.read_exact(&mut input).await.unwrap();
            stream.write_all(&[0, 20, 0, 0]).await.unwrap();
            std::future::pending::<()>().await;
        });
        let _upstream = TestUpstream { address, task };
        assert!(tokio::time::timeout(
            Duration::from_secs(2),
            query_tcp(
                &address.to_string(),
                &query(None),
                Duration::from_millis(100)
            )
        )
        .await
        .unwrap()
        .is_none());
    }

    /// An upstream reply is only ours if the txid AND the question both match.
    /// (Audit 2026-08-04, M-10.)
    #[test]
    fn response_matches_requires_txid_and_question() {
        let mut query = response(&[], false); // reuse the builder: 1 question, no answers
        query[2] = 0x01; // a query, not a response (RD set, QR clear)
        query[3] = 0x00;
        let good = response(&[300], false);
        let txid = [0xAB, 0xCD];

        assert!(
            response_matches(&good, txid, &query),
            "the real answer must pass"
        );

        // Wrong transaction ID.
        let mut wrong_txid = good.clone();
        wrong_txid[0] = 0x00;
        assert!(!response_matches(&wrong_txid, txid, &query));

        // Right txid, DIFFERENT question — the birthday-spray case the question match
        // exists to stop.
        let mut other = good.clone();
        other[13] = b'X'; // "example" -> "Xxample"
        assert!(!response_matches(&other, txid, &query));

        // Right txid and question, but QR clear — a reflected query, not an answer.
        let mut reflected = good.clone();
        reflected[2] &= 0x7F;
        assert!(!response_matches(&reflected, txid, &query));

        // Truncated to the header, and to nothing.
        assert!(!response_matches(&good[..11], txid, &query));
        assert!(!response_matches(&[], txid, &query));

        // A reply claiming a different QDCOUNT must not pass.
        let mut qd = good.clone();
        qd[5] = 2;
        assert!(!response_matches(&qd, txid, &query));
    }

    #[test]
    fn blocklist_matching_is_bounded_and_nxdomain_strips_request_records() {
        let q = query(Some(1232));
        let exact = compile_blocklist(&["EXAMPLE.COM.".to_string()]);
        assert!(is_blocked(&q, &exact));
        let not_a_label_suffix = compile_blocklist(&["ample.com".to_string()]);
        assert!(!is_blocked(&q, &not_a_label_suffix));

        // A compression loop is untrusted input, not a name to keep decoding forever.
        let mut looped = q.clone();
        looped[12..14].copy_from_slice(&[0xC0, 0x0C]);
        assert!(!is_blocked(&looped, &exact));

        // Even if the client supplied resource records, a synthetic NXDOMAIN carries only
        // the validated question. Reflecting those bytes as part of our answer would produce
        // contradictory counts and replay attacker-controlled payload.
        let mut with_answer = response(&[60], true);
        with_answer[2] &= 0x7F; // make it a request-shaped message
        let q_end = question_section_end(&with_answer).unwrap();
        let blocked = blocked_response(&with_answer).unwrap();
        assert_eq!(blocked.len(), q_end);
        assert_eq!(&blocked[12..], &with_answer[12..q_end]);
        assert_eq!(blocked[2] & 0x80, 0x80);
        assert_eq!(blocked[3] & 0x0F, 3);
        assert_eq!(&blocked[6..12], &[0, 0, 0, 0, 0, 0]);
    }

    /// Build a minimal DNS response: one question, `answers` A-records with the given TTLs.
    fn response(ttls: &[u32], compressed_names: bool) -> Vec<u8> {
        let mut m = vec![0u8; 12];
        m[0] = 0xAB;
        m[1] = 0xCD; // txid
        m[2] = 0x81;
        m[3] = 0x80; // response, no error
        m[4] = 0;
        m[5] = 1; // QDCOUNT = 1
        m[6] = 0;
        m[7] = ttls.len() as u8; // ANCOUNT
                                 // Question: "example.com" A IN
        m.extend_from_slice(&[7]);
        m.extend_from_slice(b"example");
        m.extend_from_slice(&[3]);
        m.extend_from_slice(b"com");
        m.push(0);
        m.extend_from_slice(&[0, 1, 0, 1]); // QTYPE=A, QCLASS=IN
        for ttl in ttls {
            if compressed_names {
                m.extend_from_slice(&[0xC0, 0x0C]); // pointer back to the question name
            } else {
                m.extend_from_slice(&[7]);
                m.extend_from_slice(b"example");
                m.extend_from_slice(&[3]);
                m.extend_from_slice(b"com");
                m.push(0);
            }
            m.extend_from_slice(&[0, 1, 0, 1]); // TYPE=A, CLASS=IN
            m.extend_from_slice(&ttl.to_be_bytes());
            m.extend_from_slice(&[0, 4]); // RDLENGTH
            m.extend_from_slice(&[93, 184, 216, 34]); // RDATA
        }
        m
    }

    fn with_soa(mut m: Vec<u8>, soa_ttl: u32, minimum: u32) -> Vec<u8> {
        m[9] = 1; // NSCOUNT = 1
        m.extend_from_slice(&[0xC0, 0x0C]); // owner = question name
        m.extend_from_slice(&[0, 6, 0, 1]); // TYPE=SOA, CLASS=IN
        m.extend_from_slice(&soa_ttl.to_be_bytes());
        m.extend_from_slice(&22u16.to_be_bytes());
        m.push(0); // MNAME = root
        m.push(0); // RNAME = root
        for value in [1, 3600, 600, 86_400, minimum] {
            m.extend_from_slice(&value.to_be_bytes());
        }
        m
    }

    fn negative_response(rcode: u8, soa_ttl: u32, minimum: u32) -> Vec<u8> {
        let mut m = response(&[], false);
        m[3] = (m[3] & 0xF0) | (rcode & 0x0F);
        with_soa(m, soa_ttl, minimum)
    }

    #[test]
    fn reads_the_smallest_answer_ttl() {
        assert_eq!(answer_min_ttl(&response(&[300], false)), Some(300));
        assert_eq!(answer_min_ttl(&response(&[300, 60, 900], false)), Some(60));
    }

    #[test]
    fn cache_admission_honours_short_ttls_and_rejects_tc() {
        assert_eq!(
            response_cache_ttl(&response(&[1], false)),
            Some(Duration::from_secs(1))
        );
        assert_eq!(
            response_cache_ttl(&response(&[4], false)),
            Some(Duration::from_secs(4))
        );
        assert_eq!(response_cache_ttl(&response(&[0], false)), None);
        let mut truncated = response(&[60], false);
        truncated[2] |= 0x02;
        assert_eq!(response_cache_ttl(&truncated), None);

        let mut malformed_positive = response(&[60], false);
        malformed_positive.truncate(malformed_positive.len() - 1);
        assert_eq!(response_cache_ttl(&malformed_positive), None);

        let no_answers = response(&[], false);
        assert_eq!(response_cache_ttl(&no_answers), None);
        let mut nxdomain = no_answers.clone();
        nxdomain[3] = (nxdomain[3] & 0xF0) | 3;
        assert_eq!(response_cache_ttl(&nxdomain), None);
        assert_eq!(
            response_cache_ttl(&negative_response(3, 300, 45)),
            Some(Duration::from_secs(45)),
            "negative TTL is min(SOA RR TTL, SOA.MINIMUM)"
        );
        assert_eq!(
            response_cache_ttl(&negative_response(0, 20, 300)),
            Some(Duration::from_secs(20)),
            "NODATA uses the same RFC 2308 SOA lifetime"
        );
        let mut cname_nxdomain = response(&[300], true);
        cname_nxdomain[3] = (cname_nxdomain[3] & 0xF0) | 3;
        assert_eq!(
            response_cache_ttl(&with_soa(cname_nxdomain, 120, 45)),
            Some(Duration::from_secs(45)),
            "NXDOMAIN with a CNAME cannot outlive its negative SOA TTL"
        );
        assert_eq!(response_cache_ttl(&negative_response(3, 300, 0)), None);
        for rcode in [1, 2, 5] {
            let mut error = no_answers.clone();
            error[3] = (error[3] & 0xF0) | rcode;
            assert!(response_is_retryable_error(&error));
            assert_eq!(
                response_cache_ttl(&error),
                None,
                "DNS error rcode {rcode} must not be cached"
            );
        }
        assert!(!response_is_retryable_error(&nxdomain));
        let mut trailing_garbage = response(&[60], false);
        trailing_garbage.push(0);
        assert_eq!(response_cache_ttl(&trailing_garbage), None);

        let mut badvers = query(Some(1232));
        badvers[2] = 0x81;
        badvers[3] = 0x80;
        let opt = additional_section_start(&badvers).unwrap();
        let after_name = skip_name(&badvers, opt).unwrap();
        badvers[after_name + 4] = 1; // extended RCODE 1 => BADVERS (full RCODE 16)
        assert!(response_is_retryable_error(&badvers));
        assert_eq!(response_cache_ttl(&badvers), None);
    }

    #[test]
    fn follows_compressed_names() {
        // The common real-world shape: answers point back at the question's name.
        assert_eq!(answer_min_ttl(&response(&[120, 45], true)), Some(45));
    }

    #[test]
    fn no_answers_yields_none() {
        // NXDOMAIN / NODATA — nothing to derive a lifetime from.
        assert_eq!(answer_min_ttl(&response(&[], false)), None);
    }

    #[test]
    fn malformed_input_never_panics() {
        // Truncated at every possible length: each must be rejected, not crash.
        let full = response(&[300, 60], true);
        for cut in 0..full.len() {
            let _ = answer_min_ttl(&full[..cut]);
        }
        // Header claims answers that are not there.
        let mut lying = response(&[300], false);
        lying[7] = 200;
        assert_eq!(answer_min_ttl(&lying), None);
        // A name length that runs past the buffer.
        let mut runaway = response(&[300], false);
        let qname = 12;
        runaway[qname] = 0xFF;
        assert_eq!(answer_min_ttl(&runaway), None);
        // Compression pointer loop: must terminate (the pointer is not followed, so this
        // is really a check that a pointer always ends the name walk).
        let mut looped = response(&[300], true);
        looped[12] = 0xC0;
        looped[13] = 0x0C;
        let _ = answer_min_ttl(&looped);
        assert_eq!(answer_min_ttl(&[]), None);
        assert_eq!(answer_min_ttl(&[0u8; 11]), None);
    }

    #[test]
    fn ttl_zero_is_distinguishable() {
        // Some(0) must survive to the caller so it can skip caching entirely.
        assert_eq!(answer_min_ttl(&response(&[0], false)), Some(0));
    }

    /// A query with no OPT record, and one advertising `payload` bytes via EDNS0.
    fn query(payload: Option<u16>) -> Vec<u8> {
        let mut m = vec![0u8; 12];
        m[0] = 0xAB;
        m[1] = 0xCD;
        m[5] = 1; // QDCOUNT = 1
        m.extend_from_slice(&[7]);
        m.extend_from_slice(b"example");
        m.extend_from_slice(&[3]);
        m.extend_from_slice(b"com");
        m.push(0);
        m.extend_from_slice(&[0, 1, 0, 1]); // QTYPE=A, QCLASS=IN
        if let Some(size) = payload {
            m[11] = 1; // ARCOUNT = 1
            m.push(0); // OPT NAME = root
            m.extend_from_slice(&[0, 41]); // TYPE = OPT (41)
            m.extend_from_slice(&size.to_be_bytes()); // CLASS carries the payload size
            m.extend_from_slice(&[0, 0, 0, 0]); // TTL (extended rcode + flags)
            m.extend_from_slice(&[0, 0]); // RDLENGTH
        }
        m
    }

    /// The size a client says it can take governs whether the answer is truncated, and RFC 1035
    /// §4.2.1's 512 is the floor when it says nothing at all.
    #[test]
    fn the_advertised_udp_size_comes_from_the_opt_record() {
        assert_eq!(advertised_udp_size(&query(None)), 512);
        assert_eq!(advertised_udp_size(&query(Some(1232))), 1232);
        // Clamped at both ends: a peer may advertise anything, and a UDP datagram cannot
        // exceed 65535 however large a number it writes.
        assert_eq!(advertised_udp_size(&query(Some(128))), 512);
        assert_eq!(advertised_udp_size(&query(Some(65535))), 65_535);

        // The receive buffer follows the advertisement, because `recv_from` silently discards
        // whatever does not fit — a client that asked for 8192 used to have its answer chopped
        // at 4096 and forwarded malformed. The floor keeps the common case unchanged.
        assert_eq!(upstream_buf_size(&query(None)), 4096);
        assert_eq!(upstream_buf_size(&query(Some(1232))), 4096);
        assert_eq!(upstream_buf_size(&query(Some(8192))), 8192);
        assert_eq!(upstream_buf_size(&query(Some(65535))), 65_535);
        // Malformed input falls back to the floor rather than panicking.
        let full = query(Some(4096));
        for cut in 0..full.len() {
            let _ = advertised_udp_size(&full[..cut]);
        }
        let mut truncated_opt = full;
        let opt = additional_section_start(&truncated_opt).unwrap();
        let after_name = skip_name(&truncated_opt, opt).unwrap();
        truncated_opt[after_name + 8..after_name + 10].copy_from_slice(&4u16.to_be_bytes());
        truncated_opt.extend_from_slice(&[1, 2, 3]); // one byte short of the declared RDATA
        assert_eq!(advertised_udp_size(&truncated_opt), 512);
    }

    /// An answer that fits is forwarded untouched; one that does not comes back as a TC=1
    /// header plus the question, NOT as the original bytes cut short.
    ///
    /// Chopping mid-record would leave the counts promising records that are not present, which
    /// a resolver reads as a malformed message rather than as "retry over TCP" — and retrying
    /// over TCP is the whole point, now that there is a TCP listener to retry against.
    /// (Audit 2026-08-01, §10.)
    #[test]
    fn an_oversized_answer_is_truncated_with_tc_set() {
        let q = query(Some(512));
        // Fits: byte-for-byte the same object comes back.
        let small = response(&[300], true);
        assert!(small.len() <= 512);
        assert_eq!(apply_udp_size_limit(&q, small.clone()), small);

        // Does not fit: 40 A-records is well past 512 bytes.
        let big = response(&[300; 40], true);
        assert!(big.len() > 512);
        let out = apply_udp_size_limit(&q, big);
        assert!(
            out.len() <= 512,
            "the reply must fit what the client advertised"
        );
        assert_eq!(&out[0..2], &q[0..2], "the txid must match the query");
        assert_eq!(out[2] & 0x80, 0x80, "QR must say this is a response");
        assert_eq!(out[2] & 0x02, 0x02, "TC must be set");
        assert_eq!(out[3] & 0x0F, 0, "a NOERROR answer stays NOERROR");
        assert_eq!(
            u16::from_be_bytes([out[4], out[5]]),
            1,
            "the question is carried, so QDCOUNT stays 1"
        );
        for (label, off) in [("ANCOUNT", 6), ("NSCOUNT", 8), ("ARCOUNT", 10)] {
            assert_eq!(
                u16::from_be_bytes([out[off], out[off + 1]]),
                0,
                "{label} must be zero — no records are carried"
            );
        }
        // The question section itself survives intact, so a resolver can match the reply.
        assert_eq!(&out[12..], &q[12..12 + (out.len() - 12)]);

        // A client that advertised room for it gets the whole thing instead.
        let big = response(&[300; 40], true);
        let roomy = query(Some(4096));
        assert!(big.len() <= 4096);
        assert_eq!(apply_udp_size_limit(&roomy, big.clone()), big);
    }

    /// The truncated reply must carry the ANSWER's flags, not the query's.
    ///
    /// They were rebuilt from the query with RCODE forced to NOERROR, so an oversized NXDOMAIN
    /// went out as NOERROR+TC and RA/AA/AD were lost with it — a stub that reads the truncated
    /// header before deciding whether the TCP retry is worth it was told the wrong thing.
    /// (Audit 2026-08-01, §10.)
    #[test]
    fn a_truncated_reply_keeps_the_answers_rcode_and_flags() {
        let q = query(Some(512));
        let mut big = response(&[300; 40], true);
        assert!(big.len() > 512);
        big[2] = 0x85; // QR + AA + RD
        big[3] = 0x83; // RA + NXDOMAIN (rcode 3)

        let out = apply_udp_size_limit(&q, big);
        assert_eq!(out[3] & 0x0F, 3, "NXDOMAIN must survive truncation");
        assert_eq!(out[3] & 0x80, 0x80, "RA must survive");
        assert_eq!(out[2] & 0x04, 0x04, "AA must survive");
        assert_eq!(out[2] & 0x80, 0x80, "QR is forced — this IS a response");
        assert_eq!(out[2] & 0x02, 0x02, "TC must be set");
    }

    /// A cached answer must be served with the REMAINING lifetime.
    ///
    /// Only the transaction ID used to be rewritten, so a reply served a second before its
    /// cache entry expired still claimed the full original TTL — and every layer that caches
    /// on that number compounds the staleness (RFC 2181 §5.2). The OPT pseudo-record is the
    /// exception: its "TTL" is the extended RCODE and flags, so ageing it would corrupt the
    /// answer rather than expire it. (Audit 2026-08-01, §10.)
    #[test]
    fn cached_answers_are_served_with_the_remaining_ttl() {
        fn ttls_of(msg: &[u8]) -> Vec<u32> {
            let qd = u16::from_be_bytes([msg[4], msg[5]]) as usize;
            let an = u16::from_be_bytes([msg[6], msg[7]]) as usize;
            let mut pos = 12;
            for _ in 0..qd {
                pos = skip_name(msg, pos).unwrap() + 4;
            }
            let mut out = Vec::new();
            for _ in 0..an {
                let p = skip_name(msg, pos).unwrap();
                out.push(u32::from_be_bytes([
                    msg[p + 4],
                    msg[p + 5],
                    msg[p + 6],
                    msg[p + 7],
                ]));
                pos = p + 10 + u16::from_be_bytes([msg[p + 8], msg[p + 9]]) as usize;
            }
            out
        }

        let mut msg = response(&[300, 60], true);
        decrement_ttls(&mut msg, 45);
        assert_eq!(
            ttls_of(&msg),
            vec![255, 15],
            "TTLs must age by the time cached"
        );

        // Saturating, never wrapping: an entry older than its record reads as 0, not as ~4e9.
        let mut expired = response(&[10], true);
        decrement_ttls(&mut expired, 5_000);
        assert_eq!(ttls_of(&expired), vec![0]);

        // Zero age is a no-op, and a malformed message must not panic.
        let untouched = response(&[300], true);
        let mut same = untouched.clone();
        decrement_ttls(&mut same, 0);
        assert_eq!(same, untouched);
        for cut in 0..untouched.len() {
            let mut t = untouched[..cut].to_vec();
            decrement_ttls(&mut t, 10);
        }

        // An OPT record's TTL field carries the extended RCODE and flags — ageing it would
        // corrupt the answer. Append one and check it comes back byte-for-byte.
        let mut with_opt = response(&[300], true);
        with_opt[11] = 1; // ARCOUNT = 1
        let opt_at = with_opt.len();
        // NAME(1: root) TYPE(2: 41) CLASS(2: 4096 payload) TTL(4: ext-rcode+flags) RDLEN(2)
        with_opt.extend_from_slice(&[0, 0, 41, 0x10, 0, 0x80, 0, 0, 0, 0, 0]);
        decrement_ttls(&mut with_opt, 100);
        assert_eq!(
            &with_opt[opt_at + 5..opt_at + 9],
            &[0x80, 0, 0, 0],
            "the OPT record's flags must not be decremented"
        );
    }
}
