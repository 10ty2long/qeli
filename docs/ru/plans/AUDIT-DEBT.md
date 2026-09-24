# Техдолг начатых аудитов

<!-- normative-sync: audit-debt-v1 -->

Дата сверки: 24 сентября 2026. По запросу пользователя новые разделы полного аудита
приостановлены до закрытия этого реестра. Это **15 групп обязательств**, а не 15 найденных
багов и не процент готовности всех 37 разделов. Источник: 57 исходных `AUDIT-Q*.md`
и план `CLIENT-CONFIG-CORE.md`; повторяющиеся ограничения объединены.

`DONE` допустим только после исправления/обоснования и нужной проверки. `BLOCKED`
обозначает недоступную внешнюю предпосылку, а не успех. Стенд Linux и Android-эмулятор
пользователь предоставил 24 сентября; старое ограничение «нет Linux» больше не действует.
Подключение к двум Linux VM подтверждено; работающий сервер и его файлы не заменялись.

[Полный план](FULL-SYSTEM-AUDIT.md) · [Общее конфигурационное ядро](CLIENT-CONFIG-CORE.md)

| ID | Разделы | Статус | Долг | Критерий закрытия / текущее свидетельство |
|---|---|---|---|---|
| D01 | 14/17/18/25 | DONE | Ошибки NAT и старого поколения | Точные спецификации правил до изменения firewall; retry и итоговая ошибка; запрет restart после неполной очистки; возврат IPv4 forwarding. Unit/cross, 18 native и 8 worker E2E PASS; baseline IPv4 leak воспроизведён. [Отчёт](../reports/AUDIT-Q14-RETAINED-CLEANUP.md). |
| D02 | 14/25 | DONE | Внутренние границы sysctl | Lock/I/O/context, trusted directory, namespace fd pins, исходный per-interface fd/witness и network_cookie v4 проверены. 1995 Linux + 32 privileged + 8 lifecycle и SIGKILL/mismatch worker E2E PASS. Потерянный interface witness сохраняется для ручного recovery; общий persistent firewall/DNS/routes остаётся D04. [Отчёт](../reports/AUDIT-Q25-NAMESPACE-GENERATION.md). |
| D03 | 22/25 | DONE | Самостоятельный kill-switch | Закреплённый namespace, сохранённый владелец точных семейств, fail-closed reconnect и безопасная смена адреса. 11 portable + 2 native регрессии, реальные счётчики IPv4/IPv6 и 2 отказа baseline. [Отчёт](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md). |
| D04 | 14/19/22/25 | IN_PROGRESS | Восстановление после crash | Server exact journal, DNS v2, guarded kill-switch restart и persistent physical client routes проверены. Остаются legacy global DNS, live persistent TUN и полная mixed firewall матрица. [Routes](../reports/AUDIT-Q25-ROUTE-JOURNAL.md), [server](../reports/AUDIT-Q14-FIREWALL-JOURNAL.md), [DNS](../reports/AUDIT-Q25-DNS-MARKER-STORAGE.md), [kill-switch](../reports/AUDIT-Q25-KILL-SWITCH-REBUILD.md). |
| D05 | 05/14/25 | IN_PROGRESS | Срок всей операции и блокировки | Убрать синхронный preflight из async handler/долгого config lock; ограничить последовательности команд и ожидания; проверить отмену и доступность соседних запросов. |
| D06 | 21/22/23/25 | IN_PROGRESS | Контекст внешних сетевых ресурсов | Проверить WAN identity, resolved/bus context, sysfs/procfs и attach/name-контракт; process-global DNS/carrier state, dynamic IPv6. Зафиксировать поддерживаемые комбинации. |
| D07 | 01/05/09/11 | TODO | Серверный конфиг в runtime | Таблица field → parse/validate/runtime/serialize; malformed/oversized input; check-config/startup/SIGHUP/HTTP save/Quick Start с сохранением действующего состояния при отказе. |
| D08 | 02/24/27 | IN_PROGRESS | Общие клиентские конфиги | Проверить весь контракт 81+3 полей, INI/import/URI/QR/form/store/reconnect через реальные адаптеры; fuzz/budget и конкурентное редактирование. |
| D09 | 14/15/25/32/33 | IN_PROGRESS | Linux lifecycle и системные отказы | Выполнить Linux tests для flock/permissions/control/hooks/process groups, worker/services/TUN/route/DNS; сохранить stdout, exit, SHA и before/after. Привилегированные ignored tests запускать явно. |
| D10 | 17/18/19/21/22/23 | IN_PROGRESS | Сетевая интеграционная матрица | Проверить off/manual/route/nat66 × NDP, DNS UDP/TCP, multiprofile, iptables/nft/firewalld, setup rollback/stop/restart и сохранение чужих ресурсов. |
| D11 | 00/24/27/34 | IN_PROGRESS | Актуальные native cores и provenance | Из чистого commit пересобрать изменённые ядра по закреплённым рецептам, сравнить A/B, обновить копии и настоящие provenance; проверить ABI/exports и пакеты. |
| D12 | 24/25/27/34 | IN_PROGRESS | Платформенное подтверждение | Android: 154 JVM + 6 API 34/x86_64 instrumentation PASS со свежим JNI; итоговый снимок ещё требуется. Windows VM, Mac/Xcode/iOS и router runtime **SKIPPED по решению пользователя 24 сентября 2026**: стендов не будет. Эти платформы не сертифицированы; это исключение из текущего объёма, не PASS. |
| D13 | 14/19/22/25 | IN_PROGRESS | Удержание ресурсов под нагрузкой | Измерить fd/tasks/threads/TUN/routes/firewall/journals/RSS до и после churn/reconnect/stop, включая отказы и несколько профилей; конечный deadline и критерии отсутствия роста. |
| D14 | 00/34 | TODO | Текущий benchmark и certification | После корректности выполнить воспроизводимый benchmark нужных режимов с текущим SHA, окружением и метриками; собрать certification только из фактических результатов. Старые результаты 0.8.0 не закрывают 0.8.2. |
| D15 | Все начатые разделы | IN_PROGRESS | Согласование evidence и документации | Сопоставить старые открытые пункты с поздними fixes; проверить применимость патчей, diff/commit и RU/EN ссылки. Каждый долг закрывать отдельным результатом, не числом коммитов. |

