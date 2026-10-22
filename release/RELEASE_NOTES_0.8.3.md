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

Scope and checks: [Issue #69 implementation plan](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/ISSUE69_PLAN_0.8.3.md).
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

[План и границы проверки](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/ISSUE69_PLAN_0.8.3.md).
Fit-path, периодические NDP-анонсы и постоянный адаптер — отдельные этапы.
Реальные Windows bindings и MLD-snooping провайдера не объявляются проверенными
на основании mock/namespace-тестов.
