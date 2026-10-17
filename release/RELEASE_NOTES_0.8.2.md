# Qeli 0.8.2 — consistent configuration and reliable operation / единые конфиги и надёжность

**Draft for review. The release is prepared locally; publication awaits approval.**
**Черновик для согласования. Релиз подготовлен локально; публикация ожидает подтверждения.**
The 0.8 line remains beta. Линейка 0.8 остаётся бета-версией.

**Language / Язык:** [English](#english) · [Русский](#русский).

## English

### What we worked on and why

Qeli 0.8.2 focuses on consistent configuration and reliable operation through reconnects,
restarts and partial failures. We reviewed the path from entering a setting in the panel
to parsing it in the core, applying it to the operating system and recovering from errors.

The work followed a [37-section system audit plan](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/FULL-SYSTEM-AUDIT.md):
configuration, panel and API, authentication and cryptography, transports, IPv4/IPv6,
DNS, clients, installation, dependencies, tests and documentation. Each section covered
code review, normal and invalid inputs, failures and concurrency, available integration
tests, and verification of fixes. All 37 sections are complete within the agreed scope.
This was an internal engineering audit; accepted limitations and excluded physical
platforms remain recorded in the [audit debt register](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/AUDIT-DEBT.md).

### Scale and main outcomes of the audit

At the release-preparation snapshot [`6b20ea2a`](https://github.com/litvinovtd/qeli/commit/6b20ea2a0059e48786c9f6cd7120faf6890807fe)
on 16 October, development since 0.8.1 comprised **1,228 commits: 1,203 non-merge
commits and 25 merge commits**. This includes product changes, refactoring, tests,
build tooling and documentation—not 1,228 separate defects. The audit covered
**37 system areas**, with **323 audit reports**, each available in English and Russian;
translations are not counted as additional investigations.
[Changes since 0.8.1](https://github.com/litvinovtd/qeli/compare/v0.8.1...6b20ea2a0059e48786c9f6cd7120faf6890807fe)
· [Audit report catalogue](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/README.md).

The most important results are stronger rules for how the system behaves:

- **Failure must not silently remove protection.** Audited network operations track
  resource ownership and verify rollback; incomplete cleanup retains the protective
  block and the information needed to retry recovery.
- **Verify the server before sending credentials.** The shared core rejects invalid
  server proofs and degenerate static keys in the reviewed authentication paths.
  Key-file errors no longer silently replace an existing identity with a new one.
- **One interpretation of configuration.** Clients share the native INI/URI contract,
  while panel transactions detect stale edits and validate configuration before
  applying changes.
- **Evidence beyond a successful build.** Native A/B rebuilds, ABI/JNI checks,
  source/artifact hashes and exact-commit CI tie checks to identified inputs.
  Historical results and unavailable physical platforms are not silently counted
  as fresh successful tests.

For Qeli, this means a more predictable foundation: fewer client-specific differences,
explicit failures instead of false success, and more controlled recovery after an
interrupted operation. It does not mean that every possible failure has been eliminated.

### Main changes since 0.8.1

- **One configuration contract across clients.** INI and `qeli://` parsing and validation
  now use the shared native core. Duplicate client parsers were removed; common positive
  and negative fixtures check the adapters. This reduces differences in how clients
  interpret the same settings. Feature availability still depends on the OS.
  [Configuration-core plan](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/CLIENT-CONFIG-CORE.md).
- **INI configuration throughout.** The panel retains its form and INI editor; its JSON
  configuration editor was removed. Notifications use `notify.ini`. JSON remains a format
  for internal API messages and service payloads.
- **Safer panel changes.** Saves detect stale revisions; INI writes are private and
  atomic. Validation, quoting, masked secrets, access lists, backup/restore and restart
  checks were corrected. Loading or validation failures preserve the working
  configuration. Listener changes require a full restart instead of an ineffective
  worker restart.
- **Authentication and key handling.** Panel and VPN password checks share a bounded
  Argon2 budget. TCP authentication has time/input limits; retained hybrid key material
  is wiped, and degenerate static-key binding is rejected.
  [Cryptographic review](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/AUDIT-Q08-CRYPTO-KEYS.md).
- **Controlled network recovery.** Route, firewall, DNS and forwarding operations have
  stronger ownership, rollback and crash-recovery checks. NetworkPlan setup and cleanup
  each have a shared 15-second budget. Incomplete cleanup retains the protective block
  and reports the failure.
- **Transport and session fixes.** HTTP/2 tasks are released on cancellation; sending
  handles partial flow-control windows, and bonded rate limiting keeps the full shared
  debt. Incomplete management messages no longer receive premature acknowledgements,
  and conflicting duplicates are rejected. DATA_FRAG authentication and resource limits
  received additional tests. REALITY-TLS traffic keys have explicit lifetime limits
  followed by reconnection.
- **Client lifecycle and private storage.** Targeted fixes cover Android startup readiness
  and recovery, Windows GUI/service transitions and trusted DLL/profile loading, macOS
  helper/storage ownership, and iOS profile/Keychain/MDM handling. Native startup, protected
  storage and cleanup errors are reported instead of appearing as successful connections.
  Windows/macOS retry transient network restoration before reconnecting, with bounded
  failures and cancellation. Incomplete cleanup retains the kill-switch and resource
  ownership; unexpected reconnect-loop faults reach a visible Error state.
  Android and iOS clear the saved log after a completed VPN disconnect by default;
  the new setting can be disabled to retain diagnostics. Reconnects and network pauses
  preserve the log.
- **More useful administration.** Responsive host-load cards, interactive log-level filters
  and refreshed user bandwidth limits improve the panel. Linux/OpenWrt/Keenetic lifecycle
  hooks receive the applied network/connection context. Installer, updater, permissions
  and secret-handling paths were reviewed and corrected.
- **Verified build inputs.** Native libraries were refreshed and independently rebuilt A/B
  for reproducibility, ABI/JNI and configuration checks. Updates include rustls 0.23.46,
  tokio-rustls 0.26.6 and h2 0.4.20. The TLS update restores propagation of deferred read
  errors; h2 fixes framing, stream-limit and shutdown handling. Gradle 9.8.1 addresses
  three high-severity build-tool vulnerabilities; its Kotlin locks/checksums and
  AndroidX Core 1.19.1 runtime notices are aligned. Publication tooling checks source identity, runtime
  evidence, notices and exact-commit CI. Pending CI is now awaited instead of causing
  premature publication refusal.
- **Less duplicate and obsolete code.** Besides client parsers, obsolete loading paths
  and six unused performance implementations were retired. Benchmark checks reject
  incomplete or duplicated measurement phases.

### IPv6 with administrator-managed routing

The new `routing.ipv6.mode = manual` separates external network administration from
Qeli's tunnel and explicitly requested NDP proxy.

| Mode | Responsibility |
| --- | --- |
| `off` | Qeli blocks the profile's IPv6 transit. |
| `manual` | The administrator manages external routing, firewall and forwarding; Qeli retains tunnel addressing/client routes and can run the requested NDP proxy. |
| `route` | Qeli configures forwarding while preserving client addresses. |
| `nat66` | Qeli configures forwarding and NAT66. |

NDP proxy supports `manual` and `route`. `ndp_proxy = required` remains a startup
requirement; `manual` does not enable external forwarding automatically.
[Configuration manual](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/manuals/CONFIG.md).

### Testing and results

- Unit, configuration-conformance and regression tests covered malformed inputs,
  cancellation, concurrent operations and preservation of the working state.
- Retained evidence covers **20 required Linux runtime cases**, including network
  application/recovery and IPv4/IPv6. ABI/JNI checks and Android emulator tests
  complemented platform builds.
- Cryptographic checks included **60 NIST ML-KEM-768 known-answer cases**, RFC vectors
  for X25519 and ChaCha20-Poly1305, and independent HKDF checks.
  These check implementation consistency, not external security certification.
  [Cryptographic checks and limits](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/AUDIT-Q08-CRYPTO-KEYS.md).
- During the audit, **15 ASan fuzz targets** exercised parser/protocol boundaries;
  their original pinned run completed **85,930,590 iterations**. A complete
  eight-phase endurance run lasted approximately **13 hours 4 minutes**, including
  10,000 TCP and 10,000 QUIC cycles, additional UDP adapters and predefined resource
  budgets. These are retained audit-campaign executions, not claims that the full
  campaign was rerun for each subsequent release candidate.
  [Q35 summary](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/AUDIT-Q35-FINAL.md)
  · [Endurance report](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/AUDIT-Q35-LONG-SOAK.md).
- Packaged candidate `1a154fbd` passed **all 20 required CI jobs** in its
  [candidate run](https://github.com/litvinovtd/qeli/actions/runs/38941015737).
  The final preparation snapshot `6b20ea2a` also passed **20 required jobs** in its
  [CI run](https://github.com/litvinovtd/qeli/actions/runs/38944909120); two scheduled
  fuzz jobs were skipped, not counted as passed. The two snapshots have identical
  product sources; the later commit records Linux release certification. These CI
  results identify those snapshots, not future edits to the release notes.

These are separate test populations, not a combined test count. Reports retain source
and artifact identities, test conditions and original failures.

### Known limits

- On the measured Android 14/API34 emulator, the automatic-family `DnsResolver` call
  without an explicit network returned `ENONET` before sending a query. An independent
  minimal VPN reproduced it. Ordinary DNS, typed A/AAAA and explicit VPN-network queries
  worked in the tested scenarios. This accepted limitation remains unfixed. Secure VPN
  and the system kill switch are retained; no DNS bypass mode is added.
  [DNS investigation](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/audit-0.8.2/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md).
- Continuous identity across replacement of a selected WAN interface remains an accepted
  limitation, D06. Stop the profile and confirm cleanup before replacing that interface.
  [Details](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/AUDIT-DEBT.md).
- Physical Mac/iOS, router and Windows network-VM tests were excluded from the agreed
  scope. Builds, simulator and adapter tests retain their own scope and do not certify
  every device or OEM.

### Upgrade notes

Keep the previous binary and its INI files together for rollback. Recheck settings with
the current validator: malformed addresses, duplicate sections and unsupported fields
may now be rejected. Transfer required legacy notification settings manually to
`notify.ini`: automatic `notify.json` import is removed, and restore rejects that legacy
sidecar. On Windows, refusal of an untrusted legacy service-profile store requires
explicit reconfiguration from an elevated GUI after preserving the required user data.

Windows distributions are single-file `QeliWin-standalone.exe` and
`QeliWin-net-required.exe` again. Original Cargo/.NET, core/client and driver license
texts are embedded; **About → Licenses** reads and exports them offline. Linux and
router executables provide `licenses --list`, `licenses --file PATH` and
`licenses --export NEW_DIRECTORY` without starting VPN. The separate
`qeli-native-legal.zip` is no longer a release download; original texts and indexes
remain available inside the binaries. This does not replace source availability
or other license obligations.

The macOS universal ZIP includes both Apple Silicon and Intel code;
extract the complete `Qeli.app` on macOS, preserving executable permissions. It uses
ad-hoc signatures, without Developer ID notarization or the per-app system extension.
The Android update retains the existing release certificate; package signatures,
alignment and the included current JNI libraries are checked. This does not replace
installation and upgrade testing on your physical device.

Full change list: [CHANGELOG.md](https://github.com/litvinovtd/qeli/blob/v0.8.2/CHANGELOG.md).

## Русский

### Что мы делали и зачем

Qeli 0.8.2 посвящён согласованной конфигурации и надёжной работе при переподключениях,
перезапусках и частичных отказах. Мы проверили путь настройки от ввода в панели до
разбора в ядре, применения в операционной системе и восстановления после ошибки.

Работа шла по [плану полного аудита из 37 разделов](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/FULL-SYSTEM-AUDIT.md):
конфиги, панель и API, аутентификация и криптография, транспорты, IPv4/IPv6, DNS,
клиенты, установка, зависимости, тесты и документация. Каждый раздел включал разбор
кода, штатные и некорректные входы, отказы и конкуренцию, доступные интеграционные
проверки и повторную проверку исправлений. Все 37 разделов завершены в согласованном
объёме. Это внутренний инженерный аудит; принятые ограничения и исключённые физические
стенды сохранены в [реестре техдолга](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/AUDIT-DEBT.md).

### Масштаб и главные результаты аудита

На снимке подготовки релиза [`6b20ea2a`](https://github.com/litvinovtd/qeli/commit/6b20ea2a0059e48786c9f6cd7120faf6890807fe)
от 16 октября разработка после 0.8.1 включает **1 228 коммитов: 1 203 обычных
и 25 merge-коммитов**. Это изменения продукта, рефакторинг, тесты, инструменты сборки
и документация, а не 1 228 отдельных дефектов. Аудит охватил **37 направлений системы**;
подготовлены **323 отчёта**, каждый на русском и английском языках. Переводы
не считаются дополнительными независимыми проверками.
[Изменения после 0.8.1](https://github.com/litvinovtd/qeli/compare/v0.8.1...6b20ea2a0059e48786c9f6cd7120faf6890807fe)
· [Каталог отчётов аудита](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/README.md).

Главные результаты — более строгие правила поведения системы:

- **Ошибка не должна незаметно снимать защиту.** Проверенные сетевые операции
  учитывают владельца ресурсов и подтверждают откат; незавершённая очистка сохраняет
  защитную блокировку и сведения, необходимые для повторного восстановления.
- **Сначала проверка сервера, затем учётные данные.** Общее ядро отклоняет неверное
  криптографическое доказательство сервера и вырожденные статические ключи
  в проверенных путях аутентификации. Ошибки файлов ключей больше не приводят
  к молчаливой замене существующей identity новой.
- **Одинаковое понимание конфигурации.** Клиенты используют общий нативный контракт
  INI/URI; транзакции панели обнаруживают устаревшие правки и проверяют конфигурацию
  до применения изменений.
- **Подтверждение не ограничивается успешной сборкой.** Независимые пересборки A/B,
  проверки ABI/JNI, хеши исходников и артефактов, CI точного коммита связывают
  результаты с конкретными входами. Исторические проверки и недоступные физические
  платформы не выдаются за новые успешные прогоны.

Для Qeli это более предсказуемая основа: меньше различий между клиентами,
явные ошибки вместо ложного успеха и более управляемое восстановление после
прерванных операций. Это не означает, что устранены все возможные отказы.

### Основные изменения относительно 0.8.1

- **Общий контракт конфигурации для клиентов.** Разбор и валидация INI и `qeli://`
  перенесены в общее нативное ядро. Отдельные клиентские парсеры удалены; адаптеры
  проверяются общими положительными и отрицательными примерами. Это сокращает
  расхождения в понимании одних и тех же настроек. Доступность функций зависит от ОС.
  [План общего конфигурационного ядра](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/CLIENT-CONFIG-CORE.md).
- **Конфиги в INI.** В панели сохранены форма и INI-редактор; JSON-редактор конфигурации
  удалён. Уведомления используют `notify.ini`. JSON остаётся форматом служебного
  обмена API и внутренних сообщений.
- **Надёжнее сохранение через панель.** Сохранение обнаруживает устаревшие ревизии,
  INI записывается приватно и атомарно. Исправлены валидация, кавычки, замаскированные
  секреты, списки доступа, backup/restore и проверки перезапуска. При ошибке загрузки
  или проверки рабочая конфигурация сохраняется. Изменение listener требует полного
  перезапуска вместо безрезультатного перезапуска worker.
- **Аутентификация и работа с ключами.** Проверки паролей панели и VPN используют
  общий ограниченный бюджет Argon2. TCP-аутентификация ограничена по времени и размеру
  входа; сохраняемый материал гибридных ключей очищается из памяти, вырожденная привязка
  статического ключа отклоняется.
  [Разбор криптографии](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/AUDIT-Q08-CRYPTO-KEYS.md).
- **Управляемое восстановление сети.** Усилены проверки владельцев маршрутов,
  firewall, DNS и forwarding, отката и восстановления после аварии. Настройка
  NetworkPlan и очистка получили отдельные общие бюджеты по 15 секунд. Незавершённая
  очистка сохраняет защитную блокировку и сообщает об ошибке.
- **Исправления транспорта и сессий.** Задачи HTTP/2 освобождаются при отмене; отправка
  использует частичное flow-control окно, а общий ограничитель полосы учитывает весь
  накопленный долг. Неполные управляющие сообщения больше не получают преждевременный
  ACK, конфликтующие повторы отклоняются. Аутентификация и ресурсные лимиты DATA_FRAG
  получили дополнительные тесты. В REALITY-TLS заданы пределы использования traffic key
  с последующим переподключением.
- **Жизненный цикл и приватные хранилища клиентов.** Исправлены отдельные сценарии
  готовности и восстановления Android, переходы GUI/службы и доверенная загрузка
  DLL/профиля Windows, владение helper и хранилищами macOS, работа профилей,
  Keychain и MDM в iOS. Ошибки запуска ядра, защищённого хранилища и очистки
  сообщаются пользователю и не маскируются успешным подключением.
  Windows/macOS повторяют временно неудачное восстановление сети до нового подключения,
  с ограничением отказов и поддержкой отмены. Незавершённая очистка сохраняет kill-switch
  и владение ресурсами; неожиданные ошибки цикла переподключения приводят к видимому Error.
  Android и iOS по умолчанию очищают сохранённый журнал после завершённого отключения
  VPN; новую настройку можно выключить для сохранения диагностики. Переподключения
  и сетевые паузы историю не стирают.
- **Удобнее администрирование.** В панели появились адаптивные карточки нагрузки
  хоста, интерактивные фильтры уровней журнала и обновление лимитов полосы
  пользователей. Lifecycle-хуки Linux/OpenWrt/Keenetic получают контекст применённой
  сети и соединения. Проверены и исправлены пути установки, обновления, файловых
  прав и секретов.
- **Проверенные входы сборки.** Нативные библиотеки обновлены и независимо пересобраны
  A/B для проверки воспроизводимости, ABI/JNI и конфигурации. Обновлены зависимости,
  включая rustls 0.23.46, tokio-rustls 0.26.6 и h2 0.4.20. TLS-обновление восстанавливает
  передачу отложенных ошибок чтения; h2 исправляет обработку кадров, stream limits
  и завершения соединения. Gradle 9.8.1 закрывает три уязвимости высокой серьёзности
  в инструментах сборки; согласованы его Kotlin locks/checksums и лицензии
  AndroidX Core 1.19.1. Инструменты выпуска
  проверяют исходники, результаты runtime-проверок, лицензии и CI точного коммита.
  Незавершённый CI теперь ожидается вместо преждевременного отказа публикации
  только из-за того, что проверки ещё идут.
- **Меньше дублирования и устаревшего кода.** Помимо отдельных парсеров удалены
  устаревшие пути загрузки и выведены из эксплуатации шесть неиспользуемых
  реализаций performance-сценариев. Проверки бенчмарков отклоняют неполные
  и повторно учтённые фазы измерений.

### IPv6 с маршрутизацией под управлением администратора

Новый `routing.ipv6.mode = manual` отделяет администрирование внешней сети
от работы туннеля Qeli и явно запрошенного NDP proxy.

| Режим | Ответственность |
| --- | --- |
| `off` | Qeli блокирует IPv6-транзит профиля. |
| `manual` | Администратор настраивает внешнюю маршрутизацию, firewall и forwarding; Qeli сохраняет адреса туннеля/клиентские маршруты и может обслуживать запрошенный NDP proxy. |
| `route` | Qeli настраивает пересылку с сохранением адресов клиентов. |
| `nat66` | Qeli настраивает пересылку и NAT66. |

NDP proxy поддерживается в `manual` и `route`. `ndp_proxy = required` остаётся
обязательным условием запуска; `manual` сам не включает внешний forwarding.
[Руководство по конфигурации](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/manuals/CONFIG.md).

### Тестирование и результаты

- Модульные тесты, общие проверки конфигурации и регрессии охватывали некорректные
  входы, отмену, конкурентные операции и сохранение рабочего состояния.
- Сохранены результаты **20 обязательных Linux runtime-сценариев**, включая применение
  и восстановление сети и IPv4/IPv6. Сборки платформ дополнены проверками ABI/JNI
  и Android на эмуляторе.
- Криптографические проверки включали **60 эталонных случаев NIST ML-KEM-768**,
  векторы RFC для X25519 и ChaCha20-Poly1305, независимые проверки HKDF.
  Это проверка согласованности реализации, а не внешняя сертификация безопасности.
  [Проверки криптографии и их границы](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/AUDIT-Q08-CRYPTO-KEYS.md).
- В ходе аудита **15 ASan fuzz-целей** проверяли границы парсеров и протокола;
  исходный закреплённый прогон выполнил **85 930 590 итераций**. Полный восьмиэтапный
  длительный прогон занял около **13 часов 4 минут**: 10 000 циклов TCP и 10 000 QUIC,
  дополнительные UDP-адаптеры и заранее заданные бюджеты ресурсов. Это сохранённые
  исполнения программы аудита, а не утверждение о повторении всей программы
  для каждого последующего кандидата релиза.
  [Итог Q35](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/AUDIT-Q35-FINAL.md)
  · [Отчёт длительного прогона](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/AUDIT-Q35-LONG-SOAK.md).
- Упакованный кандидат `1a154fbd` прошёл **все 20 обязательных заданий CI**
  в [прогоне кандидата](https://github.com/litvinovtd/qeli/actions/runs/38941015737).
  Итоговый снимок подготовки `6b20ea2a` также прошёл **20 обязательных заданий**
  в [CI](https://github.com/litvinovtd/qeli/actions/runs/38944909120); два задания fuzz
  по расписанию пропущены, а не засчитаны как PASS. Исходники продукта у этих снимков
  одинаковые; поздний коммит фиксирует Linux-квалификацию релиза. Эти результаты CI
  относятся к указанным снимкам, а не к будущим правкам описания релиза.

Это отдельные наборы проверок, их числа не складываются в один счётчик тестов.
В отчётах сохранены версии исполненных исходников и артефактов, условия
и первоначальные отказы.

### Известные ограничения

- На проверенном эмуляторе Android 14/API34 вызов `DnsResolver` с автоматическим
  выбором семейства адресов без явной сети возвращал `ENONET` до отправки запроса.
  Независимый минимальный VPN воспроизвёл результат. Обычный DNS, типизированные
  A/AAAA и запросы через явно выбранную VPN-сеть работали в проверенных сценариях.
  Принятое ограничение остаётся неисправленным. Сохраняются защищённый VPN
  и системный kill switch; DNS bypass-режим не добавляется.
  [Расследование DNS](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/audit-0.8.2/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md).
- Непрерывная идентичность при замене выбранного WAN-интерфейса остаётся принятым
  ограничением D06. Перед заменой интерфейса нужно остановить профиль
  и подтвердить очистку. [Подробности](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/AUDIT-DEBT.md).
- Физические Mac/iOS, роутер и сетевая Windows VM исключены из согласованного объёма.
  Результаты сборок, симулятора и проверок адаптеров сохраняют собственные границы
  и не подтверждают работу каждого устройства или OEM.

### Порядок обновления

Сохраните прежний бинарный файл вместе с его INI для отката. Проверьте настройки
текущим валидатором: некорректные адреса, повторные секции и неподдерживаемые поля
теперь могут отклоняться. Нужные старые настройки уведомлений перенесите вручную
в `notify.ini`: автоматический импорт `notify.json` удалён, restore отклоняет этот
legacy-файл. Если Windows отказывает в доверии старому сервисному хранилищу,
сохраните нужные пользовательские данные и явно настройте службу из GUI
с правами администратора.

Windows снова распространяется отдельными single-file `QeliWin-standalone.exe`
и `QeliWin-net-required.exe`. Оригинальные тексты лицензий Cargo/.NET, ядра/клиента
и драйверов встроены; **О программе → Лицензии** позволяет читать и экспортировать
их без сети. Linux- и роутерные бинарники поддерживают `licenses --list`,
`licenses --file PATH`, `licenses --export NEW_DIRECTORY` без запуска VPN.
Отдельный `qeli-native-legal.zip` убран из загрузок релиза; исходные тексты и индексы
доступны внутри бинарников. Это не заменяет доступность исходников и остальные
условия лицензий.

Universal ZIP для macOS содержит код Apple Silicon и Intel;
распакуйте весь `Qeli.app` на macOS с сохранением прав запуска. Подпись — ad-hoc,
без Developer ID notarization и системного расширения per-app.
Android-обновление сохраняет прежний релизный сертификат; проверяются подпись пакета,
выравнивание и включённые актуальные JNI-библиотеки. Это не заменяет проверку установки
и обновления на вашем физическом устройстве.

Полный список изменений — [CHANGELOG.md](https://github.com/litvinovtd/qeli/blob/v0.8.2/CHANGELOG.md).
