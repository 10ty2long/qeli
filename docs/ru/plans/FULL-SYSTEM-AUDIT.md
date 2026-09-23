# Полный аудит Qeli: история проверок и пошаговый тестовый план

<!-- normative-sync: full-system-audit-v1 -->

Дата инвентаризации: **22 сентября 2026**. База: ветка `dev`, commit
`fc6f4a5dc8df7f119f2d99a6b72b08916ae7a268`, разработка **0.8.2**.
Рабочее дерево до этой документации было чистым. Предыдущие исправления панели и
`manual + NDP` уже находятся в истории Git. Код продукта в этом этапе не изменялся.

Цель — последовательно проверить все системы Qeli на функциональные ошибки,
небезопасные границы доверия, гонки, утечки ресурсов/трафика, несогласованность
платформ, мёртвый код и расхождения документации. Это исполнимый план нового цикла,
а не заявление, что все системы уже прошли аудит.

## 1. Что удалось восстановить по памяти

Использованы история этой задачи, релевантные задачи «аудит» и «ipv6», локальная
память Qeli, сохранённые отчёты, Git и текущая структура исходников. Старые записи
могут относиться к удалённым реализациям; отметка «исправлено» в памяти не закрывает
регрессию на текущем commit. Независимый внешний криптоаудит этим не подтверждается.

| Источник | Период и восстановленное покрытие | Доказательство и предел |
|---|---|---|
| H01 | 10–12 июня: crypto/KDF/replay, handshake/framing, auth/Argon2, users/sessions, Win storage, DNS/kill switch и INI | [Аудит 10 июня](../archive/audits/AUDIT-2026-06-10.md), [11 июня](../archive/audits/AUDIT-2026-06-11.md), [12 июня](../archive/audits/AUDIT-2026-06-12.md); исторические версии |
| H02 | 18 июня — 5 июля: панель/CSRF, SSRF notify, DNS/DHCP, supervisor/control, NAT cleanup, obfs/WS/AWG и serializers | Память Qeli и [сводка 5 июля](../../archive/audits/AUDIT-FIXES-2026-07-05.md); отдельные старые подозрения были опровергнуты |
| H03 | 11–12 июля: trust boundary hooks/restore/client-save, session teardown, quota/iroute, clients и backup | Запись `project_qeli_audit_2026-07-11.md`; исторические fixes/build gates, не свежий E2E |
| H04 | 23–27 июля: core/data plane, UDP auth/HOL, pool races, max_clients, NAT tags, routes, парсеры, все клиенты и supply chain | Память core/client audits 24/25 июля и [сводка 27 июля](../../archive/audits/AUDIT-2026-07-27-FIXES.md); часть платформенных проверок была compile-only |
| H05 | 4 августа: общий аудит core, клиентов, панели и scripts | Запись `project_qeli_audit_2026-08-04.md`; перечень исторических кандидатов, не реестр текущих дефектов |
| H06 | Август–сентябрь: IPv6/TAP/NetworkPlan/PMTU/DATA_FRAG, roaming/CONTROL_V2, native/core parity | [План IPv6](IPV6-IMPLEMENTATION-PLAN.md), [роуминг](ROAMING.md), [transport core](../reference/TRANSPORT-CORE.md), `release/ipv6_lab_matrix_dev.json`; применимость evidence к текущему SHA перепроверяется |
| H07 | 26 августа — 1 сентября: REALITY/H2 PCAP, обнаружимость и throughput/CPU/RSS | [DPI](../reports/DPI-AUDIT.md), [сравнительный бенчмарк](../reports/benchmarks/vpn_protocol_benchmark_repeat_2026-09-01.md); измерения 0.8.0, не 0.8.2 |
| H08 | 16 сентября: общий многослойный аудит A01–A14 и fixes | Локальные `audit-vpn-20260916/AUDIT.md` и `FIX-VERIFICATION.md`; Rust/managed builds, probes и Linux tests; не физические платформенные E2E |
| H09 | 16–17 сентября: углублённый аудит панели/парсеров A01–A14 | Локальные `audit-panel-20260916/AUDIT.md`, `panel-fixes-20260917/RESULT.md`; Rust/JS probes, сохранение, ACL-поля, секреты, черновики и INI roundtrip |
| H10 | 22 сентября: продолжение панели A15–A22 | Локальные `audit-panel-20260922/AUDIT.md`, `panel-fixes-20260922/RESULT.md`; restart/preflight, staged restore, share transaction, quotes/dev, notify INI; commit `a8c5050b` |
| H11 | 22 сентября: IPv6 manual/NDP и документация | Локальный `ipv6-manual-20260922/RESULT.md`; commit `fc6f4a5d`; 632 portable Rust tests, 25 JS groups, 16 isolated runtime probes; без настоящего Linux NDP/firewall E2E |

Локальные отчёты H08–H11 лежат вне репозитория; их имена оставлены для поиска в
рабочем архиве. Секреты и адреса лабораторных/рабочих серверов сюда не переносятся.
Номера **A01–A14 повторно использованы в двух разных аудитах**: ссылка на находку
обязана включать H08 или H09. Числа тестов разных feature/OS-наборов не складываются
и не сравниваются как показатель роста или уменьшения покрытия.

## 2. Правила нового прохода

У каждого раздела две независимые оценки: **историческое покрытие** и **новый прогон**.
Состояния нового прогона: `TODO`, `IN_PROGRESS`, `PASS`, `FAIL`, `BLOCKED`, `N/A`.
`BLOCKED` требует причины и нужного стенда; `N/A` — подтверждённой неприменимости.
Отсутствующий runtime не превращает проверку в `PASS`. Исправленная находка закрывается
после проверки исправления; нерешённая находка остаётся в очереди независимо от других PASS.

Каждый раздел проходит одни и те же уровни:

1. **Структура:** входы/выходы, владелец состояния, зависимости, trust boundaries и все
   callers; OS/feature gates, FFI/reflection/generated code и потенциально мёртвые ветви.
2. **Контракт:** штатные сценарии, границы, malformed input, defaults и сохранение смысла;
   differential/roundtrip проверки независимыми реализациями.
3. **Отказы:** failure injection при acquire/apply/save, partial I/O, concurrent actions,
   timeout/cancel/crash/restart и корректное восстановление владельцем ресурса.
4. **Интеграция:** настоящий процесс/API/browser/TUN/firewall/OS adapter, затем реальные
   устройства, если поведение зависит от драйвера, ОС или мобильной сети.
5. **Регрессия:** воспроизводитель на исходном дефекте, исправление, повтор затронутых
   проверок, документация и фиксация результата с точным SHA/feature/target.

Для каждого шага нужен отчёт: ID, дата, commit + dirty diff/hash, версии инструментов,
OS/arch/features, команда, fixtures/seed, ожидаемый/фактический результат, stdout/stderr,
exit code, PCAP/host-state при необходимости, finding ID/severity и ограничения.
Гипотеза отделяется от воспроизведённого бага; accepted risk имеет обоснование.
Долгие тесты получают timeout и план остановки; утечки измеряются, а не оцениваются по UI.

До запуска старого lab/E2E-скрипта проверить target, hardcoded endpoints, cleanup,
credentials handling и destructive defaults. Наличие файла теста ниже означает
**точку входа для проверки обвязки**, а не готовый безопасный набор или доказанное
покрытие всех перечисленных сценариев. Результат теста самого harness не равен E2E.
Сетевые/разрушительные сценарии выполняются на отдельном стенде со снимком, не на prod.

## 3. Стенды и обязательная матрица

| Стенд | Назначение |
|---|---|
| Локальный portable | Parser/crypto/protocol/KAT, JS state, Python harness, docs; не Linux server runtime |
| Изолированный Linux | Server/CLI/API/backup/systemd/TUN/routes/DNS/NAT/NDP; iptables и nft/firewalld, multiprofile, crash recovery |
| Windows VM | GUI/LocalSystem/Wintun/WinDivert/ACL/DPAPI, реальные DNS/routes/firewall и восстановление |
| macOS Intel + ARM | launchd/utun/pf/Keychain/Network Extension/per-app; build отдельно от runtime |
| Android устройство | Release APK/JNI/VpnService, Wi-Fi/LTE/Doze/always-on/lockdown, sleep/wake |
| iOS устройство | Signed IPA/PacketTunnel/On Demand/NAT64/per-app/MDM; simulator отдельно |
| OpenWrt/Keenetic | MIPS/ARM и 32-bit ABI, init/UCI/LuCI, реальная сеть и ограниченная память |
| Нагрузочный стенд | Контролируемые bandwidth/RTT/jitter/loss/reorder/MTU, отдельные генератор и наблюдатель |

Сначала составить список **реально поддерживаемых** transport × obfuscation сочетаний
из кода: Quick Start не является полным списком protocol capabilities. Неподдерживаемые
сочетания проверять на понятный отказ, не записывать как недостающий PASS.
Для каждого поддерживаемого режима: соединение/auth, трафик вверх/вниз, DNS,
reconnect и stop/cleanup. Дополнительные оси:

- outer IPv4/IPv6 × inner IPv4/IPv6/dual; IPv6-only uplink/NAT64;
- full/split, TUN/TAP, client-to-client/site-to-site/per-app;
- IPv6 egress `off/manual/route/nat66` × NDP `off/auto/required`, все переходы;
- старый/новый сервер и клиент, несовместимые ABI/capabilities, off/prefer/required;
- single/multiprofile, single/multisession/multipath и пустой/исчерпанный pool;
- clean stop, timeout, revoke, crash, reload/restart, suspend/resume и network change.

Обязательные сочетания безопасности выполняются полностью. Для остальных взаимодействий
допустим pairwise-набор с явной таблицей покрытых комбинаций, а не обещание полного
декартова произведения. Для leak-проверок нужны packet capture на tunnel и physical
интерфейсах; для cleanup — before/after routes/DNS/firewall/sysctl/fd/tasks.

## 4. Этап 00 — исходная точка

**DONE: инвентаризация и ограниченные локальные проверки.** Это не завершение аудита
продукта. Следующий рабочий раздел — **01: серверный INI**, затем 02–07. Linux E2E
для последних административных и IPv6-изменений остаётся обязательным.

| Проверка на `fc6f4a5d` | Результат | Граница |
|---|---|---|
| `check_panel.py` | PASS: 11 шаблонов, 1142 RU строки | Статика |
| `test_panel_editors.cjs` | PASS: 25 групп | Реальные JS-компоненты в Node, не browser E2E |
| `unittest discover -s scripts -p 'test_native_*.py'` | PASS: 67 тестов | Проверяет инструменты/контракты, не пересобирает все native cores |
| `sync_version.py` | PASS: dev/planned 0.8.2, released 0.8.1 | Согласованность версии |
| `native-libs/provenance.py --check` | **FAIL: STALE NATIVE CORES** | Digest исходников отличается от recorded native digest |
| `release_certification.py --quiet` | **FAIL: manifest missing** | Нет `release/certification/0.8.2.json` |