## Порядок закрытия

1. D01–D06: код и регрессии подтверждённых дефектов; D09 выполнять параллельно доступной локальной работе.
2. D07–D10: runtime/контракт и отказы на исправленном снимке.
3. D11–D13: чистая сборка, клиенты/устройства и измерение ресурсов.
4. D14–D15: текущие измерения, сверка пакетов/отчётов и итоговый перечень закрытых обязательств.

Изменение privileged-ресурса сторонним root между проверкой и записью не обещает
атомарной защиты; это предел интерфейса ОС, а не автоматически новый функциональный
дефект. Однако потеря identity, ошибка команды и неопределённый исход должны сохранять
сведения для восстановления и не давать ложный успешный результат.

## Источники

- [AUDIT-Q01-SERVER-INI](../reports/AUDIT-Q01-SERVER-INI.md)
- [AUDIT-Q02-CLIENT-PARSERS](../reports/AUDIT-Q02-CLIENT-PARSERS.md)
- [AUDIT-Q05-PREFLIGHT](../reports/AUDIT-Q05-PREFLIGHT.md)
- [AUDIT-Q14-CONTROL](../reports/AUDIT-Q14-CONTROL.md)
- [AUDIT-Q14-DNS-OWNERSHIP](../reports/AUDIT-Q14-DNS-OWNERSHIP.md)
- [AUDIT-Q14-H2-TASKS](../reports/AUDIT-Q14-H2-TASKS.md)
- [AUDIT-Q14-HOOKS](../reports/AUDIT-Q14-HOOKS.md)
- [AUDIT-Q14-IPV6-PARTIAL-ACQUIRE](../reports/AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md)
- [AUDIT-Q14-NAT-CLEANUP](../reports/AUDIT-Q14-NAT-CLEANUP.md)
- [AUDIT-Q14-NAT-COMMANDS](../reports/AUDIT-Q14-NAT-COMMANDS.md)
- [AUDIT-Q14-OWNED-SHUTDOWN](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md)
- [AUDIT-Q14-PROFILE-SHUTDOWN](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md)
- [AUDIT-Q14-Q15-WORKER-USAGE](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md)
- [AUDIT-Q14-Q19-LIFECYCLE](../reports/AUDIT-Q14-Q19-LIFECYCLE.md)
- [AUDIT-Q14-Q25-FIREWALL-CHECKS](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md)
- [AUDIT-Q14-Q32-NOTIFICATIONS](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md)
- [AUDIT-Q14-Q33-CONFIG-TRUST](../reports/AUDIT-Q14-Q33-CONFIG-TRUST.md)
- [AUDIT-Q14-SUPERVISOR](../reports/AUDIT-Q14-SUPERVISOR.md)
- [AUDIT-Q14-SYSCTL-RECOVERY](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md)
- [AUDIT-Q19-DNS-CACHE](../reports/AUDIT-Q19-DNS-CACHE.md)
- [AUDIT-Q19-DNS-EDNS](../reports/AUDIT-Q19-DNS-EDNS.md)
- [AUDIT-Q19-DNS-PROXY](../reports/AUDIT-Q19-DNS-PROXY.md)
- [AUDIT-Q19-Q22-NETWORK-PLAN](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md)
- [AUDIT-Q25-CLIENT-COMMANDS](../reports/AUDIT-Q25-CLIENT-COMMANDS.md)
- [AUDIT-Q25-CLIENT-NAMESPACE](../reports/AUDIT-Q25-CLIENT-NAMESPACE.md)
- [AUDIT-Q25-CORE-LIFECYCLE](../reports/AUDIT-Q25-CORE-LIFECYCLE.md)
- [AUDIT-Q25-CREDENTIAL-COMMANDS](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md)
- [AUDIT-Q25-DNS-LEASES](../reports/AUDIT-Q25-DNS-LEASES.md)
- [AUDIT-Q25-DNS-RECOVERY](../reports/AUDIT-Q25-DNS-RECOVERY.md)
- [AUDIT-Q25-EXIT-OWNERSHIP](../reports/AUDIT-Q25-EXIT-OWNERSHIP.md)
- [AUDIT-Q25-GATEWAY-IDENTITY](../reports/AUDIT-Q25-GATEWAY-IDENTITY.md)
- [AUDIT-Q25-GATEWAY-ROLLBACK](../reports/AUDIT-Q25-GATEWAY-ROLLBACK.md)
- [AUDIT-Q25-GATEWAY-WAN](../reports/AUDIT-Q25-GATEWAY-WAN.md)
- [AUDIT-Q25-H2-TASKS](../reports/AUDIT-Q25-H2-TASKS.md)
- [AUDIT-Q25-KILL-SWITCH-LIFETIME](../reports/AUDIT-Q25-KILL-SWITCH-LIFETIME.md)
- [AUDIT-Q25-NETWORK-CLEANUP](../reports/AUDIT-Q25-NETWORK-CLEANUP.md)
- [AUDIT-Q25-PASSWORD-FILES](../reports/AUDIT-Q25-PASSWORD-FILES.md)
- [AUDIT-Q25-PATH-MONITOR](../reports/AUDIT-Q25-PATH-MONITOR.md)
- [AUDIT-Q25-ROUTE-IDENTITY](../reports/AUDIT-Q25-ROUTE-IDENTITY.md)
- [AUDIT-Q25-ROUTE-OUTCOME](../reports/AUDIT-Q25-ROUTE-OUTCOME.md)
- [AUDIT-Q25-ROUTE-OWNERSHIP](../reports/AUDIT-Q25-ROUTE-OWNERSHIP.md)
- [AUDIT-Q25-ROUTE-PENDING](../reports/AUDIT-Q25-ROUTE-PENDING.md)
- [AUDIT-Q25-ROUTE-POSTCONDITIONS](../reports/AUDIT-Q25-ROUTE-POSTCONDITIONS.md)
- [AUDIT-Q25-ROUTE-SCOPE](../reports/AUDIT-Q25-ROUTE-SCOPE.md)
- [AUDIT-Q25-SETUP-FLUSH](../reports/AUDIT-Q25-SETUP-FLUSH.md)
- [AUDIT-Q25-SETUP-IDENTITY](../reports/AUDIT-Q25-SETUP-IDENTITY.md)
- [AUDIT-Q25-SYSCTL-NAMESPACE](../reports/AUDIT-Q25-SYSCTL-NAMESPACE.md)
- [AUDIT-Q25-SYSCTL-OWNER-EVIDENCE](../reports/AUDIT-Q25-SYSCTL-OWNER-EVIDENCE.md)
- [AUDIT-Q25-SYSTEM-COMMANDS](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md)
- [AUDIT-Q25-TCP-TASKS](../reports/AUDIT-Q25-TCP-TASKS.md)
- [AUDIT-Q25-TUN-ADMISSION](../reports/AUDIT-Q25-TUN-ADMISSION.md)
- [AUDIT-Q25-TUN-ATTACH](../reports/AUDIT-Q25-TUN-ATTACH.md)
- [AUDIT-Q25-TUN-CLEANUP](../reports/AUDIT-Q25-TUN-CLEANUP.md)
- [AUDIT-Q25-TUN-LIFETIME](../reports/AUDIT-Q25-TUN-LIFETIME.md)
- [AUDIT-Q25-TUN-WORKERS](../reports/AUDIT-Q25-TUN-WORKERS.md)
- [AUDIT-Q25-TUNNEL-ROUTES](../reports/AUDIT-Q25-TUNNEL-ROUTES.md)
- [AUDIT-Q25-UDP-TASKS](../reports/AUDIT-Q25-UDP-TASKS.md)

