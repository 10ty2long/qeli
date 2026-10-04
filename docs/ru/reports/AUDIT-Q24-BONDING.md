# Q24: multipath, bonding и общий бюджет

<!-- normative-sync: audit-q24-bonding-v1 -->

**IN_PROGRESS. Общий итог: 23/37 (62,2%).** 4 октября 2026.
[Evidence](../../../release/certification/evidence/q24-bonding-20261004.json).

## Q24-F001: один зависший carrier останавливал здоровые потоки (P2)

В изолированной Linux-лабе сначала прошли 480/480 UDP echoes по 16 внутренним
потокам. Затем один вторичный TCP carrier был заблокирован, одновременно запущен
ограниченный UDP flood. Получены всего 11 echoes; за последние пять секунд — 0.
После удаления блокировки трафик восстановился без замены процесса или TUN.

Клиентская очередь на 4096 элементов сохраняла TunPacket из общего малого пула.
Заблокированный writer мог удержать весь пул, после чего TUN reader переставал
читать пакеты для остальных carrier. Серверная очередь также могла вместить
весь общий 4 МиБ wire-пул, хотя должна была делить его с другими соединениями.

В общем ядре добавлен один 4 МиБ пул snapshots для bonded uplink. В очередь
попадает PooledBuffer; исходный TUN-пакет освобождается сразу, включая FIFO
reservations Wintun. Каждая очередь получает долю общего бюджета с запасом
для остальных потоков и временного resume candidate. Нет fallback allocation;
при перегрузке пакет отбрасывается. Сервер использует тот же расчёт доли.
Одиночный TCP сохраняет прежний путь. Новый INI-ключ, wire-format или ABI не вводится.

## Разбор слоя

Проверены primary AUTH/server push, bearer JOIN и authenticated JOIN proof,
свежий transcript/epoch, stream caps, атомарное admission/revocation, fixed/adaptive
actors, logical-slot scheduler, IPv4/IPv6 fragment affinity, независимые upload/download
rate buckets, cover/stealth, общие счётчики квоты и освобождение задач/сокетов.
Bearer JOIN остаётся необходимым для совместимости; подтверждённого мёртвого
production-кода в этом проходе не найдено.

Мануал уточнён: диапазон 1–16, fixed является целью с retry; adaptive растёт
раз в 3 секунды под нагрузкой и не делает downscale. Стабильные слоты требуют
согласованного authenticated resume; старый modulo scheduler меняет назначение
при изменении состава. Один внутренний поток не распределяется между carriers.

## Проверки и оставшаяся работа

До изменения production-кода прошли fixed/adaptive handover и resume, grace
expiry, shared 8 Mbps cap/one-carrier reset/revocation в fake-tls и plain.
Исправлена fixture, ошибочно ожидавшая строку 1/1 вместо реальной bonded width.
В REALITY-TLS постоянная 15% loss задержала финальный обмен iperf до timeout;
новая fixture ограничивает длительность loss и сохраняет отдельные phase logs.
Этот результат не объявлен подтверждённой production-регрессией.

Исправление Q24-F001 закоммичено в audit worktree. Финальные Linux units/Clippy/release,
повтор baseline/fixed в обоих направлениях, пакет runtime-сценариев и native A/B/ABI
ещё проходят квалификацию. Старые 2405 units и native bytes нельзя считать новой
проверкой изменённого ядра. Q24 остаётся открытым до завершения этих gates.

Лаба .11 принимает TCP/22, но не выдаёт SSH banner; новых auth-попыток нет.
Доступный стенд .10 используется последовательно, одним compiler job. Работающий
service/binary .10 сохранён. Mac/iOS/router/Windows VM исключены пользователем;
INI остаётся единственным форматом профилей, служебный JSON API сохраняется.