**B00-01:** native digest recorded `85f2f17ed9f58e7ad8d0368b936542f2391961bf0a739c7985a93208d6a1cb80`,
actual `3deb3da9d8306e0eaf4fd3d3505a7ad8f4e822b7bd502e8e748f632a852baa52`.
Не выдавать packaged-библиотеки за проверку текущего Rust. Закрывается в 22/34
пересборкой и проверкой происхождения; простое обновление записи digest не подходит.

**B00-02:** отсутствие manifest не доказывает поломку runtime, но означает отсутствие
полного подтверждения готовности 0.8.2 к выпуску. Закрывается в 34 реальными evidence
из матрицы; создавать формальный файл с `passed` без прогонов нельзя.

## 5. Реестр модулей: что уже аудировали и что повторяем

Ниже полный восстановленный перечень в порядке нового прохода. H-коды ссылаются
на раздел 1; это история review/test, не текущий PASS. Кросс-системные работы 35–37
также применяются к каждому из разделов 01–34.

| ID | Модуль | Предыдущие проверки | Новый проход |
|---|---|---|---|
| 01 | Серверный INI и схема | H01, H04, H08–H10 | IN_PROGRESS |
| 02 | Клиентские парсеры и qeli:// | H04, H06, H08–H10 | IN_PROGRESS |
| 03 | Панель: UI и состояние | H02, H09–H11 | TODO |
| 04 | Web auth и защита API | H01–H03, H08–H09 | TODO |
| 05 | Транзакции конфигурации и restart | H08–H10 | TODO |
| 06 | Пользователи, группы и выдача доступа | H01, H04, H09–H10 | TODO |
| 07 | Backup, restore и history | H03–H04, H08, H10 | TODO |
| 08 | Криптография, identity и ключи | H01, H04, H08 | TODO |
| 09 | Handshake и pre-auth TCP/UDP | H01, H04, H08 | IN_PROGRESS |
| 10 | PacketCodec, replay и control framing | H01, H04, H08 | TODO |
| 11 | REALITY, TLS 1.3 и HTTP/2 | H07–H08 | IN_PROGRESS |
| 12 | Транспорты и wire-маскировка | H02, H07–H08 | TODO |
| 13 | Recordizer, padding и shaping | H02, H07–H08 | TODO |
| 14 | Supervisor, workers и профили | H02–H03, H08 | IN_PROGRESS |
| 15 | Сессии, IP-пулы и лимиты | H01, H03–H04, H08 | IN_PROGRESS |
| 16 | ACL, push routes и site-to-site | H03–H04, H06 | TODO |
| 17 | IPv4 NAT, forwarding и sysctl | H02, H04, H08 | IN_PROGRESS |
| 18 | IPv6 off/manual/route/nat66 и NDP | H06, H11 | IN_PROGRESS |
| 19 | DNS сервера и клиентов | H01–H02, H05–H06 | IN_PROGRESS |
| 20 | DHCP и lease lifecycle | H02, H05 | TODO |
| 21 | TUN/TAP, IP, MTU/PMTU и фрагментация | H06, H08 | IN_PROGRESS |
| 22 | Transport core, FFI/JNI и память | H06, H08 | IN_PROGRESS |
| 23 | Роуминг, resume и CONTROL_V2 | H06, H08 | IN_PROGRESS |
| 24 | Multipath, bonding и общий бюджет | H04, H06, H08 | IN_PROGRESS |
| 25 | Linux CLI и восстановление сети | H01, H04, H08 | IN_PROGRESS |
| 26 | Общий C# и managed/native граница | H04, H06, H08 | TODO |
| 27 | Windows: GUI, служба и драйверы | H01, H04, H08 | IN_PROGRESS |
| 28 | macOS: daemon, utun, pf и Network Extension | H04, H08 | TODO |
| 29 | Android: VpnService, JNI и lifecycle | H04, H06, H08 | TODO |
| 30 | iOS: PacketTunnel, Swift и MDM | H04, H06, H08 | TODO |
| 31 | OpenWrt, LuCI и Keenetic | H04, H06, H08 | TODO |
| 32 | Метрики, usage, логи и уведомления | H02–H03, H08, H10 | IN_PROGRESS |
| 33 | Установка, обновление, файловые права и hooks | H01, H04, H08 | IN_PROGRESS |
| 34 | CI, зависимости, native provenance и релиз | H04, H06, H08 | IN_PROGRESS |
| 35 | Fuzzing, concurrency, DoS и soak | H04, H06, H08 | TODO |
| 36 | Бенчмарки и методика измерения | H07 | TODO |
| 37 | Документация, тестовая обвязка и мёртвый код | H06, H08–H09, H11 | TODO |

## 6. Сценарии по каждому разделу

Порядок — 01 → 37. Если проверка требует другого стенда, фиксируется BLOCKED только
для этой части; независимый анализ следующего раздела продолжается, а блокер остаётся
в очереди. Итоговый PASS раздела требует всех обязательных уровней из раздела 2.

### 01. Серверный INI и схема

**Код:** `qeli/src/config`.

Каждый ключ: parse → validate → runtime → serialize; defaults, диапазоны, дубли секций/ключей, unknown keys, кавычки/TAB/Unicode. Неверный ввод отклоняется до записи. INI — единственный формат конфигов; JSON остаётся служебным API.

**Имеющаяся обвязка/fixtures:** `qeli/tests/config_examples.rs`.

- [ ] Review и мёртвый код.
- [x] Штатные, граничные и негативные сценарии парсера: 12 новых регрессий и существующий набор.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [x] Исправления первого прохода, повторная проверка и evidence (Q01-F001–F007).

**Статус: IN_PROGRESS.**

**Первый проход, 2026-09-22:** [отчёт Q01](../reports/AUDIT-Q01-SERVER-INI.md).
Исправлены 6 дефектов обработки INI и пробел покрытия fixture. 651 переносимый Rust-тест
и 25 JS-групп прошли; Linux all-targets check прошёл. Контроль fixture: 163 имени ключей
и 3 динамических семейства. Проверена изоляция 32 параллельных разборов.

**Открыто:** полный trace каждого поля до runtime и расширенные отказы; выполнение
Linux startup/CLI/SIGHUP/HTTP save — BLOCKED отсутствием настроенного Linux-стенда.
Компиляция не закрывает этот пункт. До этого раздел не получает общий PASS.

### 02. Клиентские парсеры и qeli://

**Код:** `qeli/src/config/client.rs`, `qeli/src/config/share.rs`, `conformance`.

Сверить текущий контракт 81 ключа в Rust/C#/Kotlin/Swift; INI ↔ формы ↔ URI, сохранение чужих полей и секретов. Проверить malformed pin/port/IPv6/MTU и запрет незаметного перехода в TOFU.

**Имеющаяся обвязка/fixtures:** `scripts/test_native_config_keys.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Проход URI, 2026-09-22:** [отчёт Q02](../reports/AUDIT-Q02-CLIENT-PARSERS.md).
Закрыты Q02-F001–F006: неоднозначные query-параметры, scalar/UTF-8/defaults и пропуск
JVM-перепроверки корпуса. 651 Rust-тест, 438 C# checks и 137 JVM-тестов прошли.
Общий корпус: 21 valid + 29 reject; контракт 81 имени ключей сохраняется.

**Открыто:** полный INI/редакторский проход; Swift build/test и платформенная интеграция.
**Архитектурное предложение пользователя:** [единый конфигурационный модуль](CLIENT-CONFIG-CORE.md)
в существующем Rust core реализован в исходниках (ABI 1.16). Местные парсеры удалены; проекции/defaults генерируются из Rust. Объединены политики маршрутов/reconnect/версий и парсер route_file. Apple runtime и релизные A/B-пересборки ещё не закрыты.

**Продолжение 22 сентября — границы конфигурации:** в общем ядре исправлены Q02-F007–F014 — восемь
воспроизведённых сценариев INI/URI: потеря символов и ошибок DNS, недопустимые имена
полей, обход проверки через BOM, маскирование секретов и неверная секция `logging.*`.
Регрессии проверяют сохранение через модели клиентов; подробности и результаты —
в [отчёте общего конфигурационного модуля](CLIENT-CONFIG-CORE.md#аудит-границ-конфигурации--22-сентября-2026).
Это не закрывает весь раздел: Linux E2E и проверки на целевых устройствах остаются открыты.

**Параметры перед runtime, 2026-09-22:** закрыты Q02-F015–F019 — неверный нулевой PIN,
некорректный host, исправление порта без его изменения, старый валидатор панели и ошибки автозаписи dev.
Регрессии и сохранение положительных случаев описаны в [реестре Q02](../reports/AUDIT-Q02-CLIENT-PARSERS.md).

**Продолжение runtime-аудита, 2026-09-22:** в том же плане зафиксированы общий бюджет/задержка
повторов, монотонное время установленной сессии, исправления конечных лимитов и возврата сети,
прерывание Linux-backoff по сигналу. Регрессии Rust/C ABI/JNI и Linux cross-Clippy пройдены.
Проверки реальных устройств/смены сети, Apple runtime и релизных библиотек этим не закрыты.

### 03. Панель: UI и состояние

**Код:** `qeli/src/web/templates`, `qeli/src/web/assets`, `qeli/src/web/pages`.

Загрузка/error/retry, dirty-state, поздние ответы, concurrent edits, Form/INI, удаление полей, маски секретов, даты и квоты. Настоящий браузер: все страницы, RU/EN, клавиатура, мобильный экран; ошибки не превращаются в сохранение defaults.

**Имеющаяся обвязка/fixtures:** `scripts/check_panel.py`, `scripts/test_panel_editors.cjs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 04. Web auth и защита API

**Код:** `qeli/src/web/auth.rs`, `qeli/src/web/mod.rs`, `qeli/src/web/api`.

Инвентаризировать routes/guards, Basic/cookie/TOTP, logout/expiry, CSRF, reverse proxy, base_path, allowed_ips. Argon2 budget/rate limit под конкуренцией. Запрос без прав не раскрывает секреты и не имеет побочных эффектов.

**Имеющаяся обвязка/fixtures:** `scripts/check_panel.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 05. Транзакции конфигурации и restart

**Код:** `qeli/src/web/api/config.rs`, `qeli/src/web/api/control.rs`, `qeli/src/server/preflight.rs`, `qeli/src/util.rs`.

Общий validator/preflight для Form/INI/API/history/Quick Start/worker/full restart. Stale revision, concurrent writers, ENOSPC/EACCES и crash между snapshot/rename/restart. Failed preflight не останавливает рабочий сервис и не публикует плохой конфиг.

**Имеющаяся обвязка/fixtures:** `scripts/test_web_reload.py`, `scripts/test_panel_editors.cjs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 06. Пользователи, группы и выдача доступа

