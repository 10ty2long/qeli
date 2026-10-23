# Qeli 0.8.3 (beta) — network policy, NDP and client diagnostics

**Draft / Черновик. Not published / Не опубликован.**

## English

This update addresses TCP buffer limits on high-RTT paths and makes UDP/TUN
queue pressure easier to diagnose. It preserves explicit operator settings and
existing memory budgets.

- Server TCP buffers default to OS management (`perf.tcp.send_buffer_size = 0`,
  `perf.tcp.recv_buffer_size = 0`). Linux installation explicitly enables receive
  autotuning, verifies readback and retains rollback data. Existing manual values
  are preserved. New multi-profile examples and panel Quick Start use the same defaults.
- The internal UDP receive handoff stays at 128 datagrams per pump. New depth and
  wait metrics help locate pressure without unbounded queue growth. The existing
  bounded UDP socket-buffer controller and manual TUN settings remain available.
- TUN dispatch first attempts immediate delivery, retaining bounded backpressure
  when full. Local `tun-stats` separates pool waits, dispatch waits, read gaps and
  UDP batch-send time. Sustained overload and all TUN drops are not claimed fixed.
- A manual client tunnel MTU no longer disables outer UDP path probing.
  `mtu_probe = false` remains authoritative, with conservative fallback when probing fails.
- macOS ships separate self-contained `Qeli-macos-arm64.tar.gz` (Apple Silicon)
  and `Qeli-macos-x86_64.tar.gz` (Intel) archives, requiring macOS 15 or newer.
  The universal packager now refuses incompatible .NET payloads; lipo alone cannot
  combine architecture-specific managed DLLs. Native Rust core remains universal2.
- macOS fixes cover Apple Silicon `openat`, recovery of old root-owned lock-file
  permissions, launchctl service detection and disconnect/status error handling.
- Android Tink moves to 1.24.0; Rust libc moves to 0.2.191. TLS dependencies
  rustls-webpki 0.103.16 and webpki-roots 1.0.10 address the newly reported
  certificate-trust advisories RUSTSEC-2026-0382 and RUSTSEC-2026-0381.
- Setup, configuration examples and Docker buffer diagnostics are aligned;
  license inventories are bound to the updated dependency locks.


- Issue #69 follow-up: SNI validation now follows actual TLS/WebSocket use.
  Legacy UDP-obfs SNI no longer blocks a valid profile; unused SNI is omitted
  from generated links. Desktop TCP WebSocket exposes its HTTP Host override.
- `pull_routes=false` declines server routes in the shared core while preserving
  local routing, independent full-tunnel policy and protected tunnel DNS routes.
  All clients preserve it through INI/models/backups and new share links.
  Explicit `gateway=false` now survives Rust INI serialization.
- NDP proxy uses an assigned uplink source rather than the advertised target,
  rechecks ready-session ownership before sending, joins known solicited-node
  groups through MLD and emits bounded appearance announcements. `manual` keeps
  administrator-owned routing/firewall. No arbitrary NA flags or periodic flood.
- UDP diagnostics separate inner MTU, outer payload budget and fallback reason;
  counters report encrypted-record split attempts and fragments created, not loss.
- Windows adds structured IPv4/IPv6 copy in GUI/service mode, remembered editor
  dimensions and a bounded traffic graph. Optional `windows_disable_sharing=false`
  applies only to the created Qeli Wintun with GUID/LUID/readback checks.