D02/D05: [чтение sysctl-журнала и lock waits](../reports/AUDIT-Q25-SYSCTL-JOURNAL-IO.md) исправлены в описанных границах; остальные критерии строк остаются открыты.

D03: [kill-switch namespace / reconnect](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md).

D02/D06: [namespace-aware link observation](../reports/AUDIT-Q25-LINK-OBSERVATION.md).

D05: [async preflight и время жизни транзакции панели](../reports/AUDIT-Q05-PANEL-TRANSACTIONS.md). Бюджет backup/restore закрыт следующим этапом; остальные сетевые последовательности открыты.

Обновление D05/D09: [бюджет backup/restore и полнота снимка](../reports/AUDIT-Q05-ARCHIVE-BUDGET.md). Остальные сетевые последовательности и filesystem fault E2E остаются открытыми.

D02: [guard внутренних границ sysctl](../reports/AUDIT-Q25-SYSCTL-CONTEXT-IO.md); durable namespace identity и исходный интерфейс ещё открыты; parent trust закрыт последующим этапом.

D02/D05/D09: [атомарная запись состояния](../reports/AUDIT-Q25-ATOMIC-STATE.md) очищает частичные временные файлы и синхронизирует каталог на Unix; реальные partial-write/fsync fault probes PASS. Остальные критерии этих групп остаются открыты.

