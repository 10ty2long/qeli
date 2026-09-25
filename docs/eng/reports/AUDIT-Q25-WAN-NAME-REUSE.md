# Q25-A125: WAN name reuse

Date: 25 September 2026. Base: `37104b1e`. D06 remains **IN_PROGRESS**.
Priority: P2 (egress through an unselected physical interface; the client address is NATed).

Exit-node MARK, MASQUERADE, and FORWARD permits use `-o <wan>`. The remembered
WAN is a name, not a continuous network-device identity.
[Netfilter documentation](https://www.netfilter.org/documentation/HOWTO/packet-filtering-HOWTO-7.html)
defines `-o` as an interface-name match. The
[nftables manual](https://netfilter.org/projects/nftables/manpage.html)
distinguishes `meta oif` (interface index) from `meta oifname` (name), explicitly
noting that a name rule matches a new interface created with the old name.

Three states were tested on isolated lab server .11 inside private network,
mount, and PID namespaces. The original `wan0` had ifindex 5 and emitted a
packet with NAT source `198.51.100.2`. After replacing the route with `wan1`
(ifindex 7), the guard dropped the packet: its counter reached 1 and the sink
saw no packet. The **new** interface, still ifindex 7, was then given the name
`wan0` and a default route, without a Qeli refresh. The old MARK/MASQUERADE
rules matched again, and the sink received `192.0.2.2 > 203.0.113.9`.
The client address `10.0.0.2` did not escape, but traffic used a different
physical device without WAN requalification. A separate live rename from
`wan0` to `wan-old` preserved both ifindex and the default route under the
new name; the old `-o wan0` selector no longer matched.

Evidence: `C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260925/tun-attach-context-phase/wanrename3.log`
and `wanliverename.log`, with `wan-rename-reuse.sh` in the same directory.
Only private namespaces were changed; installed lab services were untouched.

**Contract until the backend is fixed:** do not rename, remove, or replace the
selected WAN while an exit-node profile is active. Stop the profile, verify
successful rule cleanup, change the interface, and then start the profile.
A route change to a distinct interface name is blocked until successful
refresh; reuse of the old name after its former device leaves is not a
protected scenario. The manual now says this explicitly; this audit step
makes no Rust changes.

A complete fix needs to bind egress admission to device identity on each
packet and account for identifier reuse after deletion. `nft meta oif` narrows
simple rename/name-reuse exposure, but an interface index can itself be
reused. Moving an isolated rule to nft without mixed iptables/nft/firewalld
and cleanup validation is not a complete fix. Design and packet/recovery
coverage remain in D06/D10.