**Код:** `qeli/src/config/users.rs`, `qeli/src/web/api/users.rs`, `qeli/src/web/api/share.rs`, `qeli/src/web/api/identity.rs`.

Inline + users_file, duplicates/precedence, missing group, неверные типы versus снятие ограничений, static addresses, quota/expiry. Ошибка identity не меняет пароль при выдаче ссылки. Revoke применяется к уже открытым TCP/UDP-сессиям.

**Имеющаяся обвязка/fixtures:** `scripts/test_user_reload.py`, `scripts/test_l3_user_limits.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 07. Backup, restore и history

**Код:** `qeli/src/web/api/backup.rs`, `qeli/src/web/api/backup_listing.rs`.

Fresh restore с custom paths, inline+external users, identity и panel-secret. Tar bombs, traversal, links, missing files, overlay/exact, совместимость staged-файлов, concurrent restore. Snapshots не архивируют себя; interrupted publish проверяется восстановлением.

**Имеющаяся обвязка/fixtures:** `scripts/test_web_reload.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 08. Криптография, identity и ключи

**Код:** `qeli/src/crypto`, `qeli/src/server/reality.rs`, `qeli/src/web/api/identity.rs`.

X25519/ML-KEM/HKDF/AEAD KAT и negative vectors; static binding, proof до credentials, pin/TOFU, RNG, nonce exhaustion, rotation и zeroization. Owner/mode/link/atomic-write ключей. Unit tests не заменяют независимый криптоаудит.

**Имеющаяся обвязка/fixtures:** `conformance/hkdf.json`, `conformance/prp-nonce.json`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 09. Handshake и pre-auth TCP/UDP

**Код:** `qeli/src/server/handler.rs`, `qeli/src/server/udp_handler.rs`, `qeli/src/protocol/capabilities.rs`.

Truncation/replay/reorder/slow peer и неверный PQ/proof/password. Permits до spawn, pending caps, anti-amplification, tarpit, deadlines/cancel. Один login не блокирует UDP recv-loop; каждый отказ освобождает ресурсы, downgrade только по контракту.

**Имеющаяся обвязка/fixtures:** `qeli/fuzz/fuzz_targets/clienthello.rs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Серверный H2 и pre-auth, 23 сентября 2026:**
[Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md): профиль ждёт вложенные H2-задачи перед
teardown; flush отказа сохраняет pre-auth slot до освобождения I/O. 13 новых регрессий,
969 Rust tests PASS. H2/ProfileTasks/semaphore проверены на host, production Linux только
кросс-компилирован. Остальные сценарии раздела и live Linux E2E остаются открытыми.

### 10. PacketCodec, replay и control framing

**Код:** `qeli/src/protocol/packet.rs`, `qeli/src/protocol/ctrl.rs`, `qeli/src/protocol/control_v2.rs`.

Lengths 0/min/max/overflow, AEAD tags, sequence около 2^63/2^64, replay-window, unknown types/generation. Malformed пакет не вызывает panic/abort/unbounded allocation; после отказа корректный пакет обрабатывается.

**Имеющаяся обвязка/fixtures:** `conformance/packet-decode.json`, `conformance/replay-window.json`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 11. REALITY, TLS 1.3 и HTTP/2

**Код:** `qeli/src/protocol/realtls`, `qeli/src/protocol/h2_carrier.rs`, `qeli/src/protocol/h2_carrier`.

Transcript/replay/decoy, TLS key budget. H2 zero/small window, SETTINGS/WINDOW_UPDATE/GOAWAY/RST, partial I/O, backpressure. Stop/timeout освобождает task/socket/permit. PCAP/active probing отдельно от работоспособности туннеля.

**Имеющаяся обвязка/fixtures:** `qeli/src/protocol/h2_carrier/hardening_tests.rs`, `scripts/reality_tls_repeat.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Серверный H2 и pre-auth, 23 сентября 2026:**
[Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md): профиль ждёт вложенные H2-задачи перед
teardown; flush отказа сохраняет pre-auth slot до освобождения I/O. 13 новых регрессий,
969 Rust tests PASS. H2/ProfileTasks/semaphore проверены на host, production Linux только
кросс-компилирован. Остальные сценарии раздела и live Linux E2E остаются открытыми.

### 12. Транспорты и wire-маскировка

**Код:** `qeli/src/protocol/tls.rs`, `qeli/src/protocol/obfs.rs`, `qeli/src/protocol/quic.rs`, `qeli/src/transport`.

Получить все допустимые сочетания из runtime/Quick Start: plain/fake-tls/reality/reality-tls/WS/obfs/UDP-QUIC/AWG. WS masking/control caps, junk counters, fallback и запрет несовместимых комбинаций. Protocol compliance и DPI detection не смешивать с goodput.

**Имеющаяся обвязка/fixtures:** `conformance/quic.json`, `qeli/fuzz/fuzz_targets/websocket_head.rs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 13. Recordizer, padding и shaping

**Код:** `qeli/src/protocol/recordizer.rs`, `qeli/src/protocol/shaper.rs`, `qeli/src/protocol/obfuscate.rs`.

Off/prefer/required и legacy peer; batch/reassembly caps, flush deadlines, cancellation и общий budget. Junk/heartbeat не вытесняют payload. Сравнить on/off на одинаковом workload; bounded memory, jitter и периодические сигналы в PCAP.

**Имеющаяся обвязка/fixtures:** `scripts/validate_shaping.py`, `scripts/bench_stealth.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 14. Supervisor, workers и профили

**Код:** `qeli/src/server/mod.rs`, `qeli/src/server/tasks.rs`, `qeli/src/server/supervisor.rs`, `qeli/src/server/control.rs`, `qeli/src/server/control_io.rs`, `qeli/src/server/control_socket.rs`, `qeli/src/main.rs`, `qeli/src/hooks.rs`, `qeli/src/hooks/process.rs`.

Start/stop/reload/crash/respawn, занятый bind/TUN, удаление/rename профиля, hook failure и умерший control client. Lock order, backoff, watchdog и tasks. Cleanup идемпотентен, ошибка одного профиля не затрагивает соседний.

