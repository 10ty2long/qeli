# Документация qeli — карта

Документация организована **по типам документов**. Русское и английское деревья имеют
одинаковую структуру, а каждый актуальный документ доступен из этой карты.

> Новичку: начните с **[Установки с нуля](manuals/GETTING-STARTED.md)**, затем откройте
> **[Конфигурацию](manuals/CONFIG.md)**. Если что-то не работает —
> **[Диагностика](manuals/TROUBLESHOOTING.md)**.

**English version → [../eng/index.md](../eng/index.md)**

## Обзор

| Документ | О чём |
|---|---|
| [README.md](README.md) | Обзор проекта: назначение, wire-режимы, криптостек и состав репозитория |

## Руководства (`manuals/`)

Практические инструкции по установке, настройке и эксплуатации.

| Документ | О чём |
|---|---|
| [GETTING-STARTED.md](manuals/GETTING-STARTED.md) | Установка и первый запуск, пошагово с нуля |
| [CONFIG.md](manuals/CONFIG.md) | Полный справочник flat-INI конфигурации сервера и клиентов |
| [OPERATIONS.md](manuals/OPERATIONS.md) | Совместимость, обновление, откат, резервное копирование и firewall |
| [PANEL.md](manuals/PANEL.md) | Установка и использование веб-панели |
| [IPV6.md](manuals/IPV6.md) | IPv4/IPv6/dual-stack, `off/manual/route/nat66`, NDP proxy и диагностика |
| [OBFUSCATION.md](manuals/OBFUSCATION.md) | Recordizer, совместимость слоёв маскировки и профили тюнинга |
| [TROUBLESHOOTING.md](manuals/TROUBLESHOOTING.md) | Диагностика подключения и справочник ошибок |
| [KEENETIC-DEPLOY.md](manuals/KEENETIC-DEPLOY.md) | Пошаговый деплой клиента на Keenetic |

## Справочники и архитектура (`reference/`)

Технические контракты и описание текущей реализации.

| Документ | О чём |
|---|---|
| [CLIENT-CONFIG-MATRIX.md](reference/CLIENT-CONFIG-MATRIX.md) | Актуальный контракт клиентских ключей по платформам и история миграции |
| [THREAT-MODEL.md](reference/THREAT-MODEL.md) | Модель угроз, границы доверия и уровень проверенности |
| [TRANSPORT-CORE.md](reference/TRANSPORT-CORE.md) | Общее транспортное Rust-ядро, source/ABI-контракт и release gates |
| [KEENETIC-PORT.md](reference/KEENETIC-PORT.md) | Архитектура порта Keenetic и обоснование dual-arch сборки |

## Планы (`plans/`)

Актуальные направления разработки и планы реализации. Это не пользовательские инструкции.

| Документ | О чём |
|---|---|
| [ROADMAP.md](plans/ROADMAP.md) | Продуктовый и технический план развития |
| [ROAMING.md](plans/ROAMING.md) | Нормативный план реализации клиентского роуминга |
| [IPV6-IMPLEMENTATION-PLAN.md](plans/IPV6-IMPLEMENTATION-PLAN.md) | Архитектура IPv6, этапы и release gates |
| [CLIENT-CONFIG-CORE.md](plans/CLIENT-CONFIG-CORE.md) | Единый INI/URI API в Rust-ядре, удаление клиентских дублей и оставшиеся проверки |
| [FULL-SYSTEM-AUDIT.md](plans/FULL-SYSTEM-AUDIT.md) | История аудитов, 37 разделов полного тестового плана, стенды и прогресс |

## Отчёты (`reports/`)

Актуальные анализы и результаты измерений. Датированные зафиксированные отчёты находятся в архиве.