D02/D05/D09: [каталог состояния и целостность lock](../reports/AUDIT-Q25-STATE-DIRECTORY.md). Parent trust закрыт в описанных границах; durable namespace identity и исходное поколение интерфейса остаются D02. Группы целиком ещё не закрыты.

D08/D11/D12: [Android JNI и emulator runtime](../reports/AUDIT-Q34-ANDROID-RUNTIME.md): исправлены cwd/API flag cargo-ndk, удалён устаревший JSON-config harness; 154 JVM + 6 instrumentation PASS. Свежий dev x86_64 APK проверен по SHA. Release A/B, полный конфигурационный/runtime контракт и остальные платформы открыты.

D02: [удержание namespace](../reports/AUDIT-Q25-NAMESPACE-PIN.md) открытыми fd действует от admission до конца транзакции; 1922 Linux + 29 privileged + 8 worker E2E PASS. Между транзакциями и после crash durable generation остаётся открытым, как и исходное поколение интерфейса.

D02: [Q25-F083 — исходный sysctl интерфейса](../reports/AUDIT-Q25-SYSCTL-TARGET.md): journal v3 удерживает fd и отказывает при потере свидетельства; 3 дефекта baseline воспроизведены, 5 дополнительных worker E2E PASS. Опасное восстановление по имени и потеря original закрыты. Для global journal durable namespace generation после crash остаётся открытым; автоматическое per-interface crash recovery не обещается.