**Имеющаяся обвязка/fixtures:** `scripts/test_web_reload.py`, `scripts/test_tun_reclaim.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Владение задачами, 23 сентября 2026:**
[проход Q14/Q19](../reports/AUDIT-Q14-Q19-LIFECYCLE.md) исправляет Q14-F001–F002:
ожидание служб после ранней ошибки запуска и барьер конкурентного/отменённого
shutdown. 7 task-ownership тестов, включая гонку 1024 ресурсов; DNS-слушатели
проверены через loopback. Linux E2E TUN/firewall, forced wrapper cancellation,
watch/control, hooks и restart/reload ещё открыты.

**Supervisor и управляющие события, 23 сентября 2026:**
[продолжение Q14](../reports/AUDIT-Q14-SUPERVISOR.md) исправляет Q14-F003–F007:
stop во время ошибок spawn, владение Child/PID, 60-секундный grace deadline,
очередь Restart/Reload и раннюю установку обработчиков сигналов. 13 поведенческих
тестов и дочерний fixture; 812 Rust-тестов суммарно — PASS. Проверены реальные
изолированные host-процессы, не Linux worker с TUN. Control socket, hooks,
Unix-сигналы и rollback на Linux остаются открытыми.

**Control socket и hooks, 23 сентября 2026:**
[проход Q14-F008–F013](../reports/AUDIT-Q14-CONTROL.md): владение Unix-сокетом,
безопасные права runtime-каталога, границы сообщений, deadlines и drain handlers
до teardown профилей, post_down только для готового поколения. 819 host Rust tests
PASS; 12 новых Unix tests только скомпилированы. Linux runtime/systemd/hooks и
forced cancellation остаются открытыми; далее — hook processes/output и startup rollback.

**Процессы hooks, 23 сентября 2026:**
[Q14-F014–F015](../reports/AUDIT-Q14-HOOKS.md): общий server/client runner удерживает
по 8 KiB stdout/stderr и завершает Linux process group при timeout/cancellation,
сохраняя штатные фоновые службы с перенаправленным выводом. 828 host Rust tests PASS;
4 новых Linux group tests только скомпилированы. Следующие участки: startup rollback,
фоновые worker services и связь trusted config с parsed contents. Linux E2E ещё открыт.

**Службы worker и учёт трафика, 23 сентября 2026:**
[Q14-F016–F017 / Q15-F001](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md): владение и контроль
периодических задач, их завершение до очистки профилей, writable accounting только после
захвата права worker, финальное сохранение на обоих путях остановки. Короткие сессии и
последние байты учитываются после удаления из реестра. 848 host Rust tests PASS;
Linux только cross-check. Уведомления, forced outer cancellation и Linux E2E ещё открыты.

**Владение уведомлениями, 23 сентября 2026:**
[Q14-F018 / Q32-F001](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md): до 128 принятых отправок,
8 активных запросов на процесс, общий лимит проб панели, ограниченные payloads и drain
до 10 секунд после завершения производителей. Detached-обёртки уведомлений удалены.
864 host Rust tests PASS; Linux только all-targets cross-check. Владение panel/metrics/
autostart supervisor, доверие конфигу и Linux E2E ещё открыты.

**Доверие прочитанному конфигу, 23 сентября 2026:**
[Q14-F019 / Q33-F001](../reports/AUDIT-Q14-Q33-CONFIG-TRUST.md): владелец/права и данные
для парсера получаются из одного дескриптора; исходное разрешение не меняется при
повторах профиля и не перепроверяет путь. Очистка готового поколения сохраняет команду
и окружение после удаления/замены конфига. 874 host Rust tests PASS; четыре новых Unix/
Linux-теста только cross-checked. Лимиты password_command, владение startup-задачами,
installer/update/restore и Linux runtime integration ещё открыты.

**Поставщик пароля и изоляция features, 23 сентября 2026:**
[Q25-F001 / Q33-F002 / Q14-F020 / Q34-F001](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md):
асинхронный password_command, deadline 30 секунд, полный stdout до 16 KiB, отброшенный
stderr и ошибки без секретов. Ранний SIGINT/SIGTERM отменяет и собирает поставщика;
watchers/sampler клиента имеют владельца. Исправлен server-only TUN gate, обе изолированные
features проверяются в CI. 883 host Rust tests PASS; четыре Linux-теста только cross-check.
Server-only check имеет 23 прежних transport dead-code warnings. Лимиты password_file,
финальный drain клиента и Linux runtime/release checks ещё открыты.

**Файловый пароль и финальный статус, 23 сентября 2026:**
[Q25-F002 / Q14-F021](../reports/AUDIT-Q25-PASSWORD-FILES.md): общий zeroizing-буфер 16 KiB
для файла/команды, одна управляемая blocking-задача чтения обычного файла, поддержка symlink,
отказ FIFO и ожидание активного I/O при штатном stop/deadline. Final пишется после join
watchers/sampler, включая ошибки после инициализации reporter. 895 host Rust tests PASS;
два Unix-теста только cross-check. Неотменяемый I/O может превышать бюджет 30 секунд.
Startup/network rollback, мониторинг фоновых ошибок и Linux E2E ещё открыты.

**Fail-closed очистка сети, 23 сентября 2026:**
[Q25-F003](../reports/AUDIT-Q25-NETWORK-CLEANUP.md): forwarding/NAT cleanup должен
завершиться успешно до снятия включённого kill-switch; ошибка сохраняет защиту и её причину.
899 host Rust tests PASS, включая четыре переносимых fault-injection сценария. Linux
пока только cross-check. Разбор begin_connection записан ниже; live firewall/E2E ещё открыты.

**Ошибки жизненного цикла ядра, 23 сентября 2026:**
[Q25-F004/F005](../reports/AUDIT-Q25-CORE-LIFECYCLE.md): ошибка запуска ядра проходит через
cleanup/post_down; ошибка остановки завершается отказом и сохраняет включённый kill-switch,
при этом очистка forwarding выполняется. Шесть новых host-регрессий, включая реальный отказ
ClientCore при полной очереди. 905 host Rust tests PASS; Linux только cross-check.
Live Linux lifecycle/firewall и полный rollback маршрутов/DNS ещё открыты.


**Передача ошибок очистки TUN/маршрутов/DNS, 23 сентября 2026:**
[Q25-F007–F009](../reports/AUDIT-Q25-TUN-CLEANUP.md): явная очистка и guards отката передают
ошибки в Linux retry loop через общий ограниченный журнал. Ошибка не становится успешной
остановкой по сигналу и не снимает включённый kill-switch. TunnelSetup владеет guard до ACK
ядра; тип terminal kick сохраняется при сопутствующих ошибках. Восемь новых host-тестов
проходят, два Linux adapter-теста только cross-check. 921 host Rust tests PASS. Live Linux
E2E, сроки выполнения команд и полное ожидание задач поколения ещё открыты.

**Серверный H2 и pre-auth, 23 сентября 2026:**
[Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md): профиль ждёт вложенные H2-задачи перед
teardown; flush отказа сохраняет pre-auth slot до освобождения I/O. 13 новых регрессий,
969 Rust tests PASS. H2/ProfileTasks/semaphore проверены на host, production Linux только
кросс-компилирован. Остальные сценарии раздела и live Linux E2E остаются открытыми.

**Системные команды TUN/DNS, 23 сентября 2026:**
[Q25-F016/F017](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md): 15 секунд на команду, полный
вывод с лимитом 16 МиБ на поток, завершение дочернего процесса и сохранение DNS marker
при отказе. Диагностика больше не обещает неподтверждённый rollback. 986 host Rust tests
PASS; два новых Linux process-group теста только cross-check. Маршруты/firewall, live
Linux и общий deadline shutdown остаются открытыми; статус раздела IN_PROGRESS.

**Очистка NAT, 23 сентября 2026:**
[Q14-F024/F025](../reports/AUDIT-Q14-NAT-CLEANUP.md): конечный проход по снимку правил,
проверка после удаления и диагностика ошибок с продолжением остальных правил/цепочек.
13 новых host-тестов, 999 Rust tests PASS; production Linux только cross-check.
Q14-F026 исправлена [следующим проходом](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md).
Q14-F027 (передача ошибок teardown), сроки firewall-команд и live Linux остаются открытыми.

**Общие firewall-проверки, 23 сентября 2026:**
[Q14-F026 / Q25-F018/F019](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md): сервер и Linux
kill-switch используют общий разбор presence/absence/errors; точечная очистка DNS
проверяет границу 1024 и продолжает TCP после отказа UDP. 17 новых host-тестов,
1016 Rust tests PASS; два новых Unix/Linux сценария только cross-check.
Q14-F027, сроки команд и реальные backend/runtime проверки остаются открытыми.

**Владение DNS firewall, 23 сентября 2026:**
[Q14-F028](../reports/AUDIT-Q14-DNS-OWNERSHIP.md): реестр worker сохраняет точные правила
при ошибке Drop/rollback; cleanup и новая установка повторяют очистку. Tokens защищают
новое поколение от старого lease; активные записи не участвуют в точечном retry.
12 новых host-тестов, 1028 Rust tests PASS; три adapter-регрессии отдельно сравнивают
baseline/fix. Q14-F027, persistent journal, deadlines и live Linux остаются открытыми.

**Итог остановки worker, 23 сентября 2026:**
[Q14-F027, частичное исправление](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md): окончательный
retry известных DNS/IPv6 sysctl leases влияет на Result и код выхода worker. Flush
статистики выполняется после ошибки сети; активные DNS leases обнаруживаются без удаления.
14 новых host-тестов, 1042 Rust tests PASS; восемь отдельных process-exit сценариев PASS.
Дополнительный проход передаёт ошибку worker через внешний supervisor при финальной
остановке: nonzero exit и принудительный kill больше не возвращают Ok. Ещё пять
host-тестов, итог этого этапа 1047 Rust tests PASS; три отдельные supervisor-регрессии PASS.
Следующий проход: [задачи профиля и TUN teardown](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md).
Ошибки shutdown JoinSet, TUN queue timeout/panic и удаления устройства теперь входят в
итог worker; 13 новых host-тестов, текущая матрица 1060 Rust tests PASS.
[Q14-F029/F030](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md): исправлены ложный успех
sysctl recovery и потеря существующего owner при неудачном повторном acquire;
1060 Rust tests и 7 отдельных fixture checks PASS.
Generic NAT, старые поколения/retry backoff, частичный IPv6 acquire, restart policy,
persistent journal и live Linux остаются открытыми.

### 15. Сессии, IP-пулы и лимиты

**Код:** `qeli/src/server/pool.rs`, `qeli/src/server/handler.rs`, `qeli/src/server/udp_handler.rs`, `qeli/src/server/usage.rs`.

Allocate/auth/reconnect/evict/reap/revoke/quota под конкуренцией, atomic v4+v6, static/reservations/excludes, pool exhaustion. Общие caps TCP/UDP/bonding. Нет duplicate IP, leaked lease/token/task/client_subnet после каждого выхода.

**Имеющаяся обвязка/fixtures:** `scripts/test_udp_reap.py`, `scripts/test_maxsessions.py`, `scripts/test_multidevice.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Проход учёта трафика:** [Q15-F001](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md) закрывает
короткие TCP/UDP-сессии, последние байты writer, baseline при reset и удаление счётчиков.
14 переносимых accounting-тестов проходят. IP-пулы, конкурентные auth/reconnect/revoke
и фактическое отключение по квоте на Linux ещё требуют остальных сценариев раздела 15.

### 16. ACL, push routes и site-to-site

**Код:** `qeli/src/server/acl.rs`, `qeli/src/config/users.rs`, `qeli/src/transport_core/network.rs`.

User/group/profile precedence, longest prefix, client_to_client, spoofed source, overlap, /0 и client_subnet return path. Проверить TCP/UDP, v4/v6. Enforcement на сервере; revoke/смена владельца маршрута не сохраняет доступ старой сессии.

**Имеющаяся обвязка/fixtures:** `scripts/test_push_matrix.py`, `scripts/test_route_push.py`, `scripts/test_l3_user_limits.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 17. IPv4 NAT, forwarding и sysctl

**Код:** `qeli/src/server/nat.rs`, `qeli/src/client/sysctl.rs`, `qeli/src/client/gateway.rs`.

NAT44/forward_private/gateway_nat/MSS и iptables/nft backend errors. Before/after rules/routes/sysctl при нескольких профилях. Точный tag вместо substring, ownership, crash journal/boot-id; чужие правила и значения сохраняются.

**Имеющаяся обвязка/fixtures:** `scripts/test_gateway_nat.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Очистка NAT, 23 сентября 2026:**
[Q14-F024/F025](../reports/AUDIT-Q14-NAT-CLEANUP.md): конечный проход по снимку правил,
проверка после удаления и диагностика ошибок с продолжением остальных правил/цепочек.
13 новых host-тестов, 999 Rust tests PASS; production Linux только cross-check.
Q14-F026 исправлена [следующим проходом](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md).
Q14-F027 (передача ошибок teardown), сроки firewall-команд и live Linux остаются открытыми.

**Общие firewall-проверки, 23 сентября 2026:**
[Q14-F026 / Q25-F018/F019](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md): сервер и Linux
kill-switch используют общий разбор presence/absence/errors; точечная очистка DNS
проверяет границу 1024 и продолжает TCP после отказа UDP. 17 новых host-тестов,
1016 Rust tests PASS; два новых Unix/Linux сценария только cross-check.
Q14-F027, сроки команд и реальные backend/runtime проверки остаются открытыми.

**Владение DNS firewall, 23 сентября 2026:**
[Q14-F028](../reports/AUDIT-Q14-DNS-OWNERSHIP.md): реестр worker сохраняет точные правила
при ошибке Drop/rollback; cleanup и новая установка повторяют очистку. Tokens защищают
новое поколение от старого lease; активные записи не участвуют в точечном retry.
12 новых host-тестов, 1028 Rust tests PASS; три adapter-регрессии отдельно сравнивают
baseline/fix. Q14-F027, persistent journal, deadlines и live Linux остаются открытыми.

**Итог остановки worker, 23 сентября 2026:**
[Q14-F027, частичное исправление](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md): окончательный
retry известных DNS/IPv6 sysctl leases влияет на Result и код выхода worker. Flush
статистики выполняется после ошибки сети; активные DNS leases обнаруживаются без удаления.
14 новых host-тестов, 1042 Rust tests PASS; восемь отдельных process-exit сценариев PASS.
Дополнительный проход передаёт ошибку worker через внешний supervisor при финальной
остановке: nonzero exit и принудительный kill больше не возвращают Ok. Ещё пять
host-тестов, итог этого этапа 1047 Rust tests PASS; три отдельные supervisor-регрессии PASS.
Следующий проход: [задачи профиля и TUN teardown](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md).
Ошибки shutdown JoinSet, TUN queue timeout/panic и удаления устройства теперь входят в
итог worker; 13 новых host-тестов, текущая матрица 1060 Rust tests PASS.
[Q14-F029/F030](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md): исправлены ложный успех
sysctl recovery и потеря существующего owner при неудачном повторном acquire;
1060 Rust tests и 7 отдельных fixture checks PASS.
Generic NAT, старые поколения/retry backoff, частичный IPv6 acquire, restart policy,
persistent journal и live Linux остаются открытыми.

