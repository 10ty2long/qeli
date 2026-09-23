# Qeli — диагностика подключения и справочник по ошибкам

> **Статус документации:** текущая ветка разработки **0.8.2**; планируемый full-IPv6 релиз **0.8.2**;
> последний опубликованный релиз **0.8.1**. Публичного релиза 0.7.17 не будет.
> Фактическую версию установленного бинарника показывает `qeli --version`.

Подробный практический гайд: как включить debug, как читать лог по стадиям
подключения, что означает каждая ошибка сервера и клиентов (Windows / macOS /
Android) и как её чинить. Все строки — точные, как они появляются в логе.

> Для отдельного чеклиста inner/outer IPv6, `off/manual/route/nat66`, NDP proxy, PMTU, DNS и утечек см.
> [руководство по IPv6](IPV6.md).

> Строки ошибок в коде **на английском** (так они и печатаются). Ниже к каждой —
> расшифровка и метод исправления. Если строки в вашем логе нет здесь — ищите по
> ключевому слову, разделы сгруппированы по подсистемам.

**Содержание**
1. [Включение debug-логов](#1-включение-debug-логов)
2. [Архитектура и жизненный цикл подключения](#2-архитектура-и-жизненный-цикл-подключения)
3. [Пошаговая диагностика](#3-пошаговая-диагностика)
4. [Каталог ошибок — сервер](#4-каталог-ошибок--сервер)
5. [Каталог ошибок — клиенты (Windows / macOS / Android)](#5-каталог-ошибок--клиенты)
6. [Типовые сценарии («симптом → причина → фикс»)](#6-типовые-сценарии)
7. [Справочник: статусы, цвета индикаторов, суффиксы лога](#7-справочник)
8. [Чеклисты команд](#8-чеклисты-команд)

---

## 1. Включение debug-логов

### 1.1 Сервер

Уровень по умолчанию — **`info`**. Бо́льшая часть причин отказа в подключении
(проблемы хендшейка/крипто/MTU **до** аутентификации) логируется на уровне
**`debug`** — при `info` их не видно. Поэтому первый шаг любой диагностики
«клиент не подключается, а на сервере тишина после `New TCP connection`» —
включить debug.

Два способа (**`RUST_LOG` имеет приоритет над `[logging] level` в конфиге** — это
задано в `main.rs::init_logging`):

**A. Через systemd drop-in (ничего в конфиге не трогаем):**
```bash
mkdir -p /etc/systemd/system/qeli.service.d
printf '[Service]\nEnvironment=RUST_LOG=debug\n' > /etc/systemd/system/qeli.service.d/zz-debug.conf
systemctl daemon-reload && systemctl restart qeli
journalctl -u qeli -f
# откат: rm /etc/systemd/system/qeli.service.d/zz-debug.conf && systemctl daemon-reload && systemctl restart qeli
```

**B. Через конфиг** — в секции `[logging]` поставить `level = debug`, затем
`systemctl restart qeli`. Ключи секции: `level` (`error`/`warn`/`info`/`debug`/`trace`),
`file` (путь к лог-файлу; по умолчанию stderr → journald), `time_format`, `format`.

**Метка времени — `time_format`.** Строка лога всегда `<метка> LEVEL target: сообщение`,
ключ задаёт форму метки: `datetime` (дефолт, локальное время) / `rfc3339` (UTC) / `time`
(без даты) / `epoch` / `none`. Два практических случая:

- **сводите логи клиента и сервера** (или нескольких серверов) — ставьте `rfc3339` с обеих
  сторон: UTC убирает расхождение часовых поясов, и строки корректно сортируются;
- **лог идёт в journald/syslog** — `none`: systemd и procd штампуют строку сами, иначе в
  `journalctl` получаются две метки времени подряд.

Полная таблица вариантов — в [CONFIG.md](CONFIG.md#time_format--метка-времени). То же
самое настраивается в приложениях: «Настройки → Время в логе» (Windows, macOS, Android)
и `log_time_format` в UCI/LuCI на OpenWrt.

> ⚠️ **`format = json` — заглушка.** Не путать с `time_format` выше: `format` отвечает за
> форму самой строки, парсится и показывается в панели, но `init_logging` его **не читает** —
> лог всегда плоский. Не рассчитывайте на JSON-логи.

Точечная фильтрация (меньше шума): `RUST_LOG=qeli::server::handler=debug,qeli::server::udp_handler=debug,info`.

**Разовый запуск в форграунде** (быстро посмотреть, без правки юнита):
```bash
systemctl stop qeli
RUST_LOG=debug /usr/bin/qeli server --config /etc/qeli/server.conf   # .deb ставит в /usr/bin
```

### 1.2 Клиенты (Windows / macOS)

Отдельного «debug-режима» нет — **клиент логирует всё сразу** во вкладку **Log** /
**Журнал** в окне приложения. По умолчанию строка начинается с локальной даты и времени:
`2026-07-18 18:10:03.259  …`. **С 0.7.12** форма метки настраивается — «Настройки →
Время в логе», варианты те же, что у `[logging] time_format` на сервере (дата и время /
RFC 3339 в UTC / только время / Unix / без метки); чтобы сверять лог приложения с
серверным, ставьте `RFC 3339` с обеих сторон. Кнопки **Copy log** / **Clear log** в шапке журнала.
«Тяжесть» строки задаётся префиксом: `ERR:`, `WARN:`, `NOTE:`, `[SECURITY]`, а
вложенные причины — строками `  <- …`.

### 1.3 Клиент Android

VPN-сервис ведёт приватный постоянный журнал, даже когда Activity закрыта или интерфейс приложения
пересоздан. Хранятся последние 1000 событий, но не более 512 КиБ; при открытии Qeli они
восстанавливаются во **вкладке «Журнал»** (индекс 3). Отключение и новое подключение историю не
удаляют — это делает только кнопка **«Очистить»**. Поэтому **Copy log** захватывает и события,
которые произошли при выключенном экране. Файл находится в приватном no-backup каталоге Android
и не содержит пароль, приватные ключи или полный профиль.

Уровень Info фиксирует границы сессии, причину остановки, потерю сети/реконнект, redelivery или
отзыв VPN со стороны Android и предупреждение об оптимизации батареи. Debug/Trace дополнительно
записывает время выключения/включения экрана и подробные события адаптера. Формат времени по-прежнему
настраивается в «Настройки → Время в логе»; восстановленные строки сохраняют исходное время события.
По умолчанию показывается только время, чтобы полная дата не съедала ширину экрана.

Активное пользовательское подключение возвращает `START_REDELIVER_INTENT`, поэтому после убийства
процесса Android может повторно доставить точную команду Connect. Явное отключение сначала
синхронно сбрасывает durable-флаг и никогда не перезапускается. OEM force-stop и агрессивные
ограничения фона всё равно могут запретить любой рестарт, поэтому при диагностике исключите Qeli
из оптимизации батареи. Тот же процессный вывод доступен через `adb`:
```bash
adb logcat -s VpnSvc VpnMain
```
`VpnSvc` — VPN-сервис, `VpnMain` — Activity. Строки необработанного Android runtime crash всё ещё
могут остаться только в logcat; собственные диагностические события Qeli сохраняются в журнале.

### 1.4 Трассировка пакетов (`QELI_TRACE`, Rust-сервер и Rust-клиент)

Когда логов мало и нужен таймлайн — «ушёл ли пакет и когда его увидела вторая сторона».
Взводится переменной окружения, выключена иначе:

```bash
# клиент
QELI_TRACE=/tmp/qeli-client.csv qeli client -c /etc/qeli/client.conf

# сервер (systemd): drop-in, затем рестарт
systemctl edit qeli
#   [Service]
#   Environment=QELI_TRACE=/tmp/qeli-server.csv
```

Выгрузка — по сигналу, в любой момент (процесс продолжает работать):

```bash
kill -USR1 $(pgrep -f 'qeli client')      # клиент
kill -USR1 $(pgrep -f 'qeli _worker')     # сервер: именно worker, не supervisor
```

В логе появится `packet trace: wrote N events`, в файле — CSV:

```
# qeli packet trace — shapes only, no payloads, no addresses
# overwritten=0 contended=0
t_us,dir,site,size,seq
479384,tx,client.tcp,40,0
```

- `t_us` — микросекунды от старта процесса, `dir` — `tx` (из TUN в туннель) / `rx`
  (из туннеля в TUN), `site` — точка съёма, `size` — байты, `seq` — индекс потока.
- Пишутся **только формы пакетов**: ни payload, ни адресов — трассу можно приложить к
  issue, не раскрывая трафик.
- Буфер кольцевой на 65 536 событий. Строка `overwritten=` в шапке говорит, сколько
  событий затёрлось (трасса длиннее буфера), `contended=` — сколько потеряно из-за
  конкуренции: трасса **не бывает молча неполной**.
- Обе стороны пишут свои файлы. Общего идентификатора пакета нет, поэтому сопоставлять
  клиент и сервер нужно по времени и размеру.

Накладные расходы при выключенной трассировке — одна атомарная загрузка на пакет, так
что переменную можно держать невзведённой в проде без опасений.

---

## 2. Архитектура и жизненный цикл подключения

### 2.1 Сервер: supervisor + worker

Процесс `qeli server` — это **supervisor**: держит веб-панель и порождает
дочерний **data-plane worker** (`qeli _worker`). Отсюда две важные вещи:

- **«Apply & Restart» в панели делает ПОЛНЫЙ `systemctl restart`** — применяется всё,
  включая сокет панели (`web.bind`/`port`/`tls`/`base_path`). Рестарт только worker'а
  (`POST /api/server/restart`) остался автоматическим фолбэком там, где systemd
  недоступен (контейнер). Ссылки в панели (share) читают конфиг **свежим с диска**
  (фикс #69), поэтому смена SNI видна в ссылке без перезапуска.
- В логе старт видно так:
  ```
  Starting server (supervisor) with config: /etc/qeli/server.conf
  Web UI (HTTPS) listening on https://0.0.0.0:8080
  supervisor: data-plane worker started (pid NNNN)
  Starting data-plane worker with config: /etc/qeli/server.conf
  Starting profile 'fake-tls' (tcp://0.0.0.0:443)
  Profile 'fake-tls': server identity public key (pin on client): 320a4700…
  Profile 'fake-tls' listening on 0.0.0.0:443 (TCP)
  ```
  Если worker падает на старте (валидация конфига) — supervisor логирует
  `supervisor: worker stopped unexpectedly — respawning in Ns` и рестартит с backoff.

Повторные ошибки создания процесса и неожиданные выходы используют задержки
1, 2, 4, 8, 16, затем максимум 30 секунд. После поколения, прожившего не меньше
30 секунд, задержка сбрасывается. SIGINT/SIGTERM обрабатываются и во время retry.
В логе отдельно выводится статус завершившегося worker; его PID в метриках
сбрасывается, пока нового процесса нет.

При **внутреннем** рестарте worker (`POST /api/server/restart`) новая конфигурация
читается после завершения старого процесса без crash-backoff. Restart во время
ожидания retry будит его; накопленные старые команды объединяются. ReloadUsers
действующего worker сохраняет соединения, а при отсутствии/завершении worker
пользователей перечитает следующее поколение.

На штатное завершение worker даётся 60 секунд, затем supervisor запрашивает SIGKILL
и дожидается выхода. Повторный Restart не продлевает этот срок. Сообщение
`worker did not stop within 60s — killing and reaping it` означает принудительный
выход: post_down мог не выполниться, полный откат firewall этим не гарантируется.
Следующий worker очищает старые NAT-правила при старте. 60 секунд ограничивают
ожидание graceful shutdown, а не зависание внутри ядра после SIGKILL.

`worker service 'usage sweep' failed: ...` или `stopped unexpectedly` означает отказ
обязательной службы учёта и квот. То же относится к `UDP loss report`: worker начинает
очистку вместо незаметного продолжения работы без этой службы. Неактивный packet trace
необязателен и может завершаться штатно. При graceful stop текущий периодический цикл
заканчивается до остановки профилей, затем последние байты собираются и сохраняются
при удержании исключительного права worker. `usage: shutdown flush failed` означает
ошибку финальной записи; проверьте место на диске, права и предшествующее сообщение.
Worker сообщает об ошибке; сохранность ещё не записанной статистики не гарантируется.

Предупреждения уведомлений: `notify: queue full` означает превышение лимита 128
отправок на процесс; новые события отбрасываются без бесконечного retry.
`notify: shutdown deadline` — истекли десять секунд drain, оставшиеся отправки отменены.
Проверьте транспортные/HTTP-предупреждения нужного канала: адресат может медленно отвечать
или отклонять запросы. **Send test** делит общий лимит и явно сообщает ошибку очереди,
отмены или timeout. Подробные ограничения описаны в [мануале панели](PANEL.md).

### 2.2 Стадии одного подключения (по логу)

Локализуйте отвал по последней успешной строке:

| # | Стадия | Клиентская строка | Серверная строка |
|---|---|---|---|
| 1 | TCP/UDP коннект | `Connecting TCP/UDP <ip>:<port> as user '<u>'…` → `TCP connected` / `Bound carrier socket…` | `New TCP connection from …` / `UDP handshake started for …` |
| 2 | Отправка ClientHello | `ClientHello sent (NNNN B, hybrid X25519+ML-KEM)` | `Received ClientHello: N bytes` *(debug)* |
| 3 | Проверка личности сервера | `Server identity verified [OK]` | `Sent server auth proof…` *(debug)* |
| 4 | Аутентификация | *(парс `OK:` из ответа)* | `AUTH attempt … user=…` → `AUTH OK …` |
| 5 | Применены пуш-параметры | `Applied server-pushed obfuscation params` | — |
| 6 | Выдан IP | `Auth OK, IP 10.x.x.x` | `Client … connected …, IP: 10.x.x.x` |
| 7 | Поднятие TUN | `Wintun adapter …` / `utun …` → `TUN MTU …` → маршруты → DNS | — |
| 8 | Туннель активен (🟢) | `TUN ready, entering tunnel loop` → **статус Connected** | — |

> **🟢 «Connected» = TUN поднят, а НЕ «Auth OK».** Между `Auth OK, IP …` и
> `Connected` статус остаётся **жёлтым** (Connecting), пока идёт `SetupTun` (на
> Windows открытие Wintun — до ~10 с). Это намеренно (issue #69): раньше зелёный
> зажигался на Auth OK, и падение установки TUN сбрасывало backoff → плотный
> reconnect-шторм.

---

## 3. Пошаговая диагностика

1. **Сервер жив и слушает?**
   ```bash
   systemctl is-active qeli
   ss -ltnp | grep -E ':443|:8443|:8444'      # TCP-профили
   ss -lunp | grep -E ':8448|:8449|:8450'     # UDP-профили
   journalctl -u qeli --since '5 min ago' -p warning --no-pager
   ```
2. **Порт открыт в облачном фаерволе / Security Group?** (частая причина «TCP
   connected» вообще не появляется).
3. **Клиентский лог: до какой стадии дошло?** (таблица §2.2). Последняя успешная
   строка указывает подсистему.
4. **Дошло до сервера?** На сервере ищите `New TCP connection` / `UDP handshake
   started` с IP клиента. Если строки нет — трафик не долетает (фаервол/маршрут/не
   тот IP-порт).
5. **Дошло, но «тишина» после accept?** Включите **debug** (§1.1), переподключитесь,
   и ищите **ровно одну** решающую строку:
   - `handshake timeout for <addr>` → клиент не дослал ClientHello (или ответ не
     дошёл) = **сетевая чёрная дыра по MTU** (см. §6.1);
   - `Client <addr> disconnected on profile '…': <причина>` → см. `<причина>` в §4.2;
   - `AUTH FAIL/DENIED/BLOCKED …` (видно уже на `info`) → креды/бан/права (см. §4.3).
6. **Сверьте ключ и режим.** Публичный ключ сервера виден в логе старта
   (`server identity public key (pin on client): …`) и по `qeli show-identity`.
   Клиентский `key=`/`reality_sid=`/`mode=` должны совпадать (см. §6.4).

---

## 4. Каталог ошибок — сервер

### 4.1 Валидация конфига — worker не стартует (`bail!`, фатально)

Эти ошибки **прерывают старт worker'а**; supervisor логирует падение и рестартит
по кругу с backoff. Все на уровне ERROR.

| Сообщение | Причина | Фикс |
|---|---|---|
| `no profiles defined in server config` | нет ни одной `[profile:*]` | добавить профиль |
| `all profiles are disabled (enabled = false) — enable at least one` | все `enabled = false` | включить профиль |
| `duplicate profile name: '<n>'` | два профиля с одним именем | переименовать |
| `profile '<n>': unknown bind.transport '<t>' — expected 'tcp' or 'udp'` | опечатка в транспорте | `bind.transport = tcp` или `udp` |
| `profile '<n>': unknown obf.mode '<m>' — expected 'fake-tls', 'obfs', 'plain' or 'reality-tls'` | опечатка в wire-режиме | исправить `obf.mode` |
| `profile '<n>': perf.connection.handshake_timeout_secs and perf.connection.max_clients must be > 0…` | один из flat-INI ключей явно задан нулём | удалить нулевой ключ для baseline-дефолта либо задать положительное значение |
| `profile '<n>': plain (raw) wire mode is TCP-only — set bind.transport = tcp` | `obf.mode=plain` на UDP | сменить транспорт на tcp |
| `profile '<n>': obfs wire mode requires a non-empty obfuscation.obfs_key…` | пустой `obfs_key` (публично выводим → нет DPI-защиты) | задать `obf.obfs_key` |
| `profile '<n>': reality_proxy.enabled requires at least one non-empty obf.tls.reality_proxy.short_ids entry…` | REALITY без short_id | задать `obf.tls.reality_proxy.short_ids` |
| `profile has an empty name` | секция вида `[profile:]` без имени | назвать профиль |
| `profile '<n>': obf.heartbeat.interval_ms must be > 0 when the heartbeat is enabled` | включён heartbeat с нулевым интервалом | задать `obf.heartbeat.interval_ms` |
| `profile '<n>': obf.heartbeat.jitter_ms (<j>) must be smaller than …` | джиттер ≥ интервала — расписание становится бессмысленным | уменьшить `obf.heartbeat.jitter_ms` |
| `profile '<n>': obf.heartbeat.data_size_bytes (<b>) must be <= <max>` | heartbeat-пакет крупнее допустимого размера записи | уменьшить `obf.heartbeat.data_size_bytes` |
| `profile '<n>': pool.cidr '<c>': <ошибка>` | пул не разбирается как CIDR (нет префикса, мусор, слишком узкий) | привести к виду `10.9.0.0/24` |
| `profile '<n>': invalid tun.address '<a>': … — expected a plain IPv4 address (e.g. 10.9.0.1)` | адрес с префиксом/маской или опечатка | оставить голый IPv4 |
| `profile '<n>': tun.address <a> is not a usable host inside pool.cidr <c>` | шлюз вне подсети VPN либо совпадает с адресом сети/broadcast | выбрать пригодный адрес внутри `pool.cidr`; его префикс — единственная настройка маски |

Не фатальные (профиль стартует), уровень WARN — просто предупреждают о
бессмысленной/слабой настройке: `obf.multipath.enabled has no effect on a UDP
transport…`, `obf.awg.enabled has no effect on a TCP … profile…`,
`reality_proxy.target '<t>' is a bare IP…`, `wire mode 'fake-tls' has LOW DPI
resistance…` (на UDP-профиле в этом последнем предлагается только `obfs` —
reality-tls живёт поверх TCP и на UDP недоступен).

### 4.1.1 Предстартовые проверки — служба не запускается вовсе (supervisor)

Отдельный класс: эти проверки выполняются **в супервизоре, до** того как поднимется
панель, стартует worker и появится хоть один TUN. Поэтому здесь не рестарт-петля
worker'а, а отказ службы целиком — и это намеренно: конфиг, попавший под такую
проверку, при запуске **отрезал бы доступ к самой машине**.

Проверяется пересечение адресации туннеля с тем, что хост уже использует. Худший
случай — `tun.address`, совпадающий с адресом шлюза: при подъёме TUN шлюз становится
локальным адресом, весь исходящий трафик умирает в туннеле, и сервер пропадает из сети
вместе с SSH и пингом, а в логе при этом всё выглядит как успешный старт.

| Сообщение | Причина | Фикс |
|---|---|---|
| `profile '<n>': tun.address <a> is this host's DEFAULT GATEWAY…` | адрес туннеля = шлюз хоста | увести туннель в свободный диапазон (`10.9.0.1` / `10.9.0.0/24`) |
| `profile '<n>': tun.address <a> is already assigned to interface '<if>'` | адрес уже занят интерфейсом хоста | выбрать адрес вне собственных сетей хоста |
| `profile '<n>': pool.cidr <c> contains this host's DEFAULT GATEWAY <gw>…` | пул накрывает шлюз | сменить пул |
| `profile '<n>': pool.cidr <c> contains <a>, the address of interface '<if>'…` | пул накрывает собственный адрес хоста | сменить пул |
| `profile '<n>': pool.cidr <c> overlaps the existing route <r> on interface '<if>'…` | пул пересекается с уже маршрутизируемой сетью (LAN, сеть провайдера) | сменить пул |
| `profile '<n>': pool.cidr <c> overlaps profile '<other>' pool <o>…` | два профиля делят диапазон | развести (`10.9.0.0/24`, `10.9.1.0/24`, …) |

Свои сети смотрите через `ip route` и `ip -4 addr`. Проверить конфиг **до** запуска:
`qeli check-config --config /etc/qeli/server.conf` — выполняет ту же проверку против
текущего хоста и печатает `would NOT start on this host — <причина>`.

Отдельный WARN: `pre-flight: could not read the host's network state (ip missing or
unreadable) — skipping the subnet-collision check`. Состояние хоста прочитать не
удалось, проверка пропущена, старт продолжается (fail-open — это защита от ошибки
оператора, а не граница безопасности). Убедитесь вручную, что `tun.address` и
`pool.cidr` не пересекаются с адресами, шлюзом и маршрутами хоста.

### 4.2 Хендшейк — до аутентификации (в основном DEBUG)

> **Ключевой момент:** эти ошибки возвращаются из `handle_client` и логируются в
> accept-цикле как **`Client <addr> disconnected on profile '<name>': <причина>`**
> на уровне **DEBUG**. При `info` — тишина. Включите debug (§1.1).

| `<причина>` в строке disconnected / отдельная строка | Что значит | Фикс |
|---|---|---|
| `handshake timeout for <addr>` | клиент не дослал ClientHello за `handshake_timeout_secs` (нет внутреннего таймаута на чтение — только этот внешний). Почти всегда = **PMTU-blackhole** большого PQ-ClientHello | см. §6.1 (MSS-clamp / MTU) |
| `failed to read ClientHello: <e>` | не прочиталась TLS-запись (обрыв/мусор) | сеть/MTU; проверить, что клиент шлёт fake-tls, а профиль — fake-tls |
| `failed to parse ClientHello` | `FakeTlsHandshake::parse_client_hello` вернул None (битая TLS-запись) | несовпадение wire-режима клиент↔сервер |
| `ClientHello missing the X25519MLKEM768 key_share` | клиент без ML-KEM (старый/классический) — PQ-гибрид обязателен во всех не-plain режимах | обновить клиент |
| `ML-KEM encapsulation failed (malformed ek)` | битый ключ ML-KEM в ClientHello | версия-скью/повреждение; обновить обе стороны |
| `rejected low-order client public key` | защита от small-subgroup (низкопорядковая X25519-точка) | клиент-баг/атака; обновить клиент |
| `invalid client public key length` | key_share ≠ 32 байт | версия-скью |
| `auth packet too short` / `invalid auth format` | первый пакет короче 32 Б / креды без `:` | версия-скью/повреждение |

### 4.3 Аутентификация — видно уже на `info` (WARN)

Если в логе есть `AUTH attempt … user=…`, значит хендшейк прошёл и дело в кредах/
правах. Все строки — **WARN** (видны без debug).

| Сообщение | Что значит | Фикс |
|---|---|---|
| `AUTH DENIED … — server key not pinned (require_client_key_proof)` | `auth.require_client_key_proof=true`, а клиент не пинит ключ сервера (нет/не тот `key=`) | прописать клиенту `key=<pubkey сервера>` (см. `qeli show-identity`) |
| `AUTH BLOCKED … — source IP locked for Ns…` | IP залочен брутфорс-защитой | подождать `lockout_secs`, или `qeli unblock <ip>`; проверить причину флуда |
| `AUTH FAIL … — not found or disabled` | юзера нет в БД или он выключен | проверить `users.conf` / `qeli add-client` |
| `AUTH FAIL … — wrong password` | неверный пароль (Argon2 не сошёлся) | перевыпустить ссылку (`qeli add-client … --link`) |
| `invalid password hash: <e>` | битый PHC-хеш пароля у юзера | пересоздать юзера |
| `AUTH DENIED … not permitted on profile '<n>'` | креды верны, но юзеру не разрешён этот профиль | добавить профиль в `profiles = …` юзера |
| `AUTH DENIED … — account expired` | истёк `expire_at` (Tier-2) | продлить аккаунт |
| `AUTH DENIED … — download quota exhausted (…GB down)` | выбрана квота скачивания | сбросить/поднять `data_limit_gb` |

Замечания: юзернейм **никогда** не лочится жёстко (анти-DoS), лочатся только
IP-адреса; неизвестному юзеру всё равно «тратится» dummy-Argon2 (анти-энумерация).

### 4.4 Приём соединений / rate-limit

| Сообщение | Уровень | Что значит |
|---|---|---|
| `New TCP connection from <addr> on profile '<n>'` | INFO | принято (прошло rate-limit), уходит в обработчик |
| `Rate limit exceeded for <ip> on profile '<n>'` | WARN | превышен лимит **новых соединений** с IP (`new_session_rate_max` за `new_session_rate_window_secs`) — соединение дропнуто **до** хендшейка. Частая причина — реконнект-шторм клиента или флуд probe'ов |
| `Accept error on profile '<n>': <e> — backing off 100ms` | ERROR | `accept()` упал (напр. EMFILE — исчерпаны fd); пауза 100мс от спина |
| `obfs accept failed for <addr> …` | DEBUG | не прошёл obfs/websocket-nonce обмен до qeli-хендшейка (несовпадение `obfs_key`/`fronting`) |

### 4.5 UDP-специфика

| Сообщение | Уровень | Что значит / фикс |
|---|---|---|
| `UDP handshake started for <addr> … (fragmented, QUIC-masked)` | INFO | принят ClientHello, отправлен ServerHello |
| `UDP handshake failed for <addr> …: <e>` | DEBUG | причина ниже |
| `UDP initial too small (NB < 1200B) — anti-amplification guard` | DEBUG | первый датаграм меньше 1200 Б — защита от рефлектор-амплификации. Нормой клиент паддит до ≥1200; если видите — старый/битый клиент |
| `UDP drop … no handshake permit (pre-auth crypto saturated)` | DEBUG | исчерпан семафор пре-авторизационного PQ-крипто (защита от спуф-флуда). Под реальной нагрузкой безобидно; под флудом — работает как задумано |
| `UDP drop … QUIC unwrap failed (<e>)` | DEBUG | датаграм заявил QUIC-маскировку, но не развернулся — несовпадение `quic` клиент↔сервер |
| `AUTH attempt UDP … user=…` → `UDP client … authenticated …, IP: …` | INFO | обычный успешный путь; auth использует те же WARN-строки из §4.3 |
| `UDP writer for <addr> kicked on profile '<n>'` | INFO | writer сессии получил kick: supersede (реконнект того же устройства) / session-cap / кража static-IP / reaper / over-quota. **Само по себе не ошибка** — см. §6.3 |

### 4.6 REALITY (`reality-tls` / reality-proxy)

Крипто REALITY молчит: невалидный клиент **прозрачно проксируется на `target`**
(защита от активного зондирования), обычно без лога или DEBUG
`REALITY: bridging non-Qeli connection … to <target>`.

| Сообщение | Уровень | Что значит |
|---|---|---|
| `REALITY: Qeli client detected from <addr> …` | INFO | клиент прошёл short_id-дискриминатор + anti-replay |
| `REALITY: Qeli client <addr> … failed after the handshake discriminator (likely config/version/core mismatch): <e>` | WARN | short_id совпал, но аутентифицированный carrier или следующий qeli-обмен упал — обычно рассинхрон конфига/версии/ядра (не probe). Сверьте `key`, `reality_sid`, версии и порядок обновления |
| `REALITY: genuine HTTP/2 carrier established with <addr>` | DEBUG | текущий H2 carrier установлен; далее идёт обычная qeli-аутентификация |
| `REALITY HTTP/2 carrier timed out/failed for <addr>: <e>` | WARN/error context | REALITY discriminator прошёл, но H2 не поднялся. Проверьте server-first порядок и уберите TLS termination/H2 conversion перед qeli |
| `REALITY: replayed session_id … — bridging as probe` | WARN | повтор session_id в окне (replay захваченного ClientHello) — забриджено как probe |
| `REALITY: failed to connect to backend <target>: <e>` | WARN | сервер не смог достучаться до decoy-сайта |

Условия, при которых клиент считается «не-qeli» и бриджится (тихо): не распарсился
ClientHello; key_share ≠ 32 Б; AEAD session_id не открылся **или** таймстамп вне
±120 с (проверьте часы!); **short_id не в allow-list** (`short_ids`). Последнее —
самая частая причина «reality не пускает»: `reality_sid` клиента должен быть в
`obf.tls.reality_proxy.short_ids` сервера.

На актуальном пути клиент также пишет INFO `REALITY-TLS carrier: genuine HTTP/2 stream`.
Обновляйте сначала сервер: новый сервер принимает H2 и legacy Reality carrier, новый клиент —
только H2. Reverse proxy/LB перед qeli должен работать как прозрачный TCP pass-through.

### 4.7 Веб-панель

| Сообщение | Уровень | Что значит / фикс |
|---|---|---|
| `Web panel NOT started: bind <addr> has NO admin password…` | ERROR | **fail-closed**: публичный бинд без `web.password_hash` → панель НЕ стартует (VPN работает!). Задать пароль: `qeli set-web-password`, включить `web.tls = true` |
| `Web panel on non-loopback <addr> WITHOUT TLS…` | WARN | публичный бинд без TLS — креды в открытом виде. Включить `web.tls` |
| `Web panel CSRF protection is DISABLED (web.csrf=false)…` | WARN | `web.csrf=false` (опасно на публичном бинде) |
| `panel: REFUSING live web-settings reload — … NO admin password…` | ERROR | live-reload панели тоже fail-closed |
| `Web UI (HTTPS) listening on https://<addr>` / `Web UI listening on http://<addr>` | INFO | панель поднялась |

---

## 5. Каталог ошибок — клиенты

Строки идентичны на **Windows и macOS** (общий data-plane `VpnTunnelBase`) и почти
идентичны на **Android** (свой Kotlin-порт с теми же сообщениями). Ниже —
объединённо; платформенные отличия помечены.

### 5.1 Подключение / хендшейк

| Строка | Что значит | Фикс |
|---|---|---|
| `Service started: TCP/fake-tls` (`+QUIC` для UDP+quic) | первая строка коннекта | — |
| `Connecting TCP/UDP <ip>:<port> as user '<u>'…` | резолв+коннект к серверу | если дальше нет `TCP connected` — порт закрыт/фаервол/не тот IP |
| `TCP connected` / `Bound carrier socket to …` | несущий сокет установлен | — |
| `ClientHello sent (NNNN B, hybrid X25519+ML-KEM)` | отправлен PQ-ClientHello | если дальше тишина → **PMTU** (см. §6.1) или сервер молча дропнул (режим/ключ) |
| `Server identity verified [OK]` | личность сервера сошлась | — |
| `Auth failed: <текст сервера>` | сервер ответил не `OK:` — **неверные креды/бан** | сверить юзера/пароль; на сервере смотреть WARN `AUTH FAIL` (§4.3) |
| `Failed to parse ServerHello` / `Failed to parse hybrid ServerHello` | ответ сервера не распарсился как ServerHello | версия-скью **или** UDP-реконнект с чужими пакетами / битый QUIC-фрейм (см. §6.2) |
| `Auth OK, IP 10.x.x.x` | сессия установлена, выдан IP | — |
| `Applied server-pushed obfuscation params` | применены пуш-настройки obfs | — |

**Крипто/пиннинг (Windows/macOS бросают `SecurityException` → терминальный стоп
без ретраев; Android — `[SECURITY]` + stop):**

| Строка | Что значит | Фикс |
|---|---|---|
| `[SECURITY] Server identity changed — possible MITM…` / `SERVER KEY MISMATCH - possible MITM` | пиннутый ключ ≠ ключ сервера | если ключ сервера **намеренно** сменился — убрать пиннинг/старую TOFU-запись и переподключиться; иначе это MITM |
| `SERVER KEY MISMATCH for <id> … Pinned <a>, got <b>. If you deliberately rotated the key, remove its line from <known_hosts>…` | TOFU-запись устарела | удалить строку сервера из known_hosts (десктоп) / очистить сохранённый ключ (Android) |
| `server sent proof-only but no server_public_key pinned` / `server auth proof INVALID` | доказательство личности не сошлось | сверить `key=` с `qeli show-identity` |
| `Pinned server key for <id> on first use (TOFU)…` | первый коннект — ключ запомнен (не ошибка) | для явного пиннинга задать `key=` |

**Guard'ы конфига на этапе коннекта (бросаются, не в парсере):**

| Строка | Что значит / фикс |
|---|---|
| `obfs wire mode requires a non-empty obfs_key (an empty key is publicly derivable → no DPI resistance)` | режим obfs без `obfs_key` — задать ключ |
| `reality-tls requires a pinned server key (auth.server_public_key)` / `server key must be 32 bytes (64 hex chars)` / `reality-tls requires reality_sid` | reality-tls без `key=`/`reality_sid=` — дозадать |
| `bind_static_to_session is on but no server key is pinned…` / `… all-zero TOFU sentinel…` | `bind_static` требует пиннинга ключа — задать `key=` или `bind_static = false` |

### 5.2 TUN / адаптер / маршруты

| Строка | Платформа | Что значит / фикс |
|---|---|---|
| `Wintun prewarm failed (<e>); will open in SetupTun` | Win | фоновое (параллельное хендшейку) создание адаптера не удалось — откроется синхронно (медленнее) |
| `NOTE: a Wintun driver (X.Y) is already loaded by another app…` | Win | другой VPN (OpenVPN/WireGuard/Tailscale) держит общий Wintun-драйвер иной версии — возможны конфликты; нужен совпадающий 0.14.x |
| `WintunCreateAdapter failed (err …; fresh name/GUID retries also failed)` | Win | не создать адаптер (нет прав администратора / повреждён драйвер). Запускать от админа |
| `WintunStartSession failed` / `WintunReceivePacket failed` | Win | сбой сессии Wintun |
| `utun: socket(PF_SYSTEM) failed (errno …) — are you root?` | mac | нет root — запустить через `sudo` или включить launchd-демон |
| `utun: connect failed / getsockopt(IFNAME) failed …` | mac | не открыть utun |
| `Failed to establish VPN interface` | Android | `VpnService.Builder.establish()` вернул null |
| `TUN establish with IPv6 failed (<e>); retrying IPv4-only` | Android | ROM отверг только синтетический IPv6-адрес блокировки утечки в IPv4-плане. Реальный согласованный IPv6-адрес никогда не downgrade'ится: такая ошибка фатальна |
| `WARN: could not determine physical gateway; full-tunnel may loop` | все | не найден физический шлюз — full-tunnel может зациклиться; проверить сеть/маршруты |
| `local = <addr>: not pinning the server route — carrier follows the bound interface's routing` | Win/mac | при заданном `local`/`lport` серверный bypass-маршрут не ставится (намеренно) |
| `Default route now via tunnel (0.0.0.0/1 + 128.0.0.0/1)` | все | full-tunnel поднят |
| `IPv6 captured into tunnel (…)` | все | закрыта dual-stack IPv6-утечка (`allow_ipv6_leak=true` отключает) |
| `Pinned server route <ip> via <gw>` | Win/mac | несущий маршрут к серверу через физ. шлюз |
| `exclude routes need Android 13+ (API 33); ignoring N` | Android | `exclude`/точечный LAN-bypass требует Android 13+ |
| `split: app not installed: <pkg>` | Android | пакет из per-app списка не установлен (пропущен) |
| `bad dns <ip>: <msg>` / `bad route <cidr>: <msg>` | все | сервер запушил/в конфиге битый резолвер/маршрут — пропущен |
| `<exe> <args> -> exit <code>: …` (`InvalidOperationException`) | Win/mac | обязательная команда `netsh`/`route`/`ifconfig` вернула ненулевой код — смотреть stdout/stderr в строке |
| `full tunnel: could not install route 0.0.0.0/1 …` / `… is not in the routing table … after being added` | Linux | **с 0.7.12 фатально.** Раньше это писалось в `warn` и клиент продолжал работу — половина IPv4 шла мимо туннеля при зелёном индикаторе. Теперь подключение отклоняется. Смотреть текст `ip` в строке: обычно нет прав (не root) или конфликт с уже существующим маршрутом |
| `full tunnel: could not pin the server bypass route …` | Linux | фатально: без обхода зашифрованный путь к серверу сам ушёл бы в строящийся туннель |
| `could not route included subnet <cidr> … refusing to run` | Linux | фатально: заказанная в `include` подсеть ушла бы в открытую |
| `could not install blackhole <half>` | Linux | в согласованном full-tunnel плане нет этой address family, а qeli не смог поставить fail-closed блокировку. Исправьте `ip route`/права, используйте dual-профиль либо осознанно включите соответствующий `allow_ipv4_leak`/`allow_ipv6_leak` |
| `kill-switch: could not install N allow rule(s) in QELI_KS_<if> …` | Linux | **с 0.7.12** цепочка не арминается, если не встало разрешающее правило (иначе хост отрезало бы от самого туннеля). Смотреть перечисленные правила |
| `interface '<dev>' already exists …` | Linux | см. §6 — свой осиротевший интерфейс забирается автоматически; отказ означает, что его держит **другой** процесс или это не tuntap |

### 5.3 Liveness / реконнект (почему рвётся и переподключается)

RX-watchdog считает только записи, прошедшие framing, проверку длины и AEAD-аутентификацию.
Reality/H2 принудительно выключает qeli heartbeat даже при старом local/pushed значении; liveness
обеспечивают carrier и обычный аутентифицированный трафик. В остальных режимах для heartbeat
порог равен `max(3×(interval+jitter), 30с)`, для shaping —
`max(3×(idle_gap_max+1с), 30с)`. При потере аутентифицированного downlink клиент рвёт линк
и переподключается. Если оба механизма выключены, RX-watchdog отсутствует. Backoff
экспоненциальный (потолок 60с), ретраи по умолчанию бесконечны.

| Строка | Что значит |
|---|---|
| `no authenticated data from server for >Ns` | до вычисленного порога `rxDead` не пришла валидная heartbeat/cover/data-запись. Сырая или поддельная UDP-датаграмма сессию живой не удерживает |
| `resumed after ~Ns suspend — reconnecting` | хост спал (стенные часы прыгнули ≫ монотонных) — немедленный реконнект. L1 |
| `Network changed — reconnecting` / `<reason> — reconnecting` | сменилась физическая сеть (Wi-Fi↔Ethernet/LTE) — проактивный `ForceReconnect`. Сопутствующая ошибка сокета (`recvfrom EBADF` / EBADF) **намеренно гасится** и в лог не идёт как `ERR:` |
| `Reconnect attempt N in Xs` | обычный backoff-ретрай |
| `Max retries reached, giving up` | достигнут заданный лимит ретраев (по умолчанию бесконечно) |
| `Reconnect disabled, giving up` | `reconnect = false` в конфиге |
| `Connection closed cleanly` | сервер закрыл соединение чисто |
| `ERR: [<Класс>] <msg>` + `  <- <причина>` | обобщённая ошибка цикла (сокет/хендшейк) — читать вложенные `<-` причины |

**Android-специфика:** `PacketTooLarge` / oversized-record под нагрузкой и
EMSGSIZE на UDP исторически валили цикл в reconnect-шторм — в актуальных сборках
паддинг обрезается под MTU, а UDP send-error дропает пакет (не фатально). Если
видите шторм на старом APK — обновите клиент.

### 5.4 Парсинг конфига

**Android** (`Config.kt`) — бросает исключения (в UI: тост `Invalid config: …`):
`config: missing [qeli] section`, `[qeli] missing required key 'server' (host:port)`,
`'server' must be host:port, got '…'`, `'server' has empty host`, `'server' has
invalid port: '…'`; для ссылок: `not a qeli:// link`, `qeli:// authority missing
:port`, `invalid port in qeli:// link`, `empty host in qeli:// link`,
`qeli:// authority malformed IPv6 [host]:port`.

**Windows/macOS** (`VpnConfig.cs`) — INI-парсер **лениентный, не бросает**: конфиг
без `[qeli]` даёт дефолты; **невалидный порт молча откатывается на 443**; guard на
пустой `obfs_key` — не в парсере, а на этапе коннекта (§5.1). Ошибки бросает только
`FromQeliUri` (те же `FormatException`, что выше). Редактор профиля валидирует поля
отдельно: `Enter the server address.`, `Invalid port (1–65535).`, `Enter the username.`.

---

## 6. Типовые сценарии

### 6.1 «accept → тишина с обеих сторон» = PMTU black-hole

**Симптом:** клиент `ClientHello sent (…B)` и висит; сервер `New TCP connection` /
`UDP handshake started` и дальше тишина; при debug — `handshake timeout for <addr>`.

**Причина:** PQ-ClientHello крупный (~1.4–1.5 КБ, с TLS/TCP/IP уже >1500). Если на
пути MTU < 1500 (PPPoE 1492, LTE/CGNAT, VPN-поверх-VPN) и ICMP «fragmentation
needed» режется — большой сегмент молча пропадает. TCP-рукопожатие прошло
(`New TCP connection` есть), а прикладной ClientHello/ServerHello не долетает.

**Фикс (сервер, обе стороны клэмпа):**
```bash
# сервер→клиент (ServerHello): клэмп на входящий SYN
iptables -t mangle -A PREROUTING -p tcp --dport 443 --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1240
# клиент→сервер (ClientHello): клэмп на исходящий SYN-ACK (обычно ставит установщик)
iptables -t mangle -A OUTPUT     -p tcp --sport 443 --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1240
iptables -t mangle -L OUTPUT -n -v | grep TCPMSS   # проверить, что применилось
# Если listener есть и на IPv6: 1280−IPv6(40)−TCP(20) = MSS 1220.
ip6tables -t mangle -A PREROUTING -p tcp --dport 443 --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1220
ip6tables -t mangle -A OUTPUT     -p tcp --sport 443 --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1220
```
IPv4 `--set-mss 1240` и IPv6 `--set-mss 1220` оба помещаются в путь MTU 1280. Подтверждение:
подключиться **с другой сети** (проводной Ethernet 1500). Если там работает — MTU
**вероятная**, но не единственная причина: тот же симптом дают DPI, NAT-hairpin
(см. §6.8), блокировка UDP и правила файрвола. Отличить просто: при MTU крупные пакеты
молча теряются, а мелкие ходят — то есть хендшейк проходит, а загрузка виснет. Если же
не устанавливается само соединение, MTU ни при чём. На клиенте можно снизить `mtu`
в профиле.

### 6.2 `Failed to parse ServerHello` на UDP-реконнекте

**Симптом:** первый коннект удачен, затем watchdog/событие сети инициирует реконнект →
`Failed to parse ServerHello` несколько раз; на сервере видно
повторную аутентификацию с **нового** source-порта и `UDP writer … kicked`.

**Причина:** UDP-реконнект с новым source-портом (NAT-ремап, особенно
VPN-поверх-VPN) + возможный рассинхрон QUIC-фрейминга/фрагментации ServerHello.
Актуальные сборки (0.7.11) переработали UDP-сессии (kick_all, фрагментированный
ServerHello, лечение утечки writer'ов). **Фикс:** обновить сервер до 0.7.11 или новее и
перетестить; проверить, что `quic` совпадает клиент↔сервер (`quic = true`/`quic=1`).

### 6.3 Reconnect-шторм / бан хостинга

**Симптом:** плотный цикл `Connecting… → Auth OK → closed/reconnect`, на сервере
`Rate limit exceeded for <ip>` и/или AUTH-флуд.

**Причины (все задокументированы в коде, issue #69):** преждевременный «Connected»
до поднятия TUN сбрасывал backoff; EMSGSIZE-петля на udp-quic; короткий (<5 Б)
UDP-record ронял цикл; быстрый Wi-Fi↔LTE флап без пола ретраев. **Фикс:** обновить
клиент (в 0.7.9+ добавлены пол реконнекта, «Connected только после TUN»,
дренаж UDP). На сервере — не снижать `new_session_rate_max` слишком агрессивно.

### 6.4 «Клиент не тот, что сервер» (ключ/режим)

**Симптом:** на сервере (info) `AUTH DENIED … server key not pinned`, или reality
`Qeli client … failed after the handshake discriminator`, или клиент — крипто-ошибка.

**Фикс:** сверить с `qeli show-identity --config <cfg>` публичный ключ; клиентский
`key=`, `mode=`, `reality_sid=` должны совпадать с сервером. Перевыпустить ссылку:
```bash
qeli add-client <user> --password '<pw>' --link --host <public-ip>:<port> \
  --link-profile <profile> --config /etc/qeli/server.conf
```

### 6.5 Серый индикатор профиля ≠ «не подключено»

Серая точка на карточке профиля — это **проба доступности сервера**
(Unknown/серый = ещё не проверяли), а **не** статус туннеля. Статус туннеля —
отдельный индикатор (Disconnected/Connecting/Connected/Error). Нажмите «Ping» /
подождите авто-опрос. Зелёный при подключённом активном профиле выставляется
напрямую (проба сквозь живой full-tunnel ненадёжна).

### 6.6 `protect() failed …` (Android) = конфликт с always-on VPN

`WARN: protect() failed for <label> after retries — the socket may not bypass the
tunnel (another active/always-on VPN, or VpnService not ready)` — почти всегда
установлен **другой always-on VPN**. Отключить его / снять «Always-on VPN» в
настройках Android.

### 6.7 Панель :8080 не поднимается, но VPN работает

`Web panel NOT started: non-loopback bind … NO admin password` (fail-closed). VPN
жив, только панель не стартует. Задать пароль (`qeli set-web-password`) + `web.tls = true`,
затем рестарт. Не путать со сбоем VPN.

### 6.8 Клиент и сервер в одной локальной сети → реконнект-петля

**Симптом:** клиент и сервер в **одной подсети** (например, оба `192.168.50.0/24`).
Хендшейк проходит полностью — `Server identity verified`, `Auth OK`, `TUN ready` — но
трафик не идёт: срабатывает аутентифицированный RX-watchdog (если включён
heartbeat/shaping) или сервер рвёт idle-сессию через ~20с (`Удаленный хост принудительно разорвал` / на сервере — реап неактивной
сессии) → бесконечный реконнект. **Тот же профиль с другой сети (интернет / другая
подсеть) работает** — это и есть главный признак.

**Причина (маршрутизация, не баг клиента/сервера):** десктоп-клиент пинит /32-маршрут
на сервер **через физический шлюз** (`Pinned server route <srv> via <gw>`), чтобы несущий
трафик не заворачивался обратно в туннель. Когда сервер **on-link** (та же подсеть, что и
клиент), это создаёт асимметрию: исходящие идут `клиент → шлюз → сервер`, а ответы —
`сервер → клиент` напрямую (та же подсеть). Шлюз пропускает пару пакетов хендшейка, но
рвёт устойчивую data-плоскость. С другой сети сервер реально за шлюзом → маршрут
симметричный → всё работает.

**Фикс:** в клиентском профиле задать `local` = IP этого хоста в локалке:
```ini
local = 192.168.50.50
```
При заданном `local` клиент привязывает несущий сокет к этому интерфейсу и **не** пинит
сервер через шлюз → сервер достаётся on-link напрямую → симметрия, туннель на той же
локалке работает. Быстрая проверка причины — подключиться с другой сети (проводной
Ethernet / мобильный интернет): если там работает, а в локалке нет — это оно.

Диагностика на сервере (пока клиент подключён, но трафик стоит): счётчики сессии
показывают `SENT`/`RECV` = 0 и растут только при реальном обмене — при этой проблеме
оба нуля даже под нагрузкой, т.к. асимметричный несущий поток не проходит.
```bash
qeli list-clients                      # SENT/RECV сессии (0/0 = data-плоскость не идёт)
```

### 6.9 Панель не даёт сгенерировать QR/ссылку — «нет прав на `/etc/qeli`»

**Симптом:** установка прошла, VPN работает, но в панели не выпускается ссылка или QR;
в логе — отказ по правам на `/etc/qeli/users.conf.lock` (или на сам `users.conf`).

**Причина.** Служба и панель работают от пользователя `qeli`, но CLI обычно запускают
через `sudo`. Атомарная запись — «записать во временный файл и `rename`» — подставляла
**новый inode, принадлежащий тому, кто писал**, поэтому один `sudo qeli add-client`
переводил `/etc/qeli/users.conf` из `qeli:qeli` в `root:root`. Файл блокировки создаётся
с владельцем охраняемого файла, тоже становился root-овым — и панель, работая от `qeli`,
больше не могла взять блокировку. `chown -R` из postinst тут не помогает: он отрабатывает
при установке, **до** этих записей.

**Исправлено** начиная с 0.7.13: атомарная запись сохраняет владельца заменяемого файла
(регрессионный тест `atomic_write_preserves_owner`, запускается от root). На **уже
сломанной** установке владельца надо вернуть руками — один раз:
```bash
sudo chown -R qeli:qeli /etc/qeli
sudo systemctl restart qeli
```
Проверка (всё должно принадлежать `qeli`):
```bash
ls -la /etc/qeli/
```

### 6.10 Клиент отказывается менять DNS: systemd-resolved не является резолвером

**Симптом:** подключение с `dns = tunnel` останавливается с сообщением
`refusing to replace /etc/resolv.conf with tunnel DNS`.

**Причина.** Клиент выбирает путь настройки DNS не по наличию бинарника, а по тому, кто
на этой машине **фактически резолвит**: указывает ли `/etc/resolv.conf` на stub
systemd-resolved. Если служба поставлена, но не включена, либо `resolv.conf` остался
обычным файлом (типовое состояние после удаления `resolvconf` на Ubuntu), то
`resolvectl dns` молча ничего не сделает.

Начиная с 0.7.15 qeli намеренно **не подменяет постоянный `/etc/resolv.conf`**: после
`SIGKILL`, сбоя питания или удаления клиента в нём мог остаться адрес исчезнувшего туннеля
и отключить DNS всей машины. Старые backup-файлы по-прежнему восстанавливаются при старте,
но новые не создаются. Включите безопасную per-link настройку:
```bash
sudo systemctl enable --now systemd-resolved
sudo ln -sf ../run/systemd/resolve/stub-resolv.conf /etc/resolv.conf
```
Проверка (должен быть симлинк на stub):
```bash
ls -l /etc/resolv.conf
```
Если DNS уже управляет NetworkManager, dnsmasq или платформа OpenWrt, оставьте это управление
ей и задайте `dns = off` в профиле qeli.

### 6.11 После сохранения настроек в панели службу приходится рестартить руками

**Симптом:** «Применить и перезапустить» отрабатывает без видимой ошибки, но служба
продолжает работать со старой конфигурацией.

**Причина.** Панель работает не от root, а `systemctl restart` непривилегированному
пользователю разрешает правило polkit. В `.deb` оно есть; при установке скриптом или
вручную — нет. Панель запрашивает у polkit право фактического пользователя службы на
фактический unit и при отказе возвращает `polkit_missing`. Сам файл правила не проверяется:
в Ubuntu каталог `/etc/polkit-1/rules.d` может быть недоступен пользователю `qeli`, хотя
polkitd успешно загрузил правило.
```bash
sudo qeli install-polkit
sudo systemctl restart qeli
```
Проверять нужно итоговое разрешение, а не чтение каталога с правилами:
```bash
sudo -u qeli systemctl restart qeli.service
```

**В контейнере** правило не поможет: `systemctl` там не управляет хостом, поэтому
«Применить и перезапустить» перезапустить службу не может — панель сообщает об этом
отдельно (`kind: container`). Перезапускать надо снаружи:
```bash
docker restart <имя-контейнера>
```

### 6.12 Клиенты: долгое восстановление после сна или разблокировки телефона

**Симптом:** после выхода из сна туннель поднимается около минуты; иногда за это время
туннель пропадает совсем и трафик идёт мимо VPN.

**Причина.** Экспоненциальный бэкофф реконнекта существует, чтобы не долбить лежащий
сервер, но он засчитывал и попытки, падавшие в **ещё не поднявшуюся сеть** (Wi-Fi
переассоциируется, DHCP не завершён). При базовой задержке 1с задержка удваивается
каждую попытку, так что несколько попыток, сгоревших за время подъёма сети, оставляли
клиента спать 16–32с уже **после** того, как сеть заработала. При заданном
`max_retries` те же попытки могли его исчерпать, а отказ от реконнекта снимает TUN и
маршруты — отсюда и уход трафика в обход туннеля.

**Чинилось в два приёма, оба в 0.7.13.** Первая правка ограничила паузы **между** попытками
(повтор не реже чем раз в ~4с в течение 30с после пробуждения). Сама по себе она верна, но
главного не покрывала: время уходило **внутри одной попытки**, и в сборках 0.7.13 до второй
правки задержка оставалась прежней.

**Что закрыла вторая правка.** Резолв имени шёл блокирующим вызовом **без таймаута**, а на
connect и на чтения хендшейка отсчитывался `ConnectionTimeoutSecs` — по умолчанию **30с
каждый**. Одна неудачно попавшая попытка перекрывала всё окно оседания целиком. Теперь на
время оседания весь этап до data-plane (резолв + connect + хендшейк) ограничен 5с, а резолв
ограничен по времени всегда. Вдобавок само окно в самом частом случае вообще не взводилось:
оно ставилось за проверкой «туннель ещё подключён», а после сна туннель к моменту события
Resume уже мёртв. Отдельных действий не требуется — нужна актуальная сборка 0.7.13.

**Закрытие мобильного и headless-сценария в 0.7.15.** Android wake lock не даёт уснуть CPU,
но не сохраняет Wi-Fi association или NAT mapping, а тот же объект Android `Network` может
пережить смену DHCP/link. Теперь сервис сравнивает capabilities, адреса, маршруты и DNS этой
сети, а после включения экрана коротко ждёт готовности физического IPv4-пути и заменяет native
generation, сохраняя TUN. iOS после `PacketTunnelProvider.wake()` тоже заменяет установленную
generation, а не только пишет событие в лог. Windows Service и macOS launch daemon сами опрашивают
отфильтрованную сигнатуру физической сети: GUI-callback не владеет headless-туннелем. На Android
и iOS одновременно допускается только один незавершённый блокирующий resolver call, поэтому
повторные реконнекты не накапливают DNS-потоки.

Ручное отключение в 0.7.15 — тоже асинхронная граница. Android показывает `Отключение`, пока
Rust runner не завершился и не закрыл все дубликаты TUN-дескрипторов; только после этого сервис
публикует `Отключено` и разрешает новое подключение. Для DNS это существенно: запуск новой
generation при ещё живом старом TUN мог оставить системный resolver Android на дескрипторе, у
которого уже нет data plane. Если после ручного отключения пропадает DNS, нужен лог от
`Отключение` до следующего `Подключено`; предупреждение о teardown дольше 5 секунд укажет на
native-владельца дескриптора, который не остановился вовремя.

Проверить по логу (вкладка **Журнал** → **Copy log**): после пробуждения должна появляться
строка `Network settling — short attempt budget 5s for the next 30s`. Если она есть, а
восстановление всё равно долгое — время уходит в другом месте, и такой лог от пробуждения
до `Connected` нужен целиком. На мобильной 0.7.15 в этом интервале также ожидаются
`Device woke` / `Device wake: replacing...` (iOS) либо
`Device woke after ... screen-off — reconnecting` (Android).

### 6.13 Панель за reverse-proxy: 404 либо выброс в корень

**Симптом, по которому диагноз ставится сразу:** с `base_path` панель отдаёт **404**, а без
него — грузится, но выбрасывает в корень сайта.

> ⚠️ **Сначала проверьте, что в строке `base_path` нет комментария.** В flat-INI `#` и `;`
> начинают комментарий **только с начала строки**, поэтому
> `base_path =    # оставить пустым` задаёт значением literal `# оставить пустым`. Панель
> монтируется под префиксом, которого никто не запрашивает, и **все** маршруты, включая
> `/login`, отдают 404. Начиная с этой версии сервер такое значение отвергает и пишет в лог
> `web.base_path = "…" is not a plain URL path`, поднимая панель в корне. Пустое значение
> пишется просто как `base_path =`, без хвоста.

**Причина.** У префикса два независимых потребителя, и настраиваются они разными вещами:

| Что | Откуда берётся | Когда применяется |
|---|---|---|
| Монтирование маршрутов | **только** `web.base_path` | на старте процесса |
| `<base href>` и редиректы | `X-Forwarded-Prefix`, иначе `web.base_path` | на каждый запрос |

Отсюда две рабочие конфигурации — и они взаимоисключающие:

| | `web.base_path` | Прокси |
|---|---|---|
| A | пусто | режет префикс, шлёт `X-Forwarded-Prefix` |
| B | `/qeli` | префикс **не** режет |

**404 при заданном `base_path` означает, что ваш прокси префикс режет** — то есть вам нужен
вариант A, а не B. В nginx это решает слеш в конце `proxy_pass`: без слеша исходный URI
уходит целиком (вариант B), со слешем — префикс срезается (вариант A).

**А выброс в корень — это `trusted_proxies`.** Страницы грузятся, потому что относительные
ссылки разрешаются от URL запроса. Но каждая страница без сессии делает
`Redirect::to("/login")`, а логин — `Redirect::to("/")`, и префикс к этим редиректам
добавляется **только если он непустой**. `X-Forwarded-Prefix` принимается лишь от адреса,
указанного в `web.trusted_proxies` — при пустом списке заголовок отбрасывается, префикс
становится пустым, и редирект уводит на корень сайта.

> Именно поэтому симптом кажется плавающим: с живой сессией редиректа нет и всё работает,
> а после рестарта службы или истечения сессии панель начинает «выбрасывать в корень».

Рабочий вариант A целиком:

```ini
[web]
bind = 127.0.0.1
base_path =
trusted_proxies = 127.0.0.1
public_host = вашдомен.ru
```

```nginx
location /qeli/ {
    proxy_pass http://127.0.0.1:1444/;          # слеш ЕСТЬ — срезает /qeli
    proxy_set_header X-Forwarded-Prefix /qeli;
    proxy_set_header Host              $host;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

`base_path` применяется только при **полном** рестарте процесса, `trusted_proxies` —
подхватывается живым.

Проверка:
```bash
curl -s https://вашдомен.ru/qeli/login | grep -o '<base href="[^"]*"'
```
Ожидается `<base href="/qeli/">`. Если `/` — в логе сервера ищите строку
`panel: ignoring X-Forwarded-Prefix from … not covered by web.trusted_proxies`: она прямо
называет адрес, который надо внести в список.

### 6.14 macOS: после удаления Qeli остался DNS `10.9.0.1`

`/etc/resolv.conf` в macOS генерируется системой; не исправляйте его вручную. Сначала
посмотрите recovery-журнал — в `previousServers` могут быть ваши собственные DNS, которые
нужно вернуть вместо `empty`:

```bash
sudo cat "/Library/Application Support/Qeli/dns-override.json" 2>/dev/null
sudo launchctl bootout system/ru.qeli.app.daemon 2>/dev/null || true
sudo launchctl bootout system/ru.autocash.qeli.daemon 2>/dev/null || true
sudo rm -f /Library/LaunchDaemons/ru.qeli.app.daemon.plist
sudo rm -f /Library/LaunchDaemons/ru.autocash.qeli.daemon.plist
networksetup -listallnetworkservices
sudo networksetup -setdnsservers "Wi-Fi" empty
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
networksetup -getdnsservers "Wi-Fi"
scutil --dns
```

Замените `Wi-Fi` точным именем активной службы. Если `previousServers` содержит адреса,
передайте их команде `-setdnsservers` вместо `empty`. Только после успешной проверки можно
удалить старый журнал:

```bash
sudo rm -f "/Library/Application Support/Qeli/dns-override.json"
```

В 0.7.15 daemon хранит намерение Connect отдельно от установки, проверяет настоящий
`launchctl bootout` и не подтверждает Disconnect, пока исходный DNS не восстановлен.

---

### 6.15 Linux: hooks игнорируются или password_command запрещён после chmod

Сообщения с `ignoring post_up`, `ignoring post_down` либо
`refusing to run auth.password_command` содержат причину запрета команды.
Используйте обычный конфиг вместо symlink, с владельцем root либо эффективным UID службы,
без записи для группы/остальных (обычно `chmod 600 /path/to/config`). Файл должен оставаться
доступен на чтение реальному пользователю службы. Проверьте также скрипты и их зависимости.

Затем перезапустите клиент или worker сервера. Одного исправления прав, повторного запуска
профиля либо SIGHUP недостаточно, чтобы разрешить команды из ранее недоверенного конфига.
Разрешённая очистка старого работающего поколения по-прежнему может выполнить сохранённый
`post_down` после изменения пути; новый конфиг применяется при следующем запуске worker.

`configuration changed while reading; retry with a stable file` означает обнаруженное
изменение содержимого или метаданных. Дождитесь завершения записи и повторите запуск;
при сохранении предпочтительна атомарная замена. `configuration must be a regular file`
отклоняет каталоги, устройства и FIFO. См. [безопасность конфигурации](CONFIG.md#безопасность).

---

### 6.16 Linux: таймаут поставщика пароля, большой вывод или скрытый stderr

`auth.password_command exceeded its execution/output deadline` означает, что выполнение,
EOF stdout либо выход shell не завершились за 30 секунд. Используйте неинтерактивный
поставщик: stdin закрыт. Фоновый потомок с открытым stdout тоже расходует этот срок;
перенаправьте его потоки, если он намеренно должен продолжить работу.

`stdout exceeds 16384 bytes` отклоняет весь вывод, в том числе короткий пароль с большим
количеством пробелов. `stdout is not valid UTF-8` отклоняет некорректные байты. Выводите
в stdout только пароль; после trim действует меньший лимит AUTH.
`failed with ...; command output is not logged` сохраняет статус выхода, скрывая stderr
от журналов Qeli. При необходимости исследуйте поставщик в защищённой сессии оператора;
не публикуйте секреты в обращениях и не включайте диагностику с паролями.
SIGINT/SIGTERM во время ожидания поставщика отменяют запуск и очищают его process group.

---

### 6.17 Linux: отказ password_file или ожидание файлового I/O при остановке

`auth.password_file must resolve to a regular file` отклоняет FIFO, устройство или каталог.
Используйте обычный файл секрета; symlink хранилища секретов поддерживается. `exceeds 16384
bytes`, `is not valid UTF-8` и `changed while reading` отклоняют весь пароль. Удалите лишний
вывод, исправьте кодировку либо закончите запись/атомарно замените файл перед повтором.
Сообщение ошибки не содержит пароль.

`auth.password_file exceeded its read deadline` относится к бюджету ожидания/чтения
30 секунд. Задача в очереди отменяется, но уже выполняющийся файловый syscall должен
вернуться до завершения штатного stop/timeout. Если остановка ждёт, проверьте файловую
систему и состояние mount. Повторная отмена вызывающей стороны не создаёт дополнительные
чтения: работающая задача удерживает единственный слот процесса до возврата I/O и выхода.

Финальный статус клиента записывается после отмены и join sampler/signal watchers.
Поэтому старый снимок sampler не перезапишет `stopped`/`failed`; зависшая синхронная запись
диагностики тоже может задержать финальную публикацию. После SIGKILL либо принудительной
отмены всей клиентской future терминальный статус не гарантируется.

---

### 6.18 Linux: kill-switch сохранён после ошибки очистки

`kill-switch retained because forwarding/NAT cleanup did not complete` означает, что
предыдущая ошибка не позволила надёжно очистить gateway/exit-node. Qeli намеренно
сохраняет включённый kill-switch. Исправьте указанную ошибку firewall/утилиты и повторите
штатную очистку либо контролируемое восстановление администратором; до этого сеть может
оставаться ограниченной. При отключённом kill_switch сообщение не выдаётся. Ошибка самого
снятия kill-switch означает другое: удаление могло частично выполниться, сохранение не
обещается.

---

### 6.19 Linux: ошибка запуска или остановки transport core

После настройки kill-switch ошибка жизненного цикла ядра завершает клиент со статусом ошибки;
она не повторяется как обычный отказ соединения. `post_down` получает `core_start_failed` /
`core_start` либо `core_stop_failed` / `core_stop` (причина / код ошибки). При отказе обоих этапов
приоритет имеет `core_stop_failed`, а сообщение сохраняет причины запуска и очистки.
Одновременный SIGINT/SIGTERM не превращает отказ остановки ядра в успешную остановку.

`kill-switch retained because transport core teardown did not complete` означает, что Qeli
не смог подтвердить остановку ядра. Очистка forwarding всё равно выполняется. Если она тоже
не удалась, сообщение указывает обе причины. Сохраните журнал ошибки и проверьте состояние
процесса, firewall и маршрутов перед восстановлением администратором; вызов хука не доказывает
успешный сброс сети. Пользовательские hook-скрипты могут самостоятельно менять firewall.

---

### 6.20 Linux: восстановление старого DNS не удалось; снимок сохранён

`failed to restore /etc/resolv.conf ... (backup kept at ...)` сообщает об ошибке восстановления
снимка от старой версии Qeli. Ошибка удаления/замены файла или восстановления прав сохраняет
снимок для повторной попытки. Запись типа `file` без содержимого, некорректная цель symlink,
неизвестный тип и повреждённые данные отклоняются: resolver не заменяется пустым файлом
молча. Явно заданное пустое содержимое исходного файла допустимо.

Проверьте указанную ошибку файловой системы и сохранённый оригинал перед восстановлением.
Не удаляйте снимок ради подавления ошибки. Если восстановление удалось, а удалить снимок
не получилось, диагностика сообщает об этом отдельно. Новые подключения используют DNS
через systemd-resolved для интерфейса; этот путь совместимости не включает перезапись
системного resolver-файла для новых соединений.

---

### 6.21 Linux: зарегистрированы ошибки очистки сетевых ресурсов

`kill-switch retained because network resource cleanup reported errors` означает, что
в этом запуске клиента произошла ошибка очистки DNS, маршрутов, TUN либо отката NetworkPlan.
Клиент пытается очистить forwarding, завершает работу с ошибкой и не выполняет reconnect.
`post_down` получает `network_cleanup_failed` / `network_cleanup`; если отказала и остановка
ядра, приоритет имеют `core_stop_failed` / `core_stop`. Одновременный сигнал остановки не
скрывает ошибку. Если сервер также прислал terminal kick, его причина сохраняется в ошибке.

Guard может повторить очистку, но поздний успех не стирает исходный отказ и не снимает
kill-switch автоматически. Проверьте первую ошибку каждого ресурса и текущее состояние DNS,
маршрутов и интерфейса перед восстановлением администратором. Запись ограничена четырьмя
категориями ресурсов по первым 2048 символам; это не полная история всех повторов.
Пользовательские hook-скрипты могут самостоятельно изменять firewall.

---

### 6.22 TCP: остановка ожидает фоновые операции

Штатная остановка закрывает создание TCP-задач и ждёт завершения reader/writer, decrypt
pipeline и обслуживания соединения перед очисткой сети. Ошибка управляющего события тоже
проходит эту последовательность. Linux-монитор путей в TCP и UDP дополнительно ждёт уже
начатое чтение маршрутов или применение обновления пути. Отдельные команды маршрутов,
firewall, TUN и resolvectl имеют [пределы](#627-linux-system-command-timed-out-или-output-limit-exceeded);
общий срок остановки также зависит от числа команд, проверок и ожидания выхода процессов.

При задержке проверьте журнал и состояние дочерних ip/iptables/resolvectl; не считайте
сетевую очистку завершённой только по запросу остановки. Принудительное завершение процесса
не гарантирует join, восстановление сети и post_down. [Отчёт и пределы проверки](../reports/AUDIT-Q25-TCP-TASKS.md).

---

### 6.23 UDP: завершение кандидата и старого пути

При остановке соединения Qeli ждёт задачи приёма, подключение кандидата и Linux-монитор
перед откатом платформенного пути и очисткой DNS/TUN. Отказ или истечение кандидата
завершает его receive pump; рабочий путь продолжает приём. После commit приём со старого
пути продолжается только в предусмотренном окне drain, затем его задача завершается.

Завершение не должно зависеть от прихода следующего пакета на тихий сокет или свободного
места в очереди. Задержку системной blocking-операции всё ещё нужно проверять отдельно.
Принудительная отмена всего клиента не подтверждает rollback или async join.
[Отчёт и проверенные сценарии](../reports/AUDIT-Q25-UDP-TASKS.md).

---

### 6.24 Завершение потоков TUN/Wintun при отмене остановки

Даже если ожидание shutdown отменено, уничтожение pump дожидается его reader/writer.
При отмене это может выполняться синхронно. Занятый blocking pool не мешает такому Drop
самому завершить join; уже работающий join дожидается обоих потоков.

Stop и закрытие входной очереди позволяют выйти из ожидания пакетов и blocking_send.
Ограниченные ожидания циклов не гарантируют конечный срок вызовов драйвера/ОС. Если
остановка зависла, различайте ожидание TUN workers, системной команды и платформенного
ACK по журналу/дампу потоков. Запрос stop или статус интерфейса не доказывает завершение
всей сетевой очистки. [Проверенные сценарии и ограничения](../reports/AUDIT-Q25-TUN-WORKERS.md).

---

### 6.25 HTTP/2: остановка при задержанном response или полном окне

Для `reality-tls` клиент учитывает внутренние H2 driver/bridge вместе с TCP-задачами,
начиная с подключения. Штатное завершение группы ждёт их освобождения до DNS/TUN cleanup.
Отмена native TCP-попытки также ждёт группу перед учётом завершения поколения.

Нулевое окно peer или заполненный bridge не должны требовать нового сетевого события
для отмены. Half-close остаётся штатным: после завершения отправки допускается ответ.
Это не обещание общего deadline остановки, подтверждение раннего rollback или гарантия
join после уничтожения всего runtime. [Отчёт и сценарии](../reports/AUDIT-Q25-H2-TASKS.md).

---

### 6.26 Серверный HTTP/2: остановка профиля и отклонённые запросы

В `reality-tls` профиль учитывает H2 driver, bridge и ограниченный flush отказа вместе
с задачами сессий. Штатный teardown ждёт их освобождения; отмена ожидания shutdown
сохраняет возможность повторного join. Нулевое окно peer не должно блокировать отмену.

Неверный H2-запрос по-прежнему получает соответствующий HTTP-статус. Пока соединение
отправляет отказ, его pre-auth слот остаётся занят; flush ограничен одной секундой.
H2 200 ещё не означает успешную внутреннюю AUTH. Общий срок shutdown и поведение при
уничтожении runtime этим не гарантируются. Проверки на Linux runtime остаются открытыми;
[отчёт и воспроизводители](../reports/AUDIT-Q14-H2-TASKS.md).

---

### 6.27 Linux: system command timed out или output limit exceeded

Для клиентских команд маршрутов, kill-switch/gateway, интерфейса TUN и `resolvectl`
эти ошибки означают превышение
15 секунд либо 16 МиБ на один поток вывода. Ошибка запуска и ненулевой exit code
обрабатываются отдельно. Qeli пытается завершить процесс, на Linux — также его группу,
и ожидает выход; частичный вывод не принимается за результат команды.

Не считайте timeout доказательством отсутствия изменений. При ошибке `resolvectl revert`
marker остаётся для повторного восстановления. Ошибка немедленного rollback теперь видна
в журнале; сообщение о попытке отката не означает успешный revert. Проверьте конкретный
интерфейс и состояние systemd-resolved. Полный срок shutdown остаётся зависимым от других
операций: команды и проверки выполняются последовательно, а kill/reap может ждать kernel.
[Исходный runner и тесты](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md),
[маршруты и firewall](../reports/AUDIT-Q25-CLIENT-COMMANDS.md).

---

### 6.28 Сервер: NAT cleanup ... incomplete

Это предупреждение означает, что очистка правил не подтверждена: команда чтения,
удаления или проверки завершилась ошибкой либо собственные правила остались после
успешного ответа на удаление. Журнал указывает таблицу/цепочку и причину. Qeli выполняет
конечный проход по найденным правилам и продолжает остальные цепочки при отказе.

Проверьте указанную цепочку тем же backend (`iptables` или `ip6tables`), доступность
утилиты и права процесса. У смешанных native nft-цепочек `-S` может не работать даже
при рабочем точечном DNS cleanup. Не считайте это предупреждение подтверждением
оставшегося правила без проверки: чтение могло завершиться ошибкой. Аналогично,
успешный выход сервера пока не доказывает восстановление firewall. Не очищайте целиком
таблицу администратора ради одного профиля. [Охват и открытые замечания](../reports/AUDIT-Q14-NAT-CLEANUP.md).

---

### 6.29 Linux: firewall inspection failed / DNS INPUT cleanup failed

Ошибка проверки firewall теперь отличается от подтверждённого отсутствия правила.
Проверьте путь инструмента, его backend, права и конкретную причину в stderr. Отказ
доступа с кодом 1 не означает, что правило уже удалено. Неизвестное или дополнительное
сообщение также оставляет состояние неподтверждённым; приложите полный журнал при разборе.

При очистке серверных DNS permits Qeli проверяет обе ветви UDP/TCP, даже если одна
завершилась ошибкой. Удаление ровно 1024 одинаковых правил поддерживается; сообщение
`still present after 1024 deletion attempts` означает, что последняя проверка ещё видела
правило. Возможны накопленные копии, конкурентное добавление или успешный no-op backend.
Итоговый retry известных DNS leases теперь влияет на код выхода worker — см. §6.31.
Это ещё не подтверждает очистку всех ресурсов сервера. [Отчёт о проверках](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md).

---

### 6.30 Сервер: exact DNS INPUT ownership retained for retry

Удаление DNS permits не подтверждено, но worker сохранил их полное описание для
повторения. Проверьте причину перед этой строкой: доступность `iptables`/`ip6tables`,
права, backend и результат exact check/delete. Последующая очистка профиля и новая
попытка установки его DNS правил сначала повторяют неудачную очистку. Пока она
не прошла, новые DNS permits этого профиля не устанавливаются.

`DNS INPUT ownership limit reached (4096)` означает заполнение реестра активными
и ожидающими очистку наборами правил. Каждый resolver занимает один набор UDP+TCP;
IPv4/IPv6 занимают отдельные наборы. Сначала устраните ошибки очистки; успешное
освобождение правил возвращает место, старые записи автоматически не вытесняются.

Сведения находятся только в памяти текущего worker. Не рассчитывайте на сохранение
этого retry после crash или перезапуска; отдельного persistent journal пока нет.
Итоговая проверка перед выходом worker описана в §6.31; она не заменяет проверку всех
ресурсов сервера. [Отчёт и ограничения](../reports/AUDIT-Q14-DNS-OWNERSHIP.md).

---

### 6.31 Сервер: Server shutdown failed — owned network cleanup

Worker после завершения профилей повторяет очистку сохранённых DNS INPUT rules и
оставшихся IPv6 sysctl leases. Если она не подтверждена, остановка worker по сигналу
завершается с кодом 1 и строкой `Server shutdown failed: owned network cleanup: ...`.
При других сбоях строка также содержит `worker`, `profile/worker task cleanup`
и/или `usage shutdown flush`.
Статистика сохраняется даже после ошибки очистки сети.

`DNS INPUT lease still active at worker shutdown` означает оставшееся активное владение;
проверка не удаляет такие правила. При обычном отказе cleanup проверьте указанную
firewall-утилиту, доступ к sysctl и предыдущие ошибки профиля. Если последний retry
подтвердил очистку, прежний временный отказ сам по себе не меняет успешный код выхода.

Внешний supervisor передаёт ошибку финальной остановки worker вызывающему CLI и пишет
`Supervisor shutdown failed: ...`. Nonzero exit и принудительный kill после grace
не считаются успешной остановкой. Успешный выход worker даёт успешный результат.
При явном Restart или неожиданном падении без запроса остановки supervisor по-прежнему
перезапускает worker; ошибка завершения журналируется.

Эта проверка покрывает только известные worker DNS/IPv6 sysctl leases. Успешный код
не доказывает отсутствие generic NAT правил и ресурсов прежних поколений.
Учёт ошибок задач профиля и TUN teardown описан в §6.32.
DNS ownership после выхода процесса теряется; автоматическое восстановление exact
rules после перезапуска пока не гарантировано.
[Отчёт и открытые границы](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md).

---

### 6.32 Сервер: profile/worker task cleanup и teardown incomplete

`Server shutdown failed: profile/worker task cleanup: ...` означает ошибку завершения
задач или текущего поколения профиля. Вложенный текст различает listener/service/child
panic, ошибку профильного supervisor, TUN queue timeout/panic и отказ удаления TUN.
Профиль также может записать `teardown incomplete: ...`.

Ошибка одного профиля не отменяет ожидание остальных, итоговую очистку известных
DNS/IPv6 sysctl leases и сохранение статистики. Worker при остановке по сигналу возвращает
exit 1, внешний supervisor передаёт отказ вызывающему CLI. Обычная отмена дочерней
задачи при shutdown не является ошибкой. Отмена ожидания или повторный вызов shutdown
не стирают уже собранную диагностику задач.

При `queue thread(s) did not stop` поток не завершился за три секунды и может удерживать
устройство. Проверьте предыдущие ошибки профиля и указанный TUN; автоматический повтор
удаления TUN этим механизмом не выполняется. Сообщение `teardown attempted` сообщает
о попытке очистки, а не подтверждает отсутствие всех NAT правил или старых устройств.
Ошибки прежнего поколения после перехода к retry/replacement требуют отдельного учёта.
[Проверки и ограничения](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md).

---

### 6.33 Сервер: could not restore stale host sysctl value(s)

При старте worker восстановление журнала sysctl обнаружило параметры без живого
владельца, которые не удалось восстановить. Запуск возвращает ошибку; в сообщении
перечислены пути. Проверьте доступ службы к указанным sysctl и предыдущие сообщения
`host networking`. Не удаляйте `sysctls.state`: сохранённые исходные значения нужны для retry.
После устранения причины повторный запуск повторит восстановление.

Один сбой не пропускает остальные записи журнала. Значения с живыми владельцами
сохраняются, как и внешние изменения администратора. Неудачное повторное получение
lease теперь сохраняет прежнего владельца, чтобы recovery не восстановил original
под ещё работающим компонентом. Политика перезапуска supervisor остаётся прежней.
[Отчёт и границы проверки](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md).

---

### 6.34 Сервер: IPv6 sysctl acquisition failed — rollback incomplete

Ошибка настройки route/nat66 может включать и исходный отказ acquire, и
`rollback incomplete`: не удалось подтвердить восстановление sysctl после отказа.
Это возможно даже при ошибке первого accept_ra, поскольку запись могла выполниться
до неудачной проверки. Профильный scope сохраняется для повторной очистки.

Обычный cleanup профиля и итоговая остановка worker повторяют освобождение scope.
Если оно всё ещё не удаётся, итог содержит `owned network cleanup` с `IPv6 sysctls/<profile>`.
Проверьте доступ службы к указанным sysctl и журналу, сохраните `sysctls.state` для retry.
Успешный финальный retry снимает эту сетевую ошибку; исходный отказ текущего поколения
может отдельно оставаться в `profile/worker task cleanup`.

Пока старый scope не освобождён, другой WAN/TUN для того же профиля не подменяет его.
Порядок accept_ra → forwarding и режимы `off`/`manual` сохранены; новых INI-ключей нет.
[Отчёт и проверки](../reports/AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md).

---

### 6.35 Сервер: system command timed out / output limit exceeded в NAT cleanup

Команды из серверного NAT-слоя, включая iptables/ip6tables, PATH version probes и
WAN route lookup, используют срок 15 секунд и лимит 16 МиБ отдельно для stdout/stderr.
`--wait 5` остаётся ожиданием xtables lock внутри этой попытки. При timeout процесс
завершается через общий runner; превышенный вывод не передаётся парсеру частично.

`system command timed out` не означает, что firewall остался неизменным. Проверьте
причину задержки backend/xtables и предыдущие сообщения профиля. Exact DNS rule specs
сохраняются для повторной очистки в текущем worker; не считайте ошибку проверки отсутствием
правила. Generic NAT sweep сохраняет прежний best effort и журналирует неудачу.

Это ограничение отдельной команды: полная последовательность cleanup и ожидание
завершения процесса могут занять больше времени. Preflight отдельно покрыт в §6.36;
клиентские firewall/routes этим изменением не покрываются. Новых INI-параметров нет.
[Охват и подтверждённые проверки](../reports/AUDIT-Q14-NAT-COMMANDS.md).

### 6.36 Сервер: задержка preflight или недоступное состояние сети

Четыре запроса `ip` для адресов/маршрутов IPv4/IPv6 теперь используют общий runner:
15 секунд на команду, по 16 МиБ stdout/stderr. Превышенный вывод не разбирается частично.

Предупреждение `pre-flight: could not read the host's network state` означает, что
IPv4 snapshot недоступен: причиной могут быть отсутствие `ip`, ненулевой exit,
ошибка чтения, timeout или превышение лимита. По существующей политике старт допускается;
отсутствие сетевой коллизии этим не подтверждается. Проверьте адреса и маршруты хоста.

Отказ чтения IPv6 адресов или маршрутов не отбрасывает IPv4 и доступную часть IPv6;
отдельного предупреждения для такого частичного отказа пока нет. Наблюдаемая коллизия
по-прежнему блокирует применение. Успешный пустой вывод допустим.

Весь preflight и транзакция панели могут занять дольше 15 секунд: команды выполняются
последовательно, а ожидание завершения процесса может продлить вызов. Синхронное
ожидание в обработчиках панели пока сохраняется. Новых INI-параметров нет.
[Охват и ограничения](../reports/AUDIT-Q05-PREFLIGHT.md).


### 6.37 Linux-клиент: ошибка наблюдения пути или задержка остановки

При debug-логе `Linux roaming path sample failed` монитор не смог получить пригодное
наблюдение маршрутов/адресов. Его три read-only запроса `ip` теперь имеют срок 15 секунд
каждый и лимиты stdout/stderr по 16 МиБ. Timeout или превышение вывода возвращает ошибку;
неполные данные не публикуются как новый путь. Служебный вывод iproute2 не меняет
INI-формат пользовательских профилей.

При штатной остановке клиент ожидает уже запущенную команду через владельца поколения,
даже если async-монитор отменён. Это не срок всей остановки: запросы последовательны,
ожидание процесса может продлить вызов. Изменяющие маршруты команды также ограничены
по отдельности. При проблеме проверьте доступность iproute2 и предыдущий debug-лог.
[Проверки и границы](../reports/AUDIT-Q25-PATH-MONITOR.md).


### 6.38 Linux gateway/exit-node: определение WAN и очистка

WAN выбирается независимо для IPv4 и IPv6. Сначала Qeli читает default route, затем
использует fallback — локальный route-get для `1.1.1.1` или `2606:4700:4700::1111`.
Каждый запрос имеет срок 15 секунд и отдельные лимиты stdout/stderr по 16 МиБ.
После ошибки первого запроса fallback может дать пригодный WAN; отказ обоих оставляет
WAN недоступным. Проверьте наличие iproute2 и таблицу маршрутизации нужного семейства.

Очистка exit-node использует все сохранённые WAN данного TUN, включая прежние uplink,
без поиска текущего маршрута для семейства с известными целями. При ошибке очистки
его targets сохраняются для повторной попытки. Если ownership пуст, discovery остаётся
best-effort; это не восстанавливает владение, потерянное при crash.

Срок относится к отдельному read-only запросу. Последовательный fallback и ожидание
процесса могут занять больше времени; firewall-команды gateway/kill-switch ещё требуют
отдельных ограничений. Новых INI-параметров нет.
[Проверки и границы](../reports/AUDIT-Q25-GATEWAY-WAN.md).

### 6.39 Linux roaming: неизвестное состояние после ошибки маршрута

Неуспешная `ip route add/replace/del` могла изменить маршрут до возврата ошибки.
Для неудавшегося add/replace Qeli проверяет destination после отката предыдущих шагов:
обычный отказ с сохранением прежнего пути требует подтверждения прежнего состояния.
Удаление и восстановление оцениваются по последующему снимку независимо от статуса
команды: подтверждённое отсутствие завершает retirement, точный прежний снимок —
восстановление (см. 6.42). Неподтверждённое прежнее состояние при отказе транзакции
передаётся контроллером как `PlatformStateUnknown` и требует остановки текущего
поколения соединения.

Сообщения `failed route mutation ... did not preserve the previous route` и
`ambiguous route snapshot` объясняют причину неудачной проверки.
Проверьте затронутый IPv4/IPv6 destination и предшествующие ошибки команд.
Несколько непустых строк снимка отклоняются; восстановление multipath-снимка не реализовано.

Это не гарантия удаления всех неопределённых маршрутов. Неуспешный add не доказывает
владение; такой маршрут может остаться для проверки. Неопределённые roaming-операции
сохраняются отдельно как pending reservations без права удаления (см. 6.43).
Постоянный crash recovery и сроки команд остаются отдельной работой. Новых INI-полей нет.
[Доказательства и ограничения](../reports/AUDIT-Q25-ROUTE-OUTCOME.md).

### 6.40 Linux: изменившееся владение маршрутом и повтор очистки

При очистке физического маршрута Qeli сравнивает записанные gateway/device и остальные
переданные параметры с текущим exact route. `owned route changed; preserving replacement`
означает, что наблюдаемая замена оставлена на месте, а устаревшая запись журнала удалена.
Уже отсутствующий маршрут также не требует delete.

`command succeeded but route remains` означает, что совпадающий маршрут остался после
команды. Недоступный, повреждённый или неоднозначный снимок тоже даёт ошибку очистки;
спецификация сохраняется для retry. При потере результата команды очистка может завершиться,
если последующий запрос подтверждает отсутствие. Успешная смена пути обновляет параметры
будущей очистки.

Журнал остаётся в памяти, но записи уже разделены по владельцу подключения (см. 6.41).
Recovery после crash, атомарная защита от изменений других процессов и сроки команд
остаются открытыми. Новых INI-параметров нет.
[Проверки селекторов](../reports/AUDIT-Q25-ROUTE-OWNERSHIP.md).

### 6.41 Linux: владелец маршрутов завершён или ещё занят

`route owner is stopped/has expired` означает, что старый prepare/commit больше не
может менять маршруты после начала очистки или освобождения владельца. Новый план
должен получить собственный owner; повтор номера generation не возобновляет прежний.

`still live or has pending cleanup` означает, что имя TUN занято живым guard либо
осталась неподтверждённая очистка. Живой guard повторяет только свою очистку. Если
последний guard уже освобождён с остатками, новое подключение может освободить
резервирование только после read-only подтверждения их отсутствия и при выполнении
условий 6.43. Автоматического присвоения/удаления неподтверждённого маршрута нет.
Сначала выясните причину исходной ошибки и состояние соответствующих маршрутов;
перезапуск процесса сам по себе не доказывает их удаление.

`belongs to another Qeli owner` означает конфликт carrier/exclude/blackhole с другим
подключением этого процесса. Совместное владение таким маршрутом не поддерживается.
`dev_attach=true` оставляет маршруты внешнему управляющему: Linux не объявляет
`ROAMING_PATH`, `roaming=auto` использует reconnect, `required` недоступен.
[Регрессии и границы](../reports/AUDIT-Q25-ROUTE-SCOPE.md).

### 6.42 Linux roaming: результат удаления или восстановления не подтверждён

`carrier route ... remains after retirement` означает, что после delete маршрут остался,
даже если команда вернула успех или «уже отсутствует». `changed before retirement`
означает, что прежний снимок изменился ещё до удаления: Qeli не удаляет наблюдаемую
замену и не воссоздаёт маршрут, исчезнувший до этого шага.

`could not restore carrier route ... snapshot differs` означает, что повторный снимок
не совпал с сохранённым; `could not verify restored carrier route` — что проверку
после восстановления выполнить не удалось. Успешный exit status сам по себе недостаточен.
Если прежний снимок подтверждён после потерянного результата, восстановление считается
завершённым. Аналогично подтверждённое отсутствие завершает удаление.

При отказе Qeli проверяет откат всех выполненных шагов. Если состояние прежнего пути
не доказано, требуется остановка поколения, а не продолжение на предположительно
восстановленном пути. Проверьте указанный destination, текущий снимок и предыдущие ошибки;
учёт pending и условия освобождения отсутствующих orphan описаны в 6.43.
Полного семантического сравнения всех атрибутов и атомарной защиты от внешних изменений
нет; новых INI-параметров также нет.
[Регрессии и границы](../reports/AUDIT-Q25-ROUTE-POSTCONDITIONS.md).

### 6.43 Linux: pending reservation после неопределённого результата

`unresolved route mutation; destination remains reserved without delete authority`
означает, что результат команды не позволил доказать владение, а destination всё ещё
присутствует. Qeli сохраняет запись операции и ошибку cleanup, но не удаляет этот
маршрут на основании pending. Совпадение его параметров с планом не доказывает авторство.
`could not verify pending route` означает, что не удалось получить корректный снимок.

Неизвестный результат commit сразу закрывает новые route-операции этого owner,
включая gateway refresh. Живой guard может повторять cleanup; pending снимается
только после подтверждённого отсутствия destination. После успешной очистки старый
owner остаётся закрытым, а новый возможен после освобождения последнего guard.

Если guard уже освобождён, новое подключение того же процесса выполняет read-only
проверку orphan только после ранее подтверждённого результата IPv4/IPv6 interface flush.
Подтверждается пустое состояние, а не только успешный статус команды (см. 6.44). Требуются
отсутствие всех сохранённых destinations и пустые маршруты интерфейса в обеих семьях.
`orphan route reservation ... is still present` и
`could not confirm empty interface routes` означают, что освобождение запрещено.
Иная ошибка запроса также сохраняет reservation; маршруты не перезаписываются.

Проверьте указанные адреса, интерфейс и исходную ошибку. Запись pending не разрешает удаление маршрута.
Независимая очистка принадлежащего Qeli интерфейса описана в 6.46. Без предшествующей очистки, после неподтверждённого
interface flush или при живом guard автоматическое освобождение недоступно. Этот механизм
работает внутри процесса: он не восстанавливает journal после crash/restart. Pending также
используется при initial setup carrier/exclude/blackhole и TUN/TAP с возможным остатком (см. 6.44–6.46).
INI-параметры не добавлены. [Отчёт и ограничения](../reports/AUDIT-Q25-ROUTE-PENDING.md).

### 6.44 Linux: проверка initial setup и очистки маршрутов интерфейса

Перед установкой carrier/exclude/blackhole и TUN/TAP Qeli проверяет exact-снимок destination.
Совпадающий существующий маршрут используется без присвоения. `initial route conflicts
with an existing route` означает конфликт до записи. Ошибка чтения тоже запрещает add.

`route is absent after initial add` означает, что команда не создала проверяемый маршрут.
`initial add outcome is not proven; destination remains reserved` и
`could not verify initial route` означают возможный неподтверждённый остаток.
Setup завершается ошибкой; pending не даёт права удалить маршрут. Даже `File exists`
после исходного отсутствия не делает внезапно появившийся маршрут безопасно заимствуемым.

После flush обеих IP-family проверяется отсутствие маршрутов интерфейса.
`interface routes remain` означает реальный остаток независимо от статуса команды.
`could not confirm empty interface routes` означает, что пустое состояние не доказано.
Потерянный результат flush допускает успех, если отсутствие подтверждено.
Отказ одной семьи не пропускает очистку другой.

Если route-query вернул отрицательный статус из-за уже удалённого TUN, Qeli подтверждает
отсутствие точного имени через отдельный `ip -o link show`. Одной строки `Cannot find device`
недостаточно. `invalid link snapshot`, ошибка выполнения, не-UTF-8 вывод или найденный
интерфейс сохраняют ошибку очистки. При I/O-ошибке route-query требуется повтор проверки.

Сопоставьте адрес, интерфейс и предыдущие ошибки в журнале; живой guard может повторить
cleanup. Это не подтверждение crash recovery, произвольных policy tables/VRF или завершения
TUN workers. Новых INI-параметров нет.
[Проверки и границы](../reports/AUDIT-Q25-SETUP-FLUSH.md).

### 6.45 Linux: таймаут route/firewall и неизвестный IPv4-путь

Команды `ip` для маршрутизации клиента и команды `iptables/ip6tables` kill-switch/gateway
используют общий предел 15 секунд на вызов и по 16 МиБ stdout/stderr. При превышении
возвращается ошибка, частичный вывод не используется. Это не 15 секунд на весь setup,
cleanup или reconnect: последующие проверки и ожидание выхода процесса требуют времени.

Если команда могла изменить сеть, таймаут сам по себе ничего не откатывает.
Неподтверждённый add остаётся pending без права удаления по этой записи.
После flush проверяется реальное отсутствие маршрутов; ошибка этого запроса требует
повтора. Недоступная firewall-цепочка тоже не считается отсутствующей.
Проверьте предыдущие ошибки, работоспособность iproute2/iptables и состояние конкретного
интерфейса; не объявляйте cleanup успешным только по завершению дочернего процесса.

`IPv4 egress is present or could not be ruled out` означает, что IPv4 firewall не
установлен, а отсутствие IPv4 default route не доказано. Ошибка запуска, отрицательный
статус, таймаут и переполнение вывода требуют защиты. Только успешный пустой список
позволяет пропустить IPv4-часть без явного `allow_ipv4_leak = true`. Этот параметр
разрешает утечку IPv4; он не восстанавливает firewall.

Новые ключи конфигурации не добавлены. Linux runtime, полный gateway rollback и общий
deadline транзакции остаются отдельными проверками.
[Отчёт и evidence](../reports/AUDIT-Q25-CLIENT-COMMANDS.md).

### 6.46 Linux: TUN/TAP-маршрут не подтверждён или повреждён список route_local

Установка connected pool, full-tunnel capture, pushed/include/DNS routes и override
локальных сетей проходит через общий installer. `initial route conflicts with an
existing route` означает, что точный prefix уже имеет другой интерфейс, gateway или
метрику. Прямой L3 TUN не заимствует маршрут с gateway. Успешного exit code недостаточно:
после add проверяется фактический снимок; при неизвестном результате setup завершается
ошибкой и owner больше не принимает route-операции.

Проверьте текущую запись и ожидаемый план. Не удаляйте конфликтующий маршрут только
по совпадению prefix: он может принадлежать другой настройке или другому владельцу Qeli.
Нулевая метрика IPv4 может не отображаться; IPv6 metric 0 имеет эффективное значение
1024. Qeli учитывает эти формы и проверяет остальные явно запрошенные метрики.

При cleanup pending не разрешает отдельный `route del`. Принадлежащий Qeli TUN/TAP
независимо очищается по интерфейсу; это охватывает и маршруты на нём, которые ранее
были заимствованы. После flush Qeli проверяет отсутствие pending destinations.
Удалённый таким образом остаток снимает reservation; маршрут на другом интерфейсе
или недоступный snapshot сохраняет ошибку. Это правило не разрешает присвоить интерфейс,
открытый через `dev_attach`.

`invalid connected IPv4 address snapshot for route_local` означает повреждённый ответ
`ip -4 -o address show up scope global`. Qeli отказывается считать его пустым списком
и останавливает setup до записи маршрутов. Проверьте доступность и вывод iproute2.
Корректный пустой ответ допустим; адреса своего TUN и сети вне RFC1918 не создают overrides.

Пользовательские конфиги остаются INI, новых ключей нет.
[Проверки и ограничения](../reports/AUDIT-Q25-TUNNEL-ROUTES.md).

---

## 7. Справочник

### 7.1 Статусы туннеля (клиенты)
`Disconnected` (серый) · `Connecting` (жёлтый, включая реконнект и «TUN ещё не
поднят») · `Connected` (зелёный, **только после поднятия TUN**) · `Error` (красный,
текст ошибки — из серверного `EXTRA_ERROR` / последней причины).

### 7.2 Цвета точки доступности (карточка профиля)
Reachable → зелёный (`N ms`) · Unreachable → красный (`offline`) · Checking →
жёлтый (`…`) · Unknown → **серый** (ещё не проверяли).

Android sentinel'ы `reach`: `-1` = недоступен (красный), `-2` = проверяется
(жёлтый), `≥0` = мс (зелёный), `null` = серый. Автоопрос — opt-in и по
умолчанию выключен; серый индикатор до ручной проверки не означает ошибку подключения.

### 7.3 Префиксы строк лога (клиенты)
`ERR:` — ошибка цикла · `WARN:` — предупреждение (не фатально) · `NOTE:` —
информационная заметка · `[SECURITY]` — крипто/MITM (**терминально**, без ретраев) ·
`  <- …` — вложенная причина исключения.

### 7.4 Уровни лога сервера
`info` (дефолт) — старт, `New TCP connection`, `AUTH OK`, все `AUTH FAIL/DENIED/
BLOCKED` (WARN). `debug` — причины отказа **до** аутентификации (`handshake timeout`,
`Client … disconnected: …`, `UDP handshake failed`, REALITY-bridging). `RUST_LOG`
переопределяет `[logging] level`.

---

## 8. Чеклисты команд

### 8.1 Сервер
```bash
# статус, версия, слушатели
systemctl is-active qeli
/usr/bin/qeli --version
ss -ltnp | grep qeli ; ss -lunp | grep qeli

# лог: только проблемы / реального времени
journalctl -u qeli --since '10 min ago' -p warning --no-pager
journalctl -u qeli -f

# включить debug и смотреть решающую строку
mkdir -p /etc/systemd/system/qeli.service.d
printf '[Service]\nEnvironment=RUST_LOG=debug\n' > /etc/systemd/system/qeli.service.d/zz-debug.conf
systemctl daemon-reload && systemctl restart qeli && journalctl -u qeli -f

# личность сервера (public key для пиннинга)
qeli show-identity --config /etc/qeli/server.conf

# брутфорс-локи
qeli list-blocked ; qeli unblock <ip>

# перевыпуск клиентской ссылки
qeli add-client <user> --password '<pw>' --link --host <ip>:<port> --link-profile <profile> --config /etc/qeli/server.conf

# REALITY-серт сервера снаружи (маскировка)
echo | openssl s_client -connect 127.0.0.1:443 -servername www.microsoft.com 2>/dev/null | openssl x509 -noout -subject

# PMTU-фикс (обе стороны)
iptables -t mangle -A PREROUTING -p tcp --dport <port> --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1240
iptables -t mangle -A OUTPUT     -p tcp --sport <port> --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1240
ip6tables -t mangle -A PREROUTING -p tcp --dport <port> --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1220
ip6tables -t mangle -A OUTPUT     -p tcp --sport <port> --tcp-flags SYN,RST SYN -j TCPMSS --set-mss 1220
```

### 8.2 Клиент Android
```bash
adb logcat -s VpnSvc VpnMain            # лог сервиса + активити
# сброс профилей/согласия VPN при залипании:
adb shell pm clear com.qeli
adb shell appops set com.qeli ACTIVATE_VPN allow   # если поддерживается
```

### 8.3 Десктоп (Windows/macOS)

- `Refusing untrusted service storage` / `refusing default DLL search` — отказ защиты,
  а не повод копировать DLL рядом с EXE. Начиная с 0.8.2 Windows проверяет владельца,
  все разрешения на запись и reparse points; сервисный профиль доступен только
  SYSTEM/Administrators. Старые небезопасные файлы автоматически не принимаются.
  Сначала остановите VPN/службу и завершите восстановление DNS/маршрутов/firewall.
  Затем сохраните резервную копию для диагностики, пересоздайте защищённое хранилище
  из elevated GUI и заново сохраните профиль из доверенного источника. Не удаляйте
  активные recovery journals и не «исправляйте» только ACL поверх непроверенных файлов.
- `Tunnel cleanup remains incomplete` означает, что очистка ещё не закончена:
  служба сохраняет Error вместо Disconnected. Повторите остановку и проверьте журнал.
- `TLS traffic key budget exhausted; reconnect required` — защитное завершение
  REALITY-TLS после 2^24 записей или 64 GiB ciphertext на одном ключе в одном направлении.
  Клиент использует обычную политику переподключения; KeyUpdate пока не реализован.

- Вкладка **Log / Журнал** → **Copy log** — прислать при разборе.
- Windows требует **администратора** (манифест `requireAdministrator`); macOS —
  **root** (`sudo`) или включённый launchd-демон.
- Kill-switch остался после краша? Windows:
  `Remove-NetFirewallRule -Group qeli_ks; Set-NetFirewallProfile -All -DefaultOutboundAction Allow`;
  macOS: перезапустить/`pfctl -d` (сообщение «Found a stale kill-switch…» само чинит при следующем старте).
- `kill-switch is owned by another live Qeli process` означает, что второй Windows-туннель
  пытается захватить общесистемное состояние firewall. Сначала остановите другой клиент/сервис.
- `[SECURITY] kill-switch disengage failed; egress remains blocked` — fail-closed режим:
  recovery state и владение сохранены, чтобы тот же процесс (или следующий запуск) повторил
  полное восстановление трёх профилей firewall. Не удаляйте recovery state вручную.

---

*Документ основан на текущем коде (`qeli/src/**`, `qeli-shared`, `qeli-win`,
`qeli-mac`, `qeli-android`) на ветке `dev`. Строки ошибок сверены с исходниками;
если поведение расходится — доверяйте коду и обновите этот файл.*

### Ошибки очистки gateway или порядка kill-switch

`gateway cleanup failed` сохраняет записи TUN/семейства для повтора в работающем
процессе. Разберите указанную ошибку firewall; успешная очистка другого профиля не
подтверждает чистоту этого. Ошибка восстановления sysctl сообщается вместе с ошибками
правил. Не снимайте сохранённый kill-switch только потому, что forwarding восстановлен.

`cannot inspect router kill-switch protection` или ошибка порядка jump запрещает
вставку либо повторное использование permit без проверенной защиты. Проверьте доступ
к iptables/ip6tables и фактический порядок FORWARD. После краша процесса журнал правил
в памяти теряется; перед ручным восстановлением разберите оставшиеся `qeli-gw-nat`
для нужного интерфейса/подсети.
[Доказательства и ограничения](../reports/AUDIT-Q25-GATEWAY-ROLLBACK.md).

### NAT exit-node или конфликт kill-switch

На общем WAN проверяйте точные комментарии NAT `qeli-exit-node:<tun>` в каждой семье.
Старый MASQUERADE без суффикса сохраняется при очистке; не удаляйте его, пока на него
может полагаться старый exit-процесс. Отсутствие запомненного WAN запрещает автоматическое
удаление firewall, но не доказывает отсутствие остатков предыдущего процесса.

`kill-switch conflict` означает, что до установки найдена чужая per-TUN или старая
общая Qeli chain. Штатно остановите её живого владельца либо подтвердите, что цепочка
осталась после сбоя, и восстановите именно её по процедуре
[Getting started](GETTING-STARTED.md). Не очищайте всю filter-таблицу.
Неполный/нечитаемый снимок также блокирует допуск. Разрешения утечек не устраняют
конфликт политик. [Доказательства и ограничения](../reports/AUDIT-Q25-EXIT-OWNERSHIP.md).

### Linux: занято владение kill-switch или неизвестно состояние IPv6

`cannot exclusively own this network namespace` означает, что резервирование
kill-switch не удалось. При `Address already in use` остановите другой защищённый
клиент в этом network namespace, включая экземпляр с тем же `dev`. При другой
ошибке проверьте ограничения AF_UNIX/sandbox и ресурсы процесса. До этого отказа
новый клиент не восстанавливает DNS и не изменяет firewall. Leak overrides его не обходят.

Lease освобождается при закрытии клиента, но сам по себе не удаляет firewall после
краша. Сначала установите, кому принадлежат оставшиеся цепочки; используйте точечное
восстановление из [GETTING-STARTED.md](GETTING-STARTED.md). У lease нет lock-файла,
который нужно удалять. Старые версии и ручные изменения firewall требуют отдельной
координации при обновлении.

`global IPv6 is present or could not be ruled out` после ошибки установки IPv6-части
означает, что безопасно пропустить её не удалось. Проверьте `ip6tables` и успешность
`ip -6 address show scope global`. Пустой ответ при ошибочном exit status не достаточен.
`allow_ipv6_leak = true` — явное принятие утечки. Если указан сбой rollback IPv4,
не считайте оставшийся firewall очищенным.
[Проверки и ограничения](../reports/AUDIT-Q25-KILL-SWITCH-LIFETIME.md).

### Linux: имя TUN уже зарезервировано

`cannot reserve TUN` при `Address already in use` означает, что другой новый Linux-клиент
уже использует это `dev` в том же network namespace, даже если сейчас переподключается
и интерфейса временно нет. Остановите владельца или выберите другое имя. Правило действует
и при `kill_switch = false`, и при `dev_attach = true`: общий multi-queue интерфейс
не предназначен для двух независимых Qeli-сессий. AF_UNIX/sandbox/resource ошибки тоже
запрещают startup; отказ происходит до восстановления DNS и изменения сети.
При конфликте kill-switch временная резервация TUN неудачного запуска освобождается.

Если IPv6 выключен через `ipv6.disable=1`, доступное точное значение `1` в
`/sys/module/ipv6/parameters/disable` позволяет не обращаться к IPv6 firewall.
Если файл недоступен или модуль просто не загружен, Qeli не делает вывод «IPv6 выключен».
Не подменяйте sysfs и не отключайте IPv6 ради обхода проверки: восстановите доступ
к диагностике и разберите исходную ошибку.
[Отчёт и ограничения](../reports/AUDIT-Q25-CLIENT-NAMESPACE.md).