| Документ | О чём |
|---|---|
| [AUDIT-Q02-CLIENT-PARSERS.md](reports/AUDIT-Q02-CLIENT-PARSERS.md) | INI/URI и редакторы: 19 находок, общий корпус, тесты Rust/C#/Kotlin и ограничения Swift |
| [AUDIT-Q19-Q22-NETWORK-PLAN.md](reports/AUDIT-Q19-Q22-NETWORK-PLAN.md) | Общий DNS-план, legacy/v2, CIDR-исключения: исправления, регрессии и границы проверки |
| [AUDIT-Q19-DNS-PROXY.md](reports/AUDIT-Q19-DNS-PROXY.md) | Серверный DNS: CNAME/NODATA, сжатые имена, TCP failover и локальные сетевые тесты |
| [AUDIT-Q19-DNS-CACHE.md](reports/AUDIT-Q19-DNS-CACHE.md) | Лимит памяти DNS-кеша, пересылка TSIG/SIG(0) и проверка запросов |
| [AUDIT-Q19-DNS-EDNS.md](reports/AUDIT-Q19-DNS-EDNS.md) | EDNS, проверка записей, общий TTL ответа и тесты IPv6 upstream |
| [AUDIT-Q14-HOOKS.md](reports/AUDIT-Q14-HOOKS.md) | Ограничение stdout/stderr, timeout/cancellation и потомки lifecycle hooks |
| [AUDIT-Q14-Q15-WORKER-USAGE.md](reports/AUDIT-Q14-Q15-WORKER-USAGE.md) | Владение службами worker, финальное сохранение и учёт коротких сессий |
| [AUDIT-Q14-Q32-NOTIFICATIONS.md](reports/AUDIT-Q14-Q32-NOTIFICATIONS.md) | Ограничение отправок уведомлений, тесты панели и завершение задач |
| [AUDIT-Q14-Q33-CONFIG-TRUST.md](reports/AUDIT-Q14-Q33-CONFIG-TRUST.md) | Доверие прочитанному конфигу, гонки файлов и очистка поколения |
| [AUDIT-Q25-CREDENTIAL-COMMANDS.md](reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md) | Лимиты password_command, ошибки без секретов, ранний stop и изоляция features |
| [AUDIT-Q25-PASSWORD-FILES.md](reports/AUDIT-Q25-PASSWORD-FILES.md) | Лимиты файлов пароля, общий буфер секрета и финальный статус клиента |
| [AUDIT-Q25-NETWORK-CLEANUP.md](reports/AUDIT-Q25-NETWORK-CLEANUP.md) | Сохранение kill-switch при ошибке очистки forwarding |
| [AUDIT-Q25-CORE-LIFECYCLE.md](reports/AUDIT-Q25-CORE-LIFECYCLE.md) | Ошибки запуска/остановки ядра, терминальные hooks и сохранение kill-switch |
| [AUDIT-Q25-DNS-RECOVERY.md](reports/AUDIT-Q25-DNS-RECOVERY.md) | Восстановление старого DNS: проверка операций и сохранение снимка |
| [AUDIT-Q25-TUN-CLEANUP.md](reports/AUDIT-Q25-TUN-CLEANUP.md) | Ошибки очистки TUN/DNS/маршрутов, владение планом и сохранение terminal kick |
| [AUDIT-Q14-OWNED-SHUTDOWN.md](reports/AUDIT-Q14-OWNED-SHUTDOWN.md) | Итоговая проверка DNS/IPv6 sysctl leases и передача ошибок worker/supervisor; Q14-F027 исправлена частично |
| [AUDIT-Q14-PROFILE-SHUTDOWN.md](reports/AUDIT-Q14-PROFILE-SHUTDOWN.md) | Ошибки задач профиля, TUN queue timeout/panic и удаления TUN в результате остановки; Q14-F027 частично |
| [AUDIT-Q14-SYSCTL-RECOVERY.md](reports/AUDIT-Q14-SYSCTL-RECOVERY.md) | Q14-F029/F030: ошибки восстановления sysctl и сохранение прежнего владельца при повторном acquire |
| [AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md](reports/AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md) | Q14-F031: учёт частичного IPv6 acquire и повторный rollback до подтверждённого освобождения |
| [AUDIT-Q05-PREFLIGHT.md](reports/AUDIT-Q05-PREFLIGHT.md) | Q05-F001: ограничения команд preflight и проверка частичного IPv4/IPv6 snapshot |
| [AUDIT-Q14-NAT-COMMANDS.md](reports/AUDIT-Q14-NAT-COMMANDS.md) | Q14-F032: серверный NAT использует общий runner со сроком и лимитом вывода; ownership после timeout |
| [AUDIT-Q14-DNS-OWNERSHIP.md](reports/AUDIT-Q14-DNS-OWNERSHIP.md) | Сохранение DNS rule specs при отказе cleanup/rollback, retry и идентичность поколения |
| [AUDIT-Q14-Q25-FIREWALL-CHECKS.md](reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md) | Общий разбор firewall-проверок сервера/клиента, точечная очистка DNS и граница 1024 правил |
| [AUDIT-Q14-NAT-CLEANUP.md](reports/AUDIT-Q14-NAT-CLEANUP.md) | Конечная очистка NAT, проверка результата, диагностика и открытые ошибки teardown |
| [AUDIT-Q25-SYSTEM-COMMANDS.md](reports/AUDIT-Q25-SYSTEM-COMMANDS.md) | Сроки и лимиты вывода команд TUN/resolvectl, завершение процессов и DNS marker |
| [AUDIT-Q25-TCP-TASKS.md](reports/AUDIT-Q25-TCP-TASKS.md) | Владение TCP-задачами, закрытие spawn и ожидание Linux path workers |
| [AUDIT-Q25-UDP-TASKS.md](reports/AUDIT-Q25-UDP-TASKS.md) | Владение UDP-задачами, передача пути и ожидание перед rollback |
| [AUDIT-Q25-TUN-WORKERS.md](reports/AUDIT-Q25-TUN-WORKERS.md) | Общее владение потоками TUN/Wintun при отмене shutdown |
| [AUDIT-Q25-H2-TASKS.md](reports/AUDIT-Q25-H2-TASKS.md) | Владение H2 driver/bridge от connect до завершения TCP-поколения |
| [AUDIT-Q14-H2-TASKS.md](reports/AUDIT-Q14-H2-TASKS.md) | H2-задачи серверного профиля, join перед teardown и удержание pre-auth при отказе |
| [AUDIT-Q14-CONTROL.md](reports/AUDIT-Q14-CONTROL.md) | Владение control socket, границы API, shutdown handlers и парность hooks |
| [AUDIT-Q14-SUPERVISOR.md](reports/AUDIT-Q14-SUPERVISOR.md) | Supervisor: stop/retry, владение Child/PID, команды и deadline завершения |
| [AUDIT-Q14-Q19-LIFECYCLE.md](reports/AUDIT-Q14-Q19-LIFECYCLE.md) | Завершение профиля, ранние ошибки запуска, DNS-слушатели и освобождение сокетов |
| [AUDIT-Q01-SERVER-INI.md](reports/AUDIT-Q01-SERVER-INI.md) | Первый проход серверного INI: 7 находок, исправления, тесты и ограничения |
| [AUDIT.md](reports/AUDIT.md) | Актуальная модель безопасности и статус аудита |
| [DPI-AUDIT.md](reports/DPI-AUDIT.md) | Анализ обнаружимости DPI и меры устранения |
| [BENCHMARK.md](reports/BENCHMARK.md) | Методика нагрузочного тестирования и замеры по режимам |
| [Qeli 0.8.0: 34 VPN-режима](reports/benchmarks/vpn_protocol_benchmark_repeat_2026-09-01.md) | Полный датированный сравнительный прогон, CPU/RSS и ограничения интерпретации |
| [COMPARISON.md](reports/COMPARISON.md) | Сравнение с WireGuard, OpenVPN и V2Ray |