D05: [Q25-F084 — общий срок DNS](../reports/AUDIT-Q25-DNS-BUDGET.md): dns/domain делят 15 секунд от admission, частичный lease сохраняется для отдельного rollback. 3 новые Linux-регрессии PASS. NAT/routes/kill-switch и остальные lock waits остаются открытыми.

D05/D09: [Q05-F008 — async health probes](../reports/AUDIT-Q05-HEALTH-PROBES.md): Status/Transport health не блокируют executor ожиданием `--version`; четыре общих async-слота, deadline включает очередь. 8 новых обычных + 1 privileged HTTP-router тест PASS; 2 контрольных возврата старого поведения дают ожидаемый FAIL. Общие сроки сетевых мутаций и полный HTTP/systemd/fault охват остаются открытыми.

D05/D09: [Q25-F085 — общий срок очистки kill-switch](../reports/AUDIT-Q25-KILL-SWITCH-BUDGET.md): 15 секунд включают operation mutex и обе семьи; частичный результат сохраняет owner для нового verified retry. 5 регрессий PASS, 2 контрольных FAIL старого поведения. Engage/refresh рассмотрены следующими фазами ниже; NAT/routes/gateway и прочие lock waits ещё открыты.

D05/D09: [Q25-F086/F087 — refresh kill-switch](../reports/AUDIT-Q25-KILL-SWITCH-REFRESH.md): общий срок admission/команд учитывает resolver; unknown не разрешает вставку. 6 новых регрессий и 5 повторных cleanup PASS; 3 контрольных FAIL прежнего поведения. DNS/NSS остаётся синхронным; engage закрыт следующей фазой ниже в части командного срока, NAT/routes/gateway и прочие ожидания открыты.

D05/D09: [Q25-F088/F089 — установка и откат kill-switch](../reports/AUDIT-Q25-KILL-SWITCH-SETUP.md): общий срок установки 15 секунд и отдельный общий срок отката 15 секунд; неполный откат нельзя принять флагами разрешения утечки. 8 новых регрессий PASS; 2 контрольных FAIL, затем 8 setup + 6 refresh + 5 cleanup PASS. Полный снимок: 1962 Linux + 30 privileged + 8 worker E2E PASS. Командные сроки engage/refresh/disengage закрыты в описанных границах; синхронный DNS/NSS, NAT/routes/gateway, прочие ожидания и полные D04/D05/D09 остаются открытыми.