### 18. IPv6 off/manual/route/nat66 и NDP

**Код:** `qeli/src/server/nat.rs`, `qeli/src/server/ndp_proxy.rs`, `qeli/src/config/server.rs`.

Все 4×3 egress/NDP комбинации и все переходы режимов для ipv4/dual/ipv6. Linux E2E: off блокирует transit; manual не добавляет IPv6 firewall/DNS/sysctl; route сохраняет source; nat66 маскирует. RA, exact cleanup, NS validation, live-session ownership/revoke, required failure, DNS 53/5353, независимый IPv4.

**Имеющаяся обвязка/fixtures:** `scripts/test_panel_ipv6_e2e.py`, `scripts/run_ipv6_release_matrix.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Очистка NAT, 23 сентября 2026:**
[Q14-F024/F025](../reports/AUDIT-Q14-NAT-CLEANUP.md): конечный проход по снимку правил,
проверка после удаления и диагностика ошибок с продолжением остальных правил/цепочек.
13 новых host-тестов, 999 Rust tests PASS; production Linux только cross-check.
Q14-F026 исправлена [следующим проходом](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md).
Q14-F027 (передача ошибок teardown), сроки firewall-команд и live Linux остаются открытыми.

**Общие firewall-проверки, 23 сентября 2026:**
[Q14-F026 / Q25-F018/F019](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md): сервер и Linux
kill-switch используют общий разбор presence/absence/errors; точечная очистка DNS
проверяет границу 1024 и продолжает TCP после отказа UDP. 17 новых host-тестов,
1016 Rust tests PASS; два новых Unix/Linux сценария только cross-check.
Q14-F027, сроки команд и реальные backend/runtime проверки остаются открытыми.

**Владение DNS firewall, 23 сентября 2026:**
[Q14-F028](../reports/AUDIT-Q14-DNS-OWNERSHIP.md): реестр worker сохраняет точные правила
при ошибке Drop/rollback; cleanup и новая установка повторяют очистку. Tokens защищают
новое поколение от старого lease; активные записи не участвуют в точечном retry.
12 новых host-тестов, 1028 Rust tests PASS; три adapter-регрессии отдельно сравнивают
baseline/fix. Q14-F027, persistent journal, deadlines и live Linux остаются открытыми.

**Итог остановки worker, 23 сентября 2026:**
[Q14-F027, частичное исправление](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md): окончательный
retry известных DNS/IPv6 sysctl leases влияет на Result и код выхода worker. Flush
статистики выполняется после ошибки сети; активные DNS leases обнаруживаются без удаления.
14 новых host-тестов, 1042 Rust tests PASS; восемь отдельных process-exit сценариев PASS.
Дополнительный проход передаёт ошибку worker через внешний supervisor при финальной
остановке: nonzero exit и принудительный kill больше не возвращают Ok. Ещё пять
host-тестов, итог этого этапа 1047 Rust tests PASS; три отдельные supervisor-регрессии PASS.
Следующий проход: [задачи профиля и TUN teardown](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md).
Ошибки shutdown JoinSet, TUN queue timeout/panic и удаления устройства теперь входят в
итог worker; 13 новых host-тестов, текущая матрица 1060 Rust tests PASS.
[Q14-F029/F030](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md): исправлены ложный успех
sysctl recovery и потеря существующего owner при неудачном повторном acquire;
1060 Rust tests и 7 отдельных fixture checks PASS.
Generic NAT, старые поколения/retry backoff, частичный IPv6 acquire, restart policy,
persistent journal и live Linux остаются открытыми.

### 19. DNS сервера и клиентов

**Код:** `qeli/src/server/dns.rs`, `qeli/src/server/dns/resolver.rs`, `qeli/src/client/dns.rs`, `qeli/src/transport_core/network.rs`.

UDP/TCP upstream, truncation fallback, timeouts, malformed packets, cache/eviction/blocklist. Full/split, resolved/resolv.conf и OS resolvers, leak v4/v6, failed apply до Connected, crash restore. Custom port/manual IPv6; DoT не объявляется реализованным при отказе валидатора.

**Имеющаяся обвязка/fixtures:** `scripts/test_dns_test_server.py`, `scripts/test_panel_route_dns.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Проход общего сетевого плана, 22–23 сентября 2026:**
[отчёт Q19/Q22](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md). Исправлены различия legacy/v2 DNS
и зависимость лимита маршрутов от порядка исключений; убрана устаревшая тестовая
реализация Linux DNS. Это проверка общего планировщика, не завершение всего модуля.
DNS proxy/cache, реальные OS apply/rollback и конкурентный lifecycle остаются открыты.

**Серверный DNS, 23 сентября 2026:** [отчёт Q19](../reports/AUDIT-Q19-DNS-PROXY.md).
Исправлены Q19-F004–F006: TTL NODATA после CNAME, проверка сжатых имён и failover
после TCP TC. Один production engine тестируется локально; 22 DNS-теста с UDP/TCP,
754 Rust-теста суммарно — PASS. Linux lifecycle, расширенные DNS-типы и нагрузочные
проверки остаются открытыми; общий статус раздела не закрыт.


**Кеш и подписанные обмены, 23 сентября 2026:**
[Продолжение Q19](../reports/AUDIT-Q19-DNS-CACHE.md) закрывает Q19-F007–F010:
лимит пакетов кеша 16 МиБ на профиль, пересылка TSIG/SIG(0) без изменения байтов
и кеширования, проверка заголовка/opcode запросов. У старого сценария панели
устранён запуск SSH при импорте. 36 DNS-тестов, 768 Rust-тестов
суммарно — PASS. Реальный Linux lifecycle, длительная конкурентная нагрузка/RSS,
расширенные EDNS/RDATA и внешний стенд подписанных обменов остаются открытыми;
раздел 19 сохраняет статус **IN_PROGRESS**.

**EDNS/RDATA, 23 сентября 2026:**
[Следующий проход Q19](../reports/AUDIT-Q19-DNS-EDNS.md) закрывает Q19-F011–F015:
TTL всех возвращаемых секций, структуру распространённых RDATA, проверку OPT/TLV
и BADVERS, сохранение расширенного RCODE при усечении, обход кеша для EDNS-опций
и новый OPT при обычном cache hit. 51 DNS-тест, включая IPv6 loopback UDP/TCP;
783 Rust-теста суммарно — PASS. Linux lifecycle, длительная нагрузка/RSS,
семантика DNSSEC/RRset, внешняя совместимость и OS DNS apply/rollback остаются открытыми.

**DNS-слушатели и cleanup, 23 сентября 2026:**
[проход Q14/Q19](../reports/AUDIT-Q14-Q19-LIFECYCLE.md): production UDP/TCP listeners
доступны host-тестам; удалён неиспользуемый ServerState. 7 listener + 52 resolver
теста: IPv4/IPv6, persistent/pipelined TCP, deadline, предел 512 соединений,
отмена запросов и повторное занятие портов. 798 Rust-тестов суммарно — PASS.
Общий lifecycle исправлен в Q14-F001–F002; реальный Linux runtime и длительная
нагрузка остаются открытыми.

**Восстановление старого resolver, 23 сентября 2026:**
[Q25-F006](../reports/AUDIT-Q25-DNS-RECOVERY.md): ошибки unlink/chmod и некорректный снимок
больше не считаются успешным восстановлением и не приводят к удалению записи восстановления.
Восемь новых Windows host-тестов проходят; три Unix-сценария только cross-check.
913 host Rust tests PASS. Live Linux DNS и передача ошибок нижележащей очистки ещё открыты.

**Передача ошибок очистки TUN/маршрутов/DNS, 23 сентября 2026:**
[Q25-F007–F009](../reports/AUDIT-Q25-TUN-CLEANUP.md): явная очистка и guards отката передают
ошибки в Linux retry loop через общий ограниченный журнал. Ошибка не становится успешной
остановкой по сигналу и не снимает включённый kill-switch. TunnelSetup владеет guard до ACK
ядра; тип terminal kick сохраняется при сопутствующих ошибках. Восемь новых host-тестов
проходят, два Linux adapter-теста только cross-check. 921 host Rust tests PASS. Live Linux
E2E, сроки выполнения команд и полное ожидание задач поколения ещё открыты.

**Системные команды TUN/DNS, 23 сентября 2026:**
[Q25-F016/F017](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md): 15 секунд на команду, полный
вывод с лимитом 16 МиБ на поток, завершение дочернего процесса и сохранение DNS marker
при отказе. Диагностика больше не обещает неподтверждённый rollback. 986 host Rust tests
PASS; два новых Linux process-group теста только cross-check. Маршруты/firewall, live
Linux и общий deadline shutdown остаются открытыми; статус раздела IN_PROGRESS.

**Очистка NAT, 23 сентября 2026:**
[Q14-F024/F025](../reports/AUDIT-Q14-NAT-CLEANUP.md): конечный проход по снимку правил,
проверка после удаления и диагностика ошибок с продолжением остальных правил/цепочек.
13 новых host-тестов, 999 Rust tests PASS; production Linux только cross-check.
Q14-F026 исправлена [следующим проходом](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md).
Q14-F027 (передача ошибок teardown), сроки firewall-команд и live Linux остаются открытыми.

**Общие firewall-проверки, 23 сентября 2026:**
[Q14-F026 / Q25-F018/F019](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md): сервер и Linux
kill-switch используют общий разбор presence/absence/errors; точечная очистка DNS
проверяет границу 1024 и продолжает TCP после отказа UDP. 17 новых host-тестов,
1016 Rust tests PASS; два новых Unix/Linux сценария только cross-check.
Q14-F027, сроки команд и реальные backend/runtime проверки остаются открытыми.

**Владение DNS firewall, 23 сентября 2026:**
[Q14-F028](../reports/AUDIT-Q14-DNS-OWNERSHIP.md): реестр worker сохраняет точные правила
при ошибке Drop/rollback; cleanup и новая установка повторяют очистку. Tokens защищают
новое поколение от старого lease; активные записи не участвуют в точечном retry.
12 новых host-тестов, 1028 Rust tests PASS; три adapter-регрессии отдельно сравнивают
baseline/fix. Q14-F027, persistent journal, deadlines и live Linux остаются открытыми.