## Архив (`archive/`)

Зафиксированные исторические документы сохранены для прослеживаемости и не обновляются как
актуальные инструкции. Начните с **[карты архива](archive/README.md)**.

### Завершённые планы и design logs

| Документ | Зафиксированный контекст |
|---|---|
| [REFACTOR-PLAN.md](archive/plans/REFACTOR-PLAN.md) | Завершённый план и журнал устранения production-дублей |
| [DESIGN-remaining.md](archive/plans/DESIGN-remaining.md) | Снимок разработки REALITY от июня 2026 |
| [RELEASE-FIXES.md](archive/plans/RELEASE-FIXES.md) | Исторический план стабилизации ранних pre-1.0 релизов |

### Исторические аудиты

| Документ | Дата |
|---|---|
| [AUDIT-2026-06-10.md](archive/audits/AUDIT-2026-06-10.md) | 2026-06-10 — аудит безопасности и надёжности |
| [AUDIT-2026-06-11.md](archive/audits/AUDIT-2026-06-11.md) | 2026-06-11 — разбор внешнего аудита и исправления |
| [AUDIT-2026-06-11-external2.md](archive/audits/AUDIT-2026-06-11-external2.md) | 2026-06-11 — разбор второго внешнего аудита |
| [AUDIT-2026-06-12.md](archive/audits/AUDIT-2026-06-12.md) | 2026-06-12 — аудит и исправления для 0.7.1 |

## Документация клиентов (рядом с кодом)

| Клиент | Документ |
|---|---|
| Windows | [qeli-win/README.md](../../qeli-win/README.md) |
| macOS | [qeli-mac/README.md](../../qeli-mac/README.md) |
| iOS ⚠️ | [qeli-ios/README.md](../../qeli-ios/README.md) · MDM: [qeli-ios/MDM/README.md](../../qeli-ios/MDM/README.md) — реализован полностью, но **на устройстве не проверялся** и не выпускается |
| Роутеры (OpenWrt) | [qeli-openwrt/README.md](../../qeli-openwrt/README.md) · Keenetic: [KEENETIC-DEPLOY.md](manuals/KEENETIC-DEPLOY.md) |
| Android | [qeli-android/README.md](../../qeli-android/README.md) |
| Linux CLI | [GETTING-STARTED §8.2](manuals/GETTING-STARTED.md) |

## Вне этого каталога

- **[../../CHANGELOG.md](../../CHANGELOG.md)** — все изменения по версиям.
- **[../../release/RELEASE_NOTES_0.8.1.md](../../release/RELEASE_NOTES_0.8.1.md)** — двуязычное
  описание закрепления архитектуры 0.8, практический результат для пользователей, порядок
  обновления, артефакты и проверка релиза.
- **[../../release/RELEASE_NOTES_0.8.0.md](../../release/RELEASE_NOTES_0.8.0.md)** — dev-миграция
  Reality/H2, значения по умолчанию, порядок обновления и проверка.
- **[../../release/dpi_audit_dev_0.8.0_h2_2026-08-26/REPORT.md](../../release/dpi_audit_dev_0.8.0_h2_2026-08-26/REPORT.md)** — датированный H2 PCAP/DPI-результат и ограничения.
- **[../../release/RELEASE_NOTES_0.7.16.md](../../release/RELEASE_NOTES_0.7.16.md)** — двуязычный
  выпускной документ `0.7.16` и влияние обновления.
- **[../../SECURITY.md](../../SECURITY.md)** — политика безопасности и приём отчётов.
- **[../../CONTRIBUTING.md](../../CONTRIBUTING.md)** — как участвовать в разработке.
- **[../../release/docker/README.md](../../release/docker/README.md)** — запуск сервера в Docker.