D05/D09: [Q14-F034 — общий срок очистки NAT](../reports/AUDIT-Q14-NAT-CLEANUP-BUDGET.md): profile/startup/final cleanup делят по 15 секунд между очередью, IPv4/IPv6, точными правилами и retired DNS UDP/TCP. Поздние ответы не дают успеха, неподтверждённые записи сохраняются. 8 новых Linux-регрессий PASS; 4 контрольных FAIL, затем 8 регрессий + 1 privileged exact-rule PASS. Полный снимок: 1970 Linux + 30 privileged + 8 worker E2E PASS. Открыты NAT setup/rollback, admission DNS lease в Drop/setup, routes/gateway, внутренние sysctl/I/O и полные D04/D05/D09.

D05/D09: [Q14-F035 — сроки и retirement DNS INPUT lease](../reports/AUDIT-Q14-DNS-INPUT-BUDGET.md): setup и cleanup получают по 15 секунд с очередью/UDP/TCP; отдельный откат после setup, неблокирующая отметка завершения и сохранение pending evidence. 4 новые переносимые + 7 Linux + 1 privileged регрессии PASS; 5 контрольных отказов, затем 20 domain + 7 DNS + 8 NAT + 2 native PASS. Полный снимок: 1981 Linux + 31 privileged + 8 worker E2E PASS. NAT setup/rollback, DNS REDIRECT, routes/gateway, внутренние sysctl/I/O и полные D04/D05/D09 остаются открытыми.

D05/D09: [Q14-F036 — установка NAT/forwarding и DNS REDIRECT](../reports/AUDIT-Q14-NAT-SETUP-BUDGET.md): общие сроки setup и точного rollback; 10 новых регрессий, 7 контрольных отказов, 1991 Linux + 31 privileged + 8 E2E PASS. Client routes/gateway, scheduler isolation и полный D05 остаются открытыми.

D02 закрыт: [Q25-F090 — поколение namespace и журнал v4](../reports/AUDIT-Q25-NAMESPACE-GENERATION.md). D04/D05 и остальные критерии сохраняются. Проверки Windows VM, Mac/iOS и роутера исключены из текущего объёма по решению пользователя; они не объявляются PASS.

D09/D10: [17/17 Linux packet matrix PASS](../reports/AUDIT-Q34-LINUX-MATRIX.md). D13: 100 TCP handover сохранили сессию/fd, но RSS превысил критерий; FAIL сохранён, долг открыт.

D05/D09: [Q25-F091 — общий срок gateway/exit-node](../reports/AUDIT-Q25-GATEWAY-BUDGET.md): 6 регрессий, 6 контрольных FAIL, 107 restored gateway и полный Linux 2001 + 32 privileged + 8 E2E PASS. Route-последовательности, внутренние locks/I/O и scheduler isolation остаются открыты.

D13: [release TCP/UDP по 100 handover](../reports/AUDIT-Q34-RELEASE-SOAK.md): 30/30 утверждений PASS, прирост RSS в прежнем лимите 32 MiB; debug FAIL сохранён. Полный ресурсный/fault охват и итоговый снимок ещё открыты.

D05/D09: [Q25-F092 — общий срок route-транзакций](../reports/AUDIT-Q25-ROUTE-BUDGET.md): 8 регрессий, 6 контрольных FAIL, 196 восстановленных route tests и полный Linux 2009 + 32 privileged + 8 E2E PASS. Executor isolation и внутренний I/O остаются открыты.

D06/D09: [Q25-F093 — строгая проверка resolver-конфига](../reports/AUDIT-Q25-RESOLVER-CONFIG.md): 3 новых теста, 2 контрольных FAIL, 18 восстановленных DNS, полный Linux 2012 + 32 privileged + 8 E2E PASS. Отдельно подтверждена cross-netns мутация через общую D-Bus-шину; bus/service identity ещё требует исправления.

