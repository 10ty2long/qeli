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
| D02 | 14/25 | IN_PROGRESS | Внутренние границы sysctl | Проверять namespace после ожидания lock и на границах I/O/prune, защищать чтение журнала, различать старый и заменённый интерфейс. Регрессии смены identity и native restore. |
| D03 | 22/25 | DONE | Самостоятельный kill-switch | Закреплённый namespace, сохранённый владелец точных семейств, fail-closed reconnect и безопасная смена адреса. 11 portable + 2 native регрессии, реальные счётчики IPv4/IPv6 и 2 отказа baseline. [Отчёт](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md). |
| D04 | 14/19/22/25 | TODO | Восстановление после crash | Определить и реализовать безопасное восстановление exact firewall/DNS/routes, включая mixed nft, SIGKILL и удалённый профиль. Нельзя выдавать process-local registry за persistent journal. |
| D05 | 05/14/25 | IN_PROGRESS | Срок всей операции и блокировки | Убрать синхронный preflight из async handler/долгого config lock; ограничить последовательности команд и ожидания; проверить отмену и доступность соседних запросов. |
| D06 | 21/22/23/25 | IN_PROGRESS | Контекст внешних сетевых ресурсов | Проверить WAN identity, resolved/bus context, sysfs/procfs и attach/name-контракт; process-global DNS/carrier state, dynamic IPv6. Зафиксировать поддерживаемые комбинации. |
| D07 | 01/05/09/11 | TODO | Серверный конфиг в runtime | Таблица field → parse/validate/runtime/serialize; malformed/oversized input; check-config/startup/SIGHUP/HTTP save/Quick Start с сохранением действующего состояния при отказе. |
| D08 | 02/24/27 | TODO | Общие клиентские конфиги | Проверить весь контракт 81+3 полей, INI/import/URI/QR/form/store/reconnect через реальные адаптеры; fuzz/budget и конкурентное редактирование. |
| D09 | 14/15/25/32/33 | IN_PROGRESS | Linux lifecycle и системные отказы | Выполнить Linux tests для flock/permissions/control/hooks/process groups, worker/services/TUN/route/DNS; сохранить stdout, exit, SHA и before/after. Привилегированные ignored tests запускать явно. |
| D10 | 17/18/19/21/22/23 | TODO | Сетевая интеграционная матрица | Проверить off/manual/route/nat66 × NDP, DNS UDP/TCP, multiprofile, iptables/nft/firewalld, setup rollback/stop/restart и сохранение чужих ресурсов. |
| D11 | 00/24/27/34 | TODO | Актуальные native cores и provenance | Из чистого commit пересобрать изменённые ядра по закреплённым рецептам, сравнить A/B, обновить копии и настоящие provenance; проверить ABI/exports и пакеты. |
| D12 | 24/25/27/34 | BLOCKED | Платформенное подтверждение | Android emulator в лабе доступен, запуск ещё нужен. Windows runtime, Mac/Xcode и iOS, а также router runtime не подтверждены; наличие нужных стендов уточняется. Compile-only не заменяет эти проверки. |
| D13 | 14/19/22/25 | TODO | Удержание ресурсов под нагрузкой | Измерить fd/tasks/threads/TUN/routes/firewall/journals/RSS до и после churn/reconnect/stop, включая отказы и несколько профилей; конечный deadline и критерии отсутствия роста. |
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

D05: [async preflight и время жизни транзакции панели](../reports/AUDIT-Q05-PANEL-TRANSACTIONS.md). Открыты общий бюджет backup/restore и остальных сетевых последовательностей.
