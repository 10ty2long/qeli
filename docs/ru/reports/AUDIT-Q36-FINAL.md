# Q36 — итог аудита бенчмарков

<!-- normative-sync: audit-q36-final-v1 -->

**DONE в согласованном объёме, 14 октября 2026.**34/37 (91,9%), остаютсяQ29/Q31/Q34. Это завершение проверки методики, текущих ограниченных измерений и обнаруженных дефектов; не гарантия заданной пропускной способности на любом оборудовании.

| Критерий | Результат и evidence |
|---|---|
| Review/мёртвый код | F001–F032, шесть perf-реализаций выведены из эксплуатации; Windows/Linux проверки отказов и raw phase independence PASS |
| Штатные/граничные/негативные | Current CLI all12 modes,36sessions/144windows, по3 повтора; production pool/queue contract11tests и2negative controls |
| Отказы/конкуренция | Initial FAIL сохранены; PID/generation/netns/full host/SHA archive guards, queue Full и возврат всех pool slots проверены |
| Интеграция/платформа | Current native/ABI/JNI consumers, Linux20/409assertions; kernel FULL_RING reason; private exact-header queue stages + независимый shipping ACK/policy control |
| Fix/retest/evidence | F030 ядро/native и свежие проверки; F031/F032 обе ОС; FIFO/batching tradeoff описан без непроверенной смены defaults |

## Подтверждённый механизм и границы

В legacy ACK uplink и downlink data проходят общую серверную per-stream FIFO. На private source fork во всех трёх policy-парах MUX снижает p50 queue residence на98–99,8%; stock CLI отдельно подтверждает median uplink2,74× и ACKp50−76,4% при unlimited downlink. Эти серии не объединены: internal residence values относятся к диагностическому артефакту. При100M batching может увеличить медиану ACK из-за deadline. Подробные таблицы: [TUN/ACK](AUDIT-Q36-TUN-STAGE.md).

Kernel TUN FULL_RING и legacy QueueFull при перегрузке остаются документированными ограничениями; увеличение очереди4000 и writer nice не приняты как fixes. Ни zero loss, ни observer neutrality, ни DPI equivalence, ни capacity budget не заявлены. PCAP delays относятся к matching headers, пропуски/повторы сохранены. Исторические IPv6/old-source/failed cohorts не переименованы в текущие; новые native builds не запускались.

Полная матрица пяти критериев, SHA входов и ограничения: `release/certification/evidence/q36-final-20261014.json`. Shipping control: `release/certification/evidence/q36-recordizer-shipping-20261014.json`; private control: `release/certification/evidence/q36-recordizer-handoff-20261014.json`. Общее ядро/native не изменено, все own ресурсы лабы очищены. NO_PUSH_DEPLOY_RELEASE.