D06/D09/D10: [Q25-F094/F095 — контекст resolved/D-Bus и DNS-порт](../reports/AUDIT-Q25-RESOLVER-CONTEXT.md): direct unique-owner вызовы с AUTH GUID, 2023 Linux + 33 privileged + 8 E2E, 17/17 packet matrix (301 assertion), 4 контрольных FAIL и restored 29 DNS + 1 privileged PASS. Реальные resolved/custom port/чужие netns и PID namespace проверены. Предыдущий пункт о незакрытом bus/service identity закрыт в описанных границах; остальные критерии D06 и D10 открыты.

D04/D06/D09: [Q14-F037 — worker network lease](../reports/AUDIT-Q14-WORKER-NETWORK-LEASE.md): исправлен обход через разные control/state пути; baseline удалял 9 правил работающего worker. 2027 Linux + 34 privileged + 8 lifecycle и 22 crash/admission/recovery проверки PASS. D04 IN_PROGRESS: persistent exact firewall/routes и mixed nft остаются открытыми.

D04/D09/D10: [Q14-F038 — persistent server firewall](../reports/AUDIT-Q14-FIREWALL-JOURNAL.md): точные NAT/routing/DNS INPUT/REDIRECT спецификации записываются до изменения, восстанавливаются после SIGKILL и удаления профиля без listing. Backend/namespace/file errors останавливают startup, evidence сохраняется. 2044 Linux + 35 privileged + 8 lifecycle; 27 recovery checks; 17/17 cases, 301 assertions PASS. D04 остаётся IN_PROGRESS: клиентские route/DNS/kill-switch и полная mixed nft/firewalld матрица открыты.

D04/D06/D09: [Q25-F096/F097 — DNS state v2](../reports/AUDIT-Q25-DNS-MARKER-STORAGE.md): доверенный закреплённый каталог, проверка файлов и SO_NETNS_COOKIE; v1 сохраняется без автоматической миграции. 3 файловых дефекта воспроизведены на baseline. 2055 Linux + 37 privileged + 8 lifecycle; 17/17 cases, 323 assertions PASS. Остаются client route/kill-switch recovery, legacy global DNS, live persistent TUN, mixed nft/firewalld и накопление sidecar-файлов; D04 IN_PROGRESS.

D04/D09/D10: [Q25-F098 — kill-switch после crash](../reports/AUDIT-Q25-KILL-SWITCH-REBUILD.md): временные точные DROP guards сохраняют прежний барьер при setup/rollback и повторном SIGKILL. Baseline реального клиента пропускал 14 IPv4 + 13 IPv6 UDP-проб; fixed — 0. 2057 Linux + 39 privileged + 8 lifecycle; 14 runtime checks; 17/17 cases, 339 assertions PASS. D04 IN_PROGRESS: routes, legacy global DNS/persistent TUN и полная mixed firewall матрица открыты.

D04/D09: [Q25-F099 — атрибуты владения маршрутом](../reports/AUDIT-Q25-ROUTE-ATTRIBUTES.md): пригодность чужого маршрута отделена от delete/replace authority; неявные protocol/metric/source и дополнительные атрибуты проверяются. Baseline удалял 10 операторских замен на ядре и static bypass настоящего клиента. 2068 Linux + 40 privileged + 8 lifecycle; 17/17 cases, 384 assertions PASS. На этом этапе persistent client route journal ещё не был реализован; продолжение Q25-F100 ниже. D04 IN_PROGRESS.

D04/D09: [Q25-F100 — постоянный журнал физических маршрутов](../reports/AUDIT-Q25-ROUTE-JOURNAL.md): durable intent/confirmed ownership, boot/cookie/TUN scope, общая блокировка и terminal roaming при I/O ошибке. Baseline оставлял bypass/blackhole после SIGKILL → reconnect → stop (3 FAIL). 2087 Linux + 43 privileged + 8 lifecycle; 17/17 cases, 489 assertions PASS. Client physical route recovery закрыт в описанных пределах; legacy global DNS, live persistent TUN и mixed firewall остаются D04 IN_PROGRESS.