**Итог остановки worker, 23 сентября 2026:**
[Q14-F027, частичное исправление](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md): окончательный
retry известных DNS/IPv6 sysctl leases влияет на Result и код выхода worker. Flush
статистики выполняется после ошибки сети; активные DNS leases обнаруживаются без удаления.
14 новых host-тестов, 1042 Rust tests PASS; восемь отдельных process-exit сценариев PASS.
Дополнительный проход передаёт ошибку worker через внешний supervisor при финальной
остановке: nonzero exit и принудительный kill больше не возвращают Ok. Ещё пять
host-тестов, итог этого этапа 1047 Rust tests PASS; три отдельные supervisor-регрессии PASS.
Следующий проход: [задачи профиля и TUN teardown](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md).
Ошибки shutdown JoinSet, TUN queue timeout/panic и удаления устройства теперь входят в
итог worker; 13 новых host-тестов, текущая матрица 1060 Rust tests PASS.
[Q14-F029/F030](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md): исправлены ложный успех
sysctl recovery и потеря существующего owner при неудачном повторном acquire;
1060 Rust tests и 7 отдельных fixture checks PASS.
Generic NAT, старые поколения/retry backoff, частичный IPv6 acquire, restart policy,
persistent journal и live Linux остаются открытыми.

### 20. DHCP и lease lifecycle

**Код:** `qeli/src/server/dhcp.rs`, `qeli/src/config/server.rs`.

DISCOVER/OFFER/REQUEST/ACK/NAK/RELEASE, bad requested_ip, duplicate xid/MAC, expiry и malformed options. NAK не расходует lease, пустой пул не underflow. Только поддерживаемая TAP/IPv4 конфигурация; проверить сохранение через панель.

**Имеющаяся обвязка/fixtures:** `qeli/tests/config_examples.rs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 21. TUN/TAP, IP, MTU/PMTU и фрагментация

**Код:** `qeli/src/tun`, `qeli/src/protocol/ip.rs`, `qeli/src/protocol/icmp.rs`, `qeli/src/protocol/data_frag.rs`, `qeli/src/protocol/udp_frag.rs`.

TUN host prefixes, TAP ARP/NDP/RA/DAD, unsupported EtherType/VLAN/multicast. MTU 1280/малый outer PMTU, spoofed PTB и смена пути. Reassembly duplicate/overlap/gap/order/expiry/ID reuse, memory cap; PCAP без непредусмотренной внешней фрагментации.

**Имеющаяся обвязка/fixtures:** `scripts/test_tap_ipv6_control_probe.py`, `qeli/fuzz/fuzz_targets/data_frag.rs`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Отмена shutdown TUN, 23 сентября 2026:** [Q25-F014](../reports/AUDIT-Q25-TUN-WORKERS.md).
Общий TunWorkers сохраняет владение Unix TUN/Wintun потоками до join, включая отмену
начатого shutdown и занятый blocking pool. Семь новых host-регрессий; 947 Rust tests PASS.
Unix-тест дескрипторов только кросс-компилирован. Реальные устройства/драйверы и остальные
сценарии раздела не проверены; полный аудит остаётся открытым.

**Системные команды TUN/DNS, 23 сентября 2026:**
[Q25-F016/F017](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md): 15 секунд на команду, полный
вывод с лимитом 16 МиБ на поток, завершение дочернего процесса и сохранение DNS marker
при отказе. Диагностика больше не обещает неподтверждённый rollback. 986 host Rust tests
PASS; два новых Linux process-group теста только cross-check. Маршруты/firewall, live
Linux и общий deadline shutdown остаются открытыми; статус раздела IN_PROGRESS.

### 22. Transport core, FFI/JNI и память

**Код:** `qeli/src/transport_core`, `qeli/include/qeli_transport_core.h`, `native-libs`.

Create/start/PREPARE/APPLY/COMMIT/stop/free, callbacks, buffers, queues, cancellation/generation. Stale handle, double-free, callback после dispose, partial failure, ABI/feature mismatch. Packaged cores должны соответствовать исходникам, а не только успешно загружаться.

**Имеющаяся обвязка/fixtures:** `scripts/test_native_repro.py`, `native-libs/provenance.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Проход общего сетевого плана, 22–23 сентября 2026:**
[отчёт Q19/Q22](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md). Исправлены различия legacy/v2 DNS
и зависимость лимита маршрутов от порядка исключений; убрана устаревшая тестовая
реализация Linux DNS. Это проверка общего планировщика, не завершение всего модуля.
DNS proxy/cache, реальные OS apply/rollback и конкурентный lifecycle остаются открыты.


**Владение задачами TCP и Linux path monitor, 23 сентября 2026:**
[Q25-F010/F011](../reports/AUDIT-Q25-TCP-TASKS.md): общий владелец закрывает создание задач
до abort/join. TCP reader/writer/pipeline и producers завершаются до сетевой очистки;
ошибка управляющего события также проходит teardown. Linux blocking-работы монитора
учитываются для TCP и UDP. 931 host Rust tests PASS; Linux только cross-check. Остальные
UDP-задачи, вложенные transport workers, полная отмена и сроки команд остаются открытыми.

**Владение UDP-задачами и порядок отката, 23 сентября 2026:**
[Q25-F012/F013](../reports/AUDIT-Q25-UDP-TASKS.md): active/candidate/draining receive,
candidate-connect и Linux-монитор принадлежат одной группе. Ошибка управляющего события
проходит штатную очистку; группа завершается до проверки/отката платформенного кандидата.
TaskHandle сохраняет обязанность join при отмене ожидания или переносе пути. Девять новых
регрессий; 940 host Rust tests PASS, Linux только cross-check. Открыты вложенные transport
workers, принудительная отмена, сроки команд и платформенные fault-injection сценарии.

**Отмена shutdown TUN, 23 сентября 2026:** [Q25-F014](../reports/AUDIT-Q25-TUN-WORKERS.md).
Общий TunWorkers сохраняет владение Unix TUN/Wintun потоками до join, включая отмену
начатого shutdown и занятый blocking pool. Семь новых host-регрессий; 947 Rust tests PASS.
Unix-тест дескрипторов только кросс-компилирован. Реальные устройства/драйверы и остальные
сценарии раздела не проверены; полный аудит остаётся открытым.

**Вложенные H2-задачи, 23 сентября 2026:** [Q25-F015](../reports/AUDIT-Q25-H2-TASKS.md).
TCP-группа создаётся до connect и ждёт driver/bridge; native runner сохраняет её при
отмене попытки. Девять новых регрессий, 956 Rust tests PASS; Linux только cross-check.
Серверный H2 проверен далее в [Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md).
Standalone H2, ранний platform rollback, UDP cancellation и deadlines остаются открытыми.

**Серверный H2 и pre-auth, 23 сентября 2026:**
[Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md): профиль ждёт вложенные H2-задачи перед
teardown; flush отказа сохраняет pre-auth slot до освобождения I/O. 13 новых регрессий,
969 Rust tests PASS. H2/ProfileTasks/semaphore проверены на host, production Linux только
кросс-компилирован. Остальные сценарии раздела и live Linux E2E остаются открытыми.

### 23. Роуминг, resume и CONTROL_V2

**Код:** `qeli/src/protocol/roaming.rs`, `qeli/src/protocol/control_v2.rs`, `qeli/src/transport_core`.

TCP make-before-break/UDP migration: proof/path validation, anti-amplification, grace expiry, replay/revoke и candidate races. NAT rebinding, family switch, Wi-Fi/LTE, sleep/wake, server restart/APPLY rollback, PMTU reset. Проверить межсерверные границы; одна константа PUSH_CONFIG не доказывает реализацию.

**Имеющаяся обвязка/fixtures:** `scripts/roaming_tcp_all_modes_netns_e2e.sh`, `scripts/roaming_udp_all_modes_netns_e2e.sh`, `scripts/roaming_mixed_version_netns_e2e.sh`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Владение задачами TCP и Linux path monitor, 23 сентября 2026:**
[Q25-F010/F011](../reports/AUDIT-Q25-TCP-TASKS.md): общий владелец закрывает создание задач
до abort/join. TCP reader/writer/pipeline и producers завершаются до сетевой очистки;
ошибка управляющего события также проходит teardown. Linux blocking-работы монитора
учитываются для TCP и UDP. 931 host Rust tests PASS; Linux только cross-check. Остальные
UDP-задачи, вложенные transport workers, полная отмена и сроки команд остаются открытыми.

**Владение UDP-задачами и порядок отката, 23 сентября 2026:**
[Q25-F012/F013](../reports/AUDIT-Q25-UDP-TASKS.md): active/candidate/draining receive,
candidate-connect и Linux-монитор принадлежат одной группе. Ошибка управляющего события
проходит штатную очистку; группа завершается до проверки/отката платформенного кандидата.
TaskHandle сохраняет обязанность join при отмене ожидания или переносе пути. Девять новых
регрессий; 940 host Rust tests PASS, Linux только cross-check. Открыты вложенные transport
workers, принудительная отмена, сроки команд и платформенные fault-injection сценарии.

**Вложенные H2-задачи, 23 сентября 2026:** [Q25-F015](../reports/AUDIT-Q25-H2-TASKS.md).
TCP-группа создаётся до connect и ждёт driver/bridge; native runner сохраняет её при
отмене попытки. Девять новых регрессий, 956 Rust tests PASS; Linux только cross-check.
Серверный H2 проверен далее в [Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md).
Standalone H2, ранний platform rollback, UDP cancellation и deadlines остаются открытыми.

### 24. Multipath, bonding и общий бюджет

**Код:** `qeli/src/transport_core/carrier.rs`, `qeli/src/transport_core/session.rs`, `qeli/src/server/handler.rs`.

JOIN proof, stream caps, asymmetric RTT/loss, отказ одного/всех путей, ordering/starvation. Bandwidth/quota/buffer cap не умножается на streams. Resume/reconnect/stream close сохраняют корректную сессию и освобождают лишние carriers.

