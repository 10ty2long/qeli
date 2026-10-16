# qeli 0.8.2 release notes draft

Not released. These notes currently describe the Linux client TUN queue change; the complete
itemised release history is in
[CHANGELOG.md](https://github.com/litvinovtd/qeli/blob/v0.8.2/CHANGELOG.md).

Не выпущен. Здесь пока описано изменение клиентской TUN-очереди Linux; полный перечень
изменений версии приведён в changelog по ссылке выше.

## Русский

### Размер клиентской TUN очереди Linux

Добавлен локальный параметр `tun.tx_queue_len` в секции `[qeli]` клиентского INI:

```ini
[qeli]
tun.tx_queue_len = 1000
```

Он задаёт размер очереди пакетов ядро → Qeli для собственного TUN/TAP Linux-клиента.
По умолчанию используется `1000` пакетов, допустимы целые значения `1..32767`.
Автоматической регулировки нет; для возврата к прежнему лабораторному размеру можно задать `500`.

Параметр появился после диагностики низкой скорости upload Reality-TLS: клиентская очередь
переполнялась ещё до H2/TLS, вызывая потери и повторные передачи внутреннего TCP. В проверке
на одном и том же бинарнике A/B/A `500 → 1000 → 500` (IPv4, по три повтора) увеличение
очереди повысило upload P1/P4 и уменьшило потери и повторные передачи в P4; возврат к `500`
вернул прежний уровень скорости. Более вместительная очередь
помогает переживать всплески, но не устраняет все потери и не гарантирует ускорения на любом
профиле или устройстве. Компромисс — возможный рост задержки и потребления памяти.
Это не изменение H2 flow control и не исправление доказанного ограничения H2.

Нужен обновлённый клиент с поддержкой параметра: сохраните INI и остановите/запустите клиент
(в панели — Disconnect → Connect). Ключ не передаётся через server push, ссылку или QR;
серверный одноимённый параметр остаётся независимым. При `dev_attach=true` ключ следует
полностью опустить: очередью управляет внешний владелец интерфейса. Другие ОС сохраняют ключ
без применения. Старые бинарники, включая прежние сборки с номером 0.8.2, не получают
это поведение от одного обновления конфигурации.

Подробности и проверка фактического размера очереди:
[CONFIG](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/manuals/CONFIG.md#client-tun-tx-queue).
Результаты разных размеров очереди, IPv6 и трёх транспортов:
[лабораторный отчёт и ограничения](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/TUN-QUEUE-SIZE-2026-10-16.md).

## English

### Linux client TUN queue size

The client INI now accepts the local `tun.tx_queue_len` setting in `[qeli]`, as shown above.
It controls the kernel-to-Qeli packet queue for a Linux client-owned TUN/TAP interface.
The default is `1000` packets; valid integer values are `1..32767`. There is no automatic
adjustment. Set `500` explicitly to restore the previous laboratory queue size.

The setting addresses burst-related upload losses identified during Reality-TLS diagnostics:
the client queue overflowed before H2/TLS, causing packet drops and inner TCP retransmissions.
An A/B/A `500 → 1000 → 500` test with the same unchanged binary (IPv4, three repetitions per
stage) showed higher P1/P4 upload throughput and fewer P4 drops and retransmissions at `1000`;
returning to `500` restored the previous throughput level.
A larger queue helps absorb bursts, but does not eliminate all losses or guarantee a speedup
on every profile or device, and may increase latency and memory use. This is not an H2
flow-control change or a fix for a demonstrated H2 bottleneck.

Use an updated client containing this feature, save the INI, then stop/start the client
(Disconnect → Connect in the panel). The setting is not carried by server push, share links
or QR codes; the same-named server setting remains independent. Omit the key entirely with
`dev_attach=true`, where the external interface owner controls the queue. Other operating
systems preserve the key without applying it. Older binaries, including earlier builds
labelled 0.8.2, do not gain this behaviour through configuration changes alone.

Configuration details and verification:
[CONFIG](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/manuals/CONFIG.md#client-tun-tx-queue).
Queue-size, IPv6 and three-transport results:
[lab report and limitations](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/TUN-QUEUE-SIZE-2026-10-16.md).