Scope and checks: [Issue #69 implementation plan](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/certification/plans/ISSUE69_0.8.3.md).
Fit-path, periodic NDP and an adapter surviving Disconnect are separate optional
stages, not claims of this update. Real Windows adapter bindings and provider
MLD-snooping are not certified by mocked or namespace-only tests.

The recorded TCP A/B experiment used 1 or 8 independent clients, added RTT of
40/100 ms and identical manual/OS-managed buffer comparisons on a two-VM Linux lab.
One eight-client uplink case reached a median aggregate 2,456 Mbit/s with OS-managed
buffers versus 279 Mbit/s with fixed 262,144-byte buffers. This is the sum across
eight tunnels on a local lab path, not a single-client or Internet speed promise.
Results and method: [benchmark evidence](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/queue-tuning-20261021.json).

For an existing Linux TCP profile, set both server TCP buffer keys to `0`, check
`net.ipv4.tcp_moderate_rcvbuf=1` and appropriate `tcp_rmem`/`tcp_wmem` limits,
validate the configuration and restart/reconnect to create new sockets. The global
`tcp_mem` budget is preserved. Do not force reinstall just to change these settings.
[Setup guide](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/eng/manuals/GETTING-STARTED.md).

The earlier accepted Android API34 ENONET limitation and excluded physical test
platforms retain their original scope; this update does not reclassify them as fixed.

### Linux candidate short test — 2026-10-22

The exact packaged **0.8.3** ELF was tested on VM300/server and VM301/client:
SHA-256 `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`,
source `be4429ae8c870c88b2682619ba8246fae3f8a4b4`. Later transport and Android
carrier-recovery changes are outside this test; it does not qualify the current
development tree. These are not measurements
of an earlier diagnostic candidate. Each VM had **2 vCPU / 2 GiB RAM**, Intel
Core i5-8500, Linux `6.12.111+deb13-amd64`, iperf3 3.18 and a dedicated local
test link. TUN MTU **1400**, `tun.tx_queue_len=1000` on both peers, two server
TUN queues; **Recordizer required**, active PACKET_MUX_V1 verified in every session.
Reality-TLS: genuine H2, handrolled and shaping, no padding/stealth/heartbeat.
No manual tuning, reboot, package installation or rebuild was performed.

Required-only subset: 12-profile functional check, external-IPv6 smoke checks
of the four main profiles, **three speed repeats** of each main profile, extended
TCP windows and before/after baselines: **32 sessions / 288 windows**. Speed
windows were **10 s + 2 s warm-up**, TCP P1/P4 and UDP 300/600 Mbit/s, inner
IPv4/IPv6 in both directions; UDP payload 1200. Off/prefer, mixed-version/mixed-load
and long-soak gates are outside this run. UDP600 is a fixed offered load, not a ceiling.
P4 means four iperf streams inside one tunnel; `gateway=false`, `dns=off`.

TCP P4 receiver Mbit/s, median of three repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ C/S |
|---|---|---|---|---|---|---|
| `tcp-faketls` | 1976.8 | 2054.3 | 1921.4 | 2045.0 | 14411 | 9222 / 11837 |
| `tcp-reality-tls` | 1210.3 | 1208.0 | 1215.0 | 1291.2 | 8963 | 9370 / 9149 |
| `udp-faketls` | 1165.9 | 1220.7 | 1137.0 | 1208.8 | 15349 | 14904 / 11642 |
| `udp-quic` | 1096.3 | 1206.6 | 1104.9 | 1175.9 | 14874 | 10043 / 13197 |

UDP600 receiver Mbit/s / **worst loss %** across the three repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | TUN TX Σ C/S |
|---|---|---|---|---|---|
| `tcp-faketls` | 599.6 / 0.0000 | 600.0 / 0.0088 | 599.8 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |
| `tcp-reality-tls` | 599.6 / 0.0034 | 600.0 / 0.0000 | 599.1 / 0.0010 | 600.0 / 0.0000 | 0 / 0 |
| `udp-faketls` | 599.4 / 0.0000 | 600.0 / 0.0000 | 599.6 / 0.0059 | 600.0 / 0.0030 | 0 / 0 |
| `udp-quic` | 599.6 / 0.0082 | 600.0 / 0.0000 | 599.6 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |

`Retr Σ` sums the 12 TCP P4 sender retransmit counts per profile. `TUN TX Σ C/S`
sums client/server TX drops across the corresponding 12 windows (TCP P4 or UDP600).
TUN observation includes warm-up and a short tail; iperf retransmits exclude warm-up.
These are not identical counter intervals; FULL_RING was not traced in this test.

Result: **`REVIEW_REQUIRED`**. 0/96 speed UDP windows exceeded 1% loss (worst 0.0088%); 3/64 speed cells had CV >5% (maximum 7.64%). Maximum baseline drift 2.07%, CV 1.46%. No all-drops-fixed or regression-free claim is made.
Both guests passed **17/17 restoration checks**; boot IDs, resources, network,
firewall, checked sysctls and prior `/opt/qeli-*` file hashes were unchanged.
The tested ELF SHA was checked again on both guests and locally. Old binaries
and raw were preserved; new raw remains separately on both guests at
`/opt/qeli-release-short-083-required-20261022-r02`.

[Full repeat-level data, TCP P1/UDP300, CPU/RSS, counters and baselines](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release_short_083_required_2026-10-22.json);
[short-test method and limitations](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/eng/manuals/RELEASE-SHORT-TEST.md).
This local Linux test is not an Internet throughput guarantee, a matched regression
A/B or a substitute for the remaining release/platform gates. Release remains a draft.
The evidence identifies the tested snapshot; its release-tag link is not claimed live
before that tag is published.

## Русский

Обновление снимает ограничение TCP-буферов на каналах с большим RTT и добавляет
диагностику давления на очереди UDP/TUN. Ручные параметры и существующие бюджеты
памяти сохраняются.

- Серверные TCP-буферы по умолчанию управляются ОС: оба `perf.tcp.*_buffer_size`
  равны `0`. Установщик Linux явно включает receive autotuning, проверяет readback
  и сохраняет данные для отката. Ручные значения сохраняются; мультипрофильные
  примеры и «Быстрый запуск» панели используют те же дефолты.
- Внутренняя очередь приёма UDP фиксирована на 128 датаграммах на pump. Метрики
  глубины и ожидания помогают найти перегрузку без бесконтрольного роста очереди.
  Ограниченный контроллер socket receive buffer и ручные параметры TUN сохранены.
- TUN dispatch сначала пробует немедленную передачу и сохраняет backpressure при
  заполнении. `tun-stats` разделяет ожидания пула, передачи обработчику, паузы чтения
  и UDP batch-send. Устранение всех TUN drops и постоянной перегрузки не заявляется.
- Ручной клиентский MTU туннеля больше не отключает проверку внешнего UDP path MTU.
  `mtu_probe = false` сохраняет приоритет; при неудачной пробе действует безопасный fallback.
- macOS поставляется отдельными self-contained архивами `Qeli-macos-arm64.tar.gz`
  для Apple Silicon и `Qeli-macos-x86_64.tar.gz` для Intel; нужен macOS 15 или новее.
  Универсальный упаковщик отклоняет несовместимые .NET-компоненты: `lipo` не объединяет
  архитектурные managed DLL. Нативное Rust-ядро сохраняет обе архитектуры.
- macOS: исправлены `openat` на Apple Silicon, восстановление прав старых root-owned
  lock-файлов, распознавание состояния launchctl и обработка ошибок отключения/статуса.
- Android Tink обновлён до 1.24.0, Rust libc — до 0.2.191. rustls-webpki 0.103.16
  и webpki-roots 1.0.10 закрывают новые предупреждения о доверии сертификатам
  RUSTSEC-2026-0382 и RUSTSEC-2026-0381. Лицензии согласованы с новыми lock-файлами.
- Обновлены инструкции установки, примеры конфигов и диагностика буферов Docker.

TCP A/B проверялся на 1/8 независимых клиентах с добавленным RTT 40/100 мс в лабе
из двух Linux VM. В одном сценарии upload восьми клиентов медиана суммарной
скорости составила 2 456 Мбит/с с буферами ОС против 279 Мбит/с с фиксированными
262 144 байтами. Это сумма восьми туннелей на локальном пути, а не обещание скорости
одного клиента или Интернета. [Методика и результаты](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/queue-tuning-20261021.json).

На существующем Linux-сервере задайте оба TCP buffer size в профиле равными `0`,
проверьте `tcp_moderate_rcvbuf=1` и пределы `tcp_rmem`/`tcp_wmem`, выполните проверку
конфига и перезапуск с переподключением клиентов. Глобальный `tcp_mem` сохраняется.
Принудительная переустановка для этого не нужна.
[Инструкция](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/ru/manuals/GETTING-STARTED.md).

Ранее принятое ограничение Android API34 ENONET и исключённые физические стенды
сохраняют прежний статус; это обновление не объявляет их исправленными.


### Доработки по issue #69

- Проверка SNI учитывает реальное использование TLS/WebSocket; старое SNI UDP
  obfs не блокирует рабочий профиль. Лишнее поле не экспортируется в новые ссылки.
  Desktop-редактор показывает HTTP Host для TCP WebSocket.
- Общий `pull_routes=false` отключает только серверные маршруты, сохраняя
  локальную политику и защищённый tunnel DNS. Настройка переносится всеми
  клиентами через INI/модели/backup/новые ссылки. Исправлено сохранение gateway=false.
- NDP выбирает назначенный source uplink, повторно проверяет готового владельца,
  использует MLD для известных адресов и ограниченный анонс появления. В manual
  маршруты/firewall остаются у администратора. Произвольных NA-флагов и периодической
  рассылки нет.
- UDP-диагностика разделяет inner MTU, внешний бюджет и причины fallback;
  счётчики DATA_FRAG показывают попытки разделения записей и созданные фрагменты.
- Windows: typed IPv4/IPv6/copy в GUI и службе, размеры редакторов и bounded график.
  Выключенная по умолчанию политика sharing-bindings применяется только к своему Wintun.

[План и границы проверки](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/certification/plans/ISSUE69_0.8.3.md).
Fit-path, периодические NDP-анонсы и постоянный адаптер — отдельные этапы.
Реальные Windows bindings и MLD-snooping провайдера не объявляются проверенными
на основании mock/namespace-тестов.

### Короткий тест Linux-кандидата — 2026-10-22

На VM300/server и VM301/client проверен точный пакетный ELF **0.8.3**:
SHA-256 `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`,
исходники `be4429ae8c870c88b2682619ba8246fae3f8a4b4`. Последующие изменения
транспорта и восстановления сети Android этим тестом не охвачены; он не является
допуском текущего дерева разработки. Это не перенос замеров
прежнего диагностического кандидата. Каждая ВМ: **2 vCPU / 2 GiB RAM**, Intel
Core i5-8500, Linux `6.12.111+deb13-amd64`, iperf3 3.18, выделенный локальный
тестовый линк. TUN MTU **1400**, `tun.tx_queue_len=1000` на обеих сторонах,
две серверные TUN-очереди; **Recordizer required**, активный PACKET_MUX_V1
подтверждён в каждой сессии. Reality-TLS: genuine H2, handrolled и shaping,
без padding/stealth/heartbeat. Ручного тюнинга, reboot, установок и пересборки нет.

Required-only подмножество: функциональная проверка 12 профилей, внешний IPv6
четырёх основных, **три скоростных повтора** каждого основного профиля, удлинённые
TCP-окна и baseline до/после — **32 сессии / 288 окон**. Скоростные окна:
**10 с + 2 с прогрева**, TCP P1/P4 и UDP 300/600 Мбит/с, внутренние IPv4/IPv6
в обе стороны; UDP payload 1200. Off/prefer, смешанные версии/нагрузки и длительный
soak сюда не входят. UDP600 — фиксированная предложенная нагрузка, не потолок.
P4 — четыре потока iperf внутри одного туннеля; `gateway=false`, `dns=off`.

TCP P4: receiver Мбит/с, медиана трёх повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ К/С |
|---|---|---|---|---|---|---|
| `tcp-faketls` | 1976.8 | 2054.3 | 1921.4 | 2045.0 | 14411 | 9222 / 11837 |
| `tcp-reality-tls` | 1210.3 | 1208.0 | 1215.0 | 1291.2 | 8963 | 9370 / 9149 |
| `udp-faketls` | 1165.9 | 1220.7 | 1137.0 | 1208.8 | 15349 | 14904 / 11642 |
| `udp-quic` | 1096.3 | 1206.6 | 1104.9 | 1175.9 | 14874 | 10043 / 13197 |

UDP600: receiver Мбит/с / **худшая потеря %** из трёх повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | TUN TX Σ К/С |
|---|---|---|---|---|---|
| `tcp-faketls` | 599.6 / 0.0000 | 600.0 / 0.0088 | 599.8 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |
| `tcp-reality-tls` | 599.6 / 0.0034 | 600.0 / 0.0000 | 599.1 / 0.0010 | 600.0 / 0.0000 | 0 / 0 |
| `udp-faketls` | 599.4 / 0.0000 | 600.0 / 0.0000 | 599.6 / 0.0059 | 600.0 / 0.0030 | 0 / 0 |
| `udp-quic` | 599.6 / 0.0082 | 600.0 / 0.0000 | 599.6 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |

`Retr Σ` — сумма sender retransmits по 12 TCP P4 окнам профиля. `TUN TX Σ К/С` —
сумма TX drops клиента/сервера по соответствующим 12 окнам (TCP P4 или UDP600).
Наблюдение TUN включает прогрев и небольшой хвост, retransmits iperf исключают
прогрев; интервалы не тождественны. FULL_RING в этом тесте не трассировался.

Результат: **`REVIEW_REQUIRED`**. Порог потерь 1% превышен в 0/96 скоростных UDP-окнах (максимум 0.0088%); CV >5% у 3/64 скоростных ячеек (максимум 7.64%). Максимальные baseline drift 2.07%, CV 1.46%. Устранение всех drops и отсутствие регрессий не заявляются.
Обе ВМ прошли **17/17 проверок восстановления**: boot IDs, ресурсы, сеть, firewall,
проверяемые sysctl и хеши прежних файлов `/opt/qeli-*` совпали. SHA проверенного
ELF повторно сверен на обоих гостях и локально. Старые бинарники и raw сохранены;
новые raw отдельно на обеих ВМ:
`/opt/qeli-release-short-083-required-20261022-r02`.

[Все повторы, TCP P1/UDP300, CPU/RSS, счётчики и baseline](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release_short_083_required_2026-10-22.json);
[методика короткого теста и ограничения](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/ru/manuals/RELEASE-SHORT-TEST.md).
Это локальная Linux-проверка, не гарантия скорости в Интернете, не сопоставимый
регрессионный A/B и не замена остальных release/platform gates. Релиз остаётся черновиком.
Данные относятся к указанному снимку; ссылка на release tag не объявляется рабочей
до публикации соответствующего тега.