**Имеющаяся обвязка/fixtures:** `scripts/test_multipath_bonding.py`, `scripts/test_multipath_resilience.py`, `scripts/test_multipath_allmodes.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Владение задачами TCP и Linux path monitor, 23 сентября 2026:**
[Q25-F010/F011](../reports/AUDIT-Q25-TCP-TASKS.md): общий владелец закрывает создание задач
до abort/join. TCP reader/writer/pipeline и producers завершаются до сетевой очистки;
ошибка управляющего события также проходит teardown. Linux blocking-работы монитора
учитываются для TCP и UDP. 931 host Rust tests PASS; Linux только cross-check. Остальные
UDP-задачи, вложенные transport workers, полная отмена и сроки команд остаются открытыми.

**Владение UDP-задачами и порядок отката, 23 сентября 2026:**
[Q25-F012/F013](../reports/AUDIT-Q25-UDP-TASKS.md): active/candidate/draining receive,
candidate-connect и Linux-монитор принадлежат одной группе. Ошибка управляющего события
проходит штатную очистку; группа завершается до проверки/отката платформенного кандидата.
TaskHandle сохраняет обязанность join при отмене ожидания или переносе пути. Девять новых
регрессий; 940 host Rust tests PASS, Linux только cross-check. Открыты вложенные transport
workers, принудительная отмена, сроки команд и платформенные fault-injection сценарии.

**Вложенные H2-задачи, 23 сентября 2026:** [Q25-F015](../reports/AUDIT-Q25-H2-TASKS.md).
TCP-группа создаётся до connect и ждёт driver/bridge; native runner сохраняет её при
отмене попытки. Девять новых регрессий, 956 Rust tests PASS; Linux только cross-check.
Серверный H2 проверен далее в [Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md).
Standalone H2, ранний platform rollback, UDP cancellation и deadlines остаются открытыми.

### 25. Linux CLI и восстановление сети

**Код:** `qeli/src/client`, `qeli/src/client_main.rs`, `qeli/src/hooks.rs`.

Endpoint route pin/same-LAN, full/split, include/exclude, leak policy/kill switch. Stop/SIGTERM/SIGKILL/reconnect/failed setup: before/after routes/DNS/firewall без удаления чужого состояния. Trusted hooks/password_command, subprocess deadlines, честный cleanup status.

**Имеющаяся обвязка/fixtures:** `scripts/test_gateway_nat.py`, `scripts/test_tun_reclaim.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Поставщик пароля и изоляция features, 23 сентября 2026:**
[Q25-F001 / Q33-F002 / Q14-F020 / Q34-F001](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md):
асинхронный password_command, deadline 30 секунд, полный stdout до 16 KiB, отброшенный
stderr и ошибки без секретов. Ранний SIGINT/SIGTERM отменяет и собирает поставщика;
watchers/sampler клиента имеют владельца. Исправлен server-only TUN gate, обе изолированные
features проверяются в CI. 883 host Rust tests PASS; четыре Linux-теста только cross-check.
Server-only check имеет 23 прежних transport dead-code warnings. Лимиты password_file,
финальный drain клиента и Linux runtime/release checks ещё открыты.

**Файловый пароль и финальный статус, 23 сентября 2026:**
[Q25-F002 / Q14-F021](../reports/AUDIT-Q25-PASSWORD-FILES.md): общий zeroizing-буфер 16 KiB
для файла/команды, одна управляемая blocking-задача чтения обычного файла, поддержка symlink,
отказ FIFO и ожидание активного I/O при штатном stop/deadline. Final пишется после join
watchers/sampler, включая ошибки после инициализации reporter. 895 host Rust tests PASS;
два Unix-теста только cross-check. Неотменяемый I/O может превышать бюджет 30 секунд.
Startup/network rollback, мониторинг фоновых ошибок и Linux E2E ещё открыты.

**Fail-closed очистка сети, 23 сентября 2026:**
[Q25-F003](../reports/AUDIT-Q25-NETWORK-CLEANUP.md): forwarding/NAT cleanup должен
завершиться успешно до снятия включённого kill-switch; ошибка сохраняет защиту и её причину.
899 host Rust tests PASS, включая четыре переносимых fault-injection сценария. Linux
пока только cross-check. Разбор begin_connection записан ниже; live firewall/E2E ещё открыты.

**Ошибки жизненного цикла ядра, 23 сентября 2026:**
[Q25-F004/F005](../reports/AUDIT-Q25-CORE-LIFECYCLE.md): ошибка запуска ядра проходит через
cleanup/post_down; ошибка остановки завершается отказом и сохраняет включённый kill-switch,
при этом очистка forwarding выполняется. Шесть новых host-регрессий, включая реальный отказ
ClientCore при полной очереди. 905 host Rust tests PASS; Linux только cross-check.
Live Linux lifecycle/firewall и полный rollback маршрутов/DNS ещё открыты.


**Восстановление старого resolver, 23 сентября 2026:**
[Q25-F006](../reports/AUDIT-Q25-DNS-RECOVERY.md): ошибки unlink/chmod и некорректный снимок
больше не считаются успешным восстановлением и не приводят к удалению записи восстановления.
Восемь новых Windows host-тестов проходят; три Unix-сценария только cross-check.
913 host Rust tests PASS. Live Linux DNS и передача ошибок нижележащей очистки ещё открыты.

**Передача ошибок очистки TUN/маршрутов/DNS, 23 сентября 2026:**
[Q25-F007–F009](../reports/AUDIT-Q25-TUN-CLEANUP.md): явная очистка и guards отката передают
ошибки в Linux retry loop через общий ограниченный журнал. Ошибка не становится успешной
остановкой по сигналу и не снимает включённый kill-switch. TunnelSetup владеет guard до ACK
ядра; тип terminal kick сохраняется при сопутствующих ошибках. Восемь новых host-тестов
проходят, два Linux adapter-теста только cross-check. 921 host Rust tests PASS. Live Linux
E2E, сроки выполнения команд и полное ожидание задач поколения ещё открыты.

**Владение задачами TCP и Linux path monitor, 23 сентября 2026:**
[Q25-F010/F011](../reports/AUDIT-Q25-TCP-TASKS.md): общий владелец закрывает создание задач
до abort/join. TCP reader/writer/pipeline и producers завершаются до сетевой очистки;
ошибка управляющего события также проходит teardown. Linux blocking-работы монитора
учитываются для TCP и UDP. 931 host Rust tests PASS; Linux только cross-check. Остальные
UDP-задачи, вложенные transport workers, полная отмена и сроки команд остаются открытыми.

**Владение UDP-задачами и порядок отката, 23 сентября 2026:**
[Q25-F012/F013](../reports/AUDIT-Q25-UDP-TASKS.md): active/candidate/draining receive,
candidate-connect и Linux-монитор принадлежат одной группе. Ошибка управляющего события
проходит штатную очистку; группа завершается до проверки/отката платформенного кандидата.
TaskHandle сохраняет обязанность join при отмене ожидания или переносе пути. Девять новых
регрессий; 940 host Rust tests PASS, Linux только cross-check. Открыты вложенные transport
workers, принудительная отмена, сроки команд и платформенные fault-injection сценарии.

**Отмена shutdown TUN, 23 сентября 2026:** [Q25-F014](../reports/AUDIT-Q25-TUN-WORKERS.md).
Общий TunWorkers сохраняет владение Unix TUN/Wintun потоками до join, включая отмену
начатого shutdown и занятый blocking pool. Семь новых host-регрессий; 947 Rust tests PASS.
Unix-тест дескрипторов только кросс-компилирован. Реальные устройства/драйверы и остальные
сценарии раздела не проверены; полный аудит остаётся открытым.

**Вложенные H2-задачи, 23 сентября 2026:** [Q25-F015](../reports/AUDIT-Q25-H2-TASKS.md).
TCP-группа создаётся до connect и ждёт driver/bridge; native runner сохраняет её при
отмене попытки. Девять новых регрессий, 956 Rust tests PASS; Linux только cross-check.
Серверный H2 проверен далее в [Q14-F022/F023](../reports/AUDIT-Q14-H2-TASKS.md).
Standalone H2, ранний platform rollback, UDP cancellation и deadlines остаются открытыми.

**Системные команды TUN/DNS, 23 сентября 2026:**
[Q25-F016/F017](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md): 15 секунд на команду, полный
вывод с лимитом 16 МиБ на поток, завершение дочернего процесса и сохранение DNS marker
при отказе. Диагностика больше не обещает неподтверждённый rollback. 986 host Rust tests
PASS; два новых Linux process-group теста только cross-check. Маршруты/firewall, live
Linux и общий deadline shutdown остаются открытыми; статус раздела IN_PROGRESS.

**Общие firewall-проверки, 23 сентября 2026:**
[Q14-F026 / Q25-F018/F019](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md): сервер и Linux
kill-switch используют общий разбор presence/absence/errors; точечная очистка DNS
проверяет границу 1024 и продолжает TCP после отказа UDP. 17 новых host-тестов,
1016 Rust tests PASS; два новых Unix/Linux сценария только cross-check.
Q14-F027, сроки команд и реальные backend/runtime проверки остаются открытыми.

### 26. Общий C# и managed/native граница

**Код:** `qeli-shared/QeliShared`, `qeli-shared/QeliConformance`.

Rust validation parity, import/export/storage, lifetime handles/callbacks. Conformance с обязательными fixtures и platform selftests. Мёртвые managed codecs проверять по OS/features/reflection до удаления; build/selftest не заменяет подключение клиента.

**Имеющаяся обвязка/fixtures:** `conformance/README.md`, `.github/workflows/ci.yml`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 27. Windows: GUI, служба и драйверы

**Код:** `qeli-win/QeliWin`, `qeli/src/transport_core/wintun.rs`.

LocalSystem IPC/ACL/SID, DPAPI, protected directories, atomic service profile и DLL loading. В Windows VM: Wintun/WinDivert/per-app, routes/DNS/firewall, stop во время connect, sleep/wake, boot service. Cleanup failures не маскируются; UAF/double-close отсутствуют.

**Имеющаяся обвязка/fixtures:** `scripts/e2e_windows_native.py`, `scripts/verify_windows_drivers.ps1`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Отмена shutdown TUN, 23 сентября 2026:** [Q25-F014](../reports/AUDIT-Q25-TUN-WORKERS.md).
Общий TunWorkers сохраняет владение Unix TUN/Wintun потоками до join, включая отмену
начатого shutdown и занятый blocking pool. Семь новых host-регрессий; 947 Rust tests PASS.
Unix-тест дескрипторов только кросс-компилирован. Реальные устройства/драйверы и остальные
сценарии раздела не проверены; полный аудит остаётся открытым.

### 28. macOS: daemon, utun, pf и Network Extension

**Код:** `qeli-mac/QeliMac`, `qeli-mac/per-app`.

Daemon IPC/owner/mode, Keychain, selected profile, Intel/ARM ABI и DNS journal. Реальный Mac: чужие pf/nat/rdr anchors, per-app entitlements, DNS leaks, reconnect, crash и sleep/wake. GUI/daemon/Network Extension проверяются отдельно.

**Имеющаяся обвязка/fixtures:** `qeli-mac/README.md`, `.github/workflows/ci.yml`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 29. Android: VpnService, JNI и lifecycle

**Код:** `qeli-android/app`.

Protect/TUN retention/generation при reconnect/cancel/stop. Keystore, INI migration, encrypted backup/lost-key recovery; manifest exports/deep links/boot. Устройство: Wi-Fi/LTE, always-on/lockdown, Doze, process kill, IPv6-only/NAT64, Release/R8.

