# Q36 — benchmark audit completion

<!-- normative-sync: audit-q36-final-v1 -->

**DONE within agreed scope,14 October2026.**34/37(91.9%),Q29/Q31/Q34 remain. Methodology, current bounded measurements and discovered defects are audited; no hardware-independent throughput guarantee.

| Criterion | Result/evidence |
|---|---|
| Review/dead code | F001–F032 methodology review; six perf implementations retired; Windows/Linux refusal/phase independence checks PASS |
| Normal/boundary/negative | Current CLI all12 modes,36sessions/144windows,three repeats; production pool/queue contract11tests/two negative controls |
| Failure/concurrency | Original FAIL retained; PID/generation/netns/full-host/SHA archive guards, queue Full and all returned pool slots |
| Integration/platform | Current native/ABI/JNI consumers,Linux20/409assertions; kernel FULL_RING reason; private exact-header queue stages + independent shipping ACK/policy control |
| Fix/retest/evidence | F030 core/native/fresh validation; F031/F032 both OS; documented FIFO/batching tradeoff without unvalidated defaults |

## Confirmed mechanism and limits

Uplink ACKs and downlink data share legacy server per-stream FIFO. In three private-source policy pairs MUX lowers p50 queue residence98–99.8%; independent stock CLI confirms median uplink2.74×/ACKp50−76.4% under unlimited downlink. Populations remain separate: internal residence values belong to the diagnostic artifact. At100M batching deadlines can increase median ACK latency. [Actual tables](AUDIT-Q36-TUN-STAGE.md).

Kernel TUN FULL_RING and legacy QueueFull under overload are documented operating limits; queue4000 and writer nice not adopted as fixes. No zero-loss/observer-neutrality/DPI-equivalence/capacity-budget assertion. PCAP delays condition on matching headers; omissions/repetitions retained. Historical IPv6/old-source/failed cohorts preserve their identity; no new native build.

Five-criterion map/input SHAs/limits:`release/certification/evidence/q36-final-20261014.json`. Shipping:`release/certification/evidence/q36-recordizer-shipping-20261014.json`;private:`release/certification/evidence/q36-recordizer-handoff-20261014.json`. Product/native unchanged; owned lab resources removed. NO_PUSH_DEPLOY_RELEASE.