**Имеющаяся обвязка/fixtures:** `scripts/roaming_android_sleep_wake_gate.py`, `scripts/roaming_android_udp_grace_expiry_gate.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 30. iOS: PacketTunnel, Swift и MDM

**Код:** `qeli-ios/QeliCore`, `qeli-ios/QeliPacketTunnel`, `qeli-ios/QeliIOS`, `qeli-ios/MDM`, `qeli-ios/QeliIOSTests`.

Exactly-once start/stop completion, generation/cancel, Keychain/app groups, extension memory. Устройство: On Demand, sleep/wake, captive portal, NAT64/DNS, per-app/MDM и rollback settings. Simulator build, signed IPA и physical evidence — разные статусы.

**Имеющаяся обвязка/fixtures:** `qeli-ios/PARITY.md`, `scripts/test_verify_ios_ipa.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 31. OpenWrt, LuCI и Keenetic

**Код:** `qeli-openwrt`, `scripts/build_keenetic.py`.

UCI → INI escaping/shell injection, LuCI ACL, secrets на flash, init/procd, upgrade/rollback. ARM/MIPS/mipsel, endian/32-bit ABI, client-only features. Реальный router: WAN renewal/reboot, DNS/firewall/hooks, memory/throughput; cross-build не device test.

**Имеющаяся обвязка/fixtures:** `scripts/keenetic_verify.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 32. Метрики, usage, логи и уведомления

**Код:** `qeli/src/server/metrics.rs`, `qeli/src/server/usage.rs`, `qeli/src/server/notify.rs`, `qeli/src/server/roaming_metrics.rs`, `qeli/src/trace.rs`, `qeli/src/web/api/logs.rs`.

Counters/quota/session accounting при reconnect/reap/crash, corrupt store, bounded log/SSE/backpressure. Notify INI/token/load races, SSRF/DNS rebinding/redirect/timeout/rate limit. Логи не раскрывают credentials, disabled trace не меняет hot path.

**Имеющаяся обвязка/fixtures:** `scripts/test_roaming_control_stats.py`, `scripts/test_blocked_settings.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Владение уведомлениями, 23 сентября 2026:**
[Q14-F018 / Q32-F001](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md): до 128 принятых отправок,
8 активных запросов на процесс, общий лимит проб панели, ограниченные payloads и drain
до 10 секунд после завершения производителей. Detached-обёртки уведомлений удалены.
864 host Rust tests PASS; Linux только all-targets cross-check. Владение panel/metrics/
autostart supervisor, доверие конфигу и Linux E2E ещё открыты.

### 33. Установка, обновление, файловые права и hooks

**Код:** `qeli/debian`, `qeli/src/server/update.rs`, `qeli/src/util.rs`, `qeli/src/hooks.rs`, `qeli/src/config_source.rs`, `release/docker`.

Fresh install/upgrade/downgrade/remove, systemd sandbox, identity/users preservation, checksums/attestation, atomic replace. Docker digest/recreate/health/rollback. File locks/symlinks/hardlinks/owners/ENOSPC, PATH hijack, panel/restore command injection и SSH timeouts.

**Имеющаяся обвязка/fixtures:** `scripts/test_ssh_run.ps1`, `scripts/release_preflight.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Доверие прочитанному конфигу, 23 сентября 2026:**
[Q14-F019 / Q33-F001](../reports/AUDIT-Q14-Q33-CONFIG-TRUST.md): владелец/права и данные
для парсера получаются из одного дескриптора; исходное разрешение не меняется при
повторах профиля и не перепроверяет путь. Очистка готового поколения сохраняет команду
и окружение после удаления/замены конфига. 874 host Rust tests PASS; четыре новых Unix/
Linux-теста только cross-checked. Лимиты password_command, владение startup-задачами,
installer/update/restore и Linux runtime integration ещё открыты.

**Поставщик пароля и изоляция features, 23 сентября 2026:**
[Q25-F001 / Q33-F002 / Q14-F020 / Q34-F001](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md):
асинхронный password_command, deadline 30 секунд, полный stdout до 16 KiB, отброшенный
stderr и ошибки без секретов. Ранний SIGINT/SIGTERM отменяет и собирает поставщика;
watchers/sampler клиента имеют владельца. Исправлен server-only TUN gate, обе изолированные
features проверяются в CI. 883 host Rust tests PASS; четыре Linux-теста только cross-check.
Server-only check имеет 23 прежних transport dead-code warnings. Лимиты password_file,
финальный drain клиента и Linux runtime/release checks ещё открыты.

### 34. CI, зависимости, native provenance и релиз

**Код:** `qeli/Cargo.toml`, `qeli/Cargo.lock`, `.github/workflows`, `native-libs`, `release/certification`.

Feature/debug/release/jemalloc matrix, lockfiles, актуальные CVE/licenses и pinned Actions/SDK. Независимая A/B rebuild cores, hashes/ABI/provenance и signatures драйверов/APK/IPA. Certification опирается на реальные evidence текущего SHA, не на ручную замену digest/status.

**Имеющаяся обвязка/fixtures:** `scripts/test_native_recipes.py`, `scripts/test_native_repro.py`, `scripts/test_release_certification.py`, `scripts/release_certification.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: IN_PROGRESS.**

**Поставщик пароля и изоляция features, 23 сентября 2026:**
[Q25-F001 / Q33-F002 / Q14-F020 / Q34-F001](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md):
асинхронный password_command, deadline 30 секунд, полный stdout до 16 KiB, отброшенный
stderr и ошибки без секретов. Ранний SIGINT/SIGTERM отменяет и собирает поставщика;
watchers/sampler клиента имеют владельца. Исправлен server-only TUN gate, обе изолированные
features проверяются в CI. 883 host Rust tests PASS; четыре Linux-теста только cross-check.
Server-only check имеет 23 прежних transport dead-code warnings. Лимиты password_file,
финальный drain клиента и Linux runtime/release checks ещё открыты.

### 35. Fuzzing, concurrency, DoS и soak

**Код:** `qeli/fuzz`, `scripts/stability_gate.py`.

Fuzz INI/hello/packet/WS/realtls/QUIC/IP/fragments/roaming с сохранением corpus. Failure injection каждой acquire/apply/save, гонки stop/auth/reload/reap. Soak 30–60 мин, перед release ≥8 ч: RSS/fd/tasks/leases/rules с churn; пороги роста задать заранее.

**Имеющаяся обвязка/fixtures:** `qeli/fuzz/README.md`, `scripts/roaming_udp_resource_soak_netns_gate.sh`, `scripts/linux_roaming_release_soak.sh`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 36. Бенчмарки и методика измерения

**Код:** `scripts/benchmark.py`, `qeli/src/packet_bench_main.rs`, `test`, `release/benchmark_results.json`.

Зафиксировать SHA/binaries, CPU/governor/affinity/VM contention, MTU/mode и background load. P=1/P=4, up/down/bidir, inner/outer v4/v6, TCP/UDP goodput, p50/p95/p99, loss/jitter, CPU/RSS/auth rate. ≥3 независимых повтора, median+spread/raw results; dev не подменяется числами 0.8.0.

**Имеющаяся обвязка/fixtures:** `scripts/perf_combined_load.py`, `scripts/bench_bonding.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

### 37. Документация, тестовая обвязка и мёртвый код

**Код:** `docs`, `qeli/config`, `scripts`, `conformance`, `site`.

Keys/defaults/errors против runtime, полные examples через check-config, RU/EN, stable/dev, ABI/benchmark dates. Legacy JSON config, unused dependencies/helpers/routes/flags и неподключённые тесты. Dead code подтвердить по всем OS/features/FFI/reflection/generators; регрессия должна падать на старом дефекте.

**Имеющаяся обвязка/fixtures:** `scripts/check_docs.py`, `scripts/check_panel.py`, `scripts/test_site_docs.js`, `scripts/sync_version.py`.

- [ ] Review и мёртвый код.
- [ ] Штатные, граничные и негативные сценарии.
- [ ] Отказы и конкуренция.
- [ ] Интеграция и целевая платформа.
- [ ] Исправления, повторная проверка и evidence.

**Статус: TODO.**

## 7. Команды исходной точки и последующих прогонов

Команды ниже выполняются из корня checkout. Использовать toolchain проекта и писать
вывод каждого запуска в отдельный лог. Они не разрешают автоматически запускать
произвольные сетевые/deploy-скрипты из предыдущих разделов.

```text
python scripts/check_docs.py
python scripts/check_panel.py
node scripts/test_panel_editors.cjs
node scripts/test_site_docs.js
python -m unittest discover -s scripts -p "test_native_*.py"
python scripts/sync_version.py
python native-libs/provenance.py --check
python scripts/release_certification.py --quiet
```

Полный серверный набор — **на Linux**; запуск той же команды на Windows имеет другое
cfg-покрытие. Зафиксировать точный target и features:

```text
cargo test --locked --manifest-path qeli/Cargo.toml --workspace -- --test-threads=1
cargo clippy --locked --manifest-path qeli/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path qeli/Cargo.toml --features transport-core-ffi transport_core -- --test-threads=1
cargo run --locked --manifest-path qeli/Cargo.toml --features conformance-gen --bin gen-conformance -- --check
```

Команды Windows/macOS/Android/iOS builds/selftests брать из текущего
[CI](../../../.github/workflows/ci.yml), сохраняя environment/fixture guards. Старое
исключение Clippy в отчётах не переносить автоматически: указать toolchain и отдельную
причину для каждого исключения. Не называть cross-check выполнением Linux runtime.

## 8. Закрытие раздела и всего цикла

Находки нового цикла именуются `Q<раздел>-F<номер>`, например `Q01-F001`, чтобы не
смешивать их с повторяющимися A-номерами старых аудитов. Для каждой — impact,
предусловия, reachable path, воспроизведение, fix и regression evidence. P0/P1 —
блокеры затронутого выпуска до исправления либо явно зафиксированного решения;
P2/P3 сохраняются как конкретные задачи, не исчезают из-за зелёных сборок.

В конце каждого раздела обновить строку в реестре, приложить результат и назвать
следующий раздел. Изменение зависимого контракта открывает регрессию у его consumers.
Перед итоговым PASS: все обязательные разделы закрыты, блокеры разрешены, все
неприменимые случаи обоснованы, native/source SHA согласованы, физические сценарии
подтверждены, benchmark воспроизводим и docs отражают пределы поддержки.

**Ближайшая работа:** продолжить 22–25: вложенные H2/transport workers и отмена
TUN shutdown, затем сроки системных команд и platform rollback/ACK. Сохраняется очередь
незакрытых 01–07, Linux E2E restart/restore/manual+NDP и платформенной сертификации.
Новый полный бенчмарк выполняется после стабилизации исправлений.
