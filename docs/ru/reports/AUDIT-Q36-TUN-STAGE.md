# Q36 — реальная диагностика границ TUN и влияние наблюдателя

<!-- normative-sync: audit-q36-tun-stage-v1 -->

13 октября 2026 года. На .10 выполнены шесть отдельных 8-секундных duplex
TCP raw фаз: downlink 100 Мбит/с и unlimited, контроль до наблюдения,
наблюдение и контроль после. Один immutable factory CLI `52511ceb`, одна
сессия и неизменный INI; новое ядро не собиралось. Наблюдатель — strace 6.13
на двух точных принадлежащих fixture TID по 0,2 с, без BPF и follow-forks.
[Фактический результат](../../../release/certification/evidence/q36-tun-stage-20261013.json).

Все шесть фаз, парные PCAP и архивирование завершились. Полная файловая
инвентаризация, binary/helper/config SHA и exact-byte/multiplicity сопоставление
прошли. Оба tracer отсоединены; endpoint PID/start ticks, production, полный
host snapshot и исходные defaults сохранены. Собственные root/netns/archive
удалены. Первый запуск отказал до измерений: клиентский INI был передан в
серверный `check-config`; добавлен штатный `--client`, исходный FAIL сохранён.

## Сопоставимые окна по 150 мс

Медианы ниже используют только ACK с server capture внутри одного окна и
однозначным точным соответствием в client capture. Дополнительные syscall
кандидаты вне capture window не входят в сравнение.

| Downlink | Контроль до, мс | Наблюдение, мс | Контроль после, мс | Наблюдение / максимум контролей |
|---|---:|---:|---:|---:|
| 100 Мбит/с | 0,401259 | 13,325437 | 1,155161 | 11,5356 |
| Unlimited | 21,403810 | 123,972554 | 15,420216 | 5,7921 |

Окна с наблюдением существенно отличаются от обоих контролей. Эту серию
нельзя использовать для объяснения исходной задержки или production capacity.
Это ограничение наблюдения, не установленная причина исходной асимметрии.
Восьмисекундные whole-phase throughput значения не заменяют проверку
искажения в коротком traced window.

У серверного профиля действительно два reader: q0/q1 при стандартном
`tun.queues=0` (auto). Выбран q0; его trace содержит только незавершённое
чтение при detach, без успешных read. На клиенте получены успешные записи
ACK, но нет ни одного однозначного соответствия всех четырёх границ.
Назначение конкретного ACK очереди q1 напрямую не измерено. Отсутствие read
в этом trace не объявляется потерей. Смена выбранной очереди не устраняет
выявленное различие traced и контрольных окон; эту серию не повторяем для
вывода исходной стадии задержки.

Clock discontinuity не обнаружен, но абсолютная погрешность runtime clock
не ограничена. Успешный TUN write не означает доставку приложению; пропущенные
промежуточные queues/crypto/socket не измерены. Причина Q36 остаётся
`NOT_ESTABLISHED`. Далее — квалификация current steady data plane без
наблюдателя; любое дальнейшее stage-наблюдение сначала требует проверки
overhead и учёта всех TUN-очередей. Полный Q36 остаётся IN_PROGRESS.
`NO_PUSH_DEPLOY_RELEASE`.

## 14 октября: текущий CLI, штатные события чтения TUN

Execution source 66cf9a2d5d40; текущий CLI SHA `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`, actual build e0ce217b630d9dfd7db5f59936bb41e6d68eb69d; product tree `8c50631e7c253ab6cacd0056098ecad83f393487` не изменился. Две fresh sessions / четыре 8-секундных окна: uplink unlimited, отдельный downlink 100 Мбит/с либо unlimited, legacy one-carrier на обоих концах. Одна session с наблюдением, одна контрольная; это не статистическая серия из трёх пар и не замена предыдущей матрицы.

Наблюдатель использовал только stock sys_enter_read/sys_exit_read, net_dev_start_xmit для небольших пакетов собственных qbench0 и consume_skb в tun_do_read. Оба server reader TID и их реальные TUN fd проверены по namespace/inode, /proc и executable SHA; создан только частный trace instance. BPF/kprobe/strace не использовались. Для двух коротких участков внутри нагрузки получены 168599 и 562547 записей, trace loss=0; время наблюдения находится внутри фактических интервалов обоих iperf children. Проверены роли/параметры нагрузки, PCAP SHA/nanotimestamps, clock continuity и отсутствие tcpdump drops.

Без stock trace uplink составил 254,24 Мбит/с при встречных 100 Мбит/с и 126,61 Мбит/с при unlimited. Для одинаковых ACK-заголовков end-to-end p50 вырос с 0,958656 до 14,155557 мс. В соответствующей наблюдаемой session — 316,40/99,75 Мбит/с и ACK p50 1,011884/22,166426 мс. Эти четыре значения относятся ко всей фазе; kernel trace короче. Одного контроля недостаточно для разделения observer overhead и межсессионной вариативности; наблюдение нельзя объявлять нейтральным.

По hashed SKB identity и длине сопоставлены только небольшие пакеты с исходным событием в заранее проверенных server threads. Для 52 байт: 5079 пар при 100 Мбит/с, 998 при unlimited; TUN queue p50 0,010/0,011 мс, p99 2,515/3,718 мс. Неполное покрытие явно сохранено: 12944/3695 успешных небольших read без admitted server xmit. Это не потеря пакетов. События ядра не содержат IP/TCP-заголовка: длина 52 байта не доказывает связь с конкретным ACK из PCAP. Между read на активном reader p99=0,021/0,008 мс; этот интервал объединяет pool acquisition, handoff и scheduling. Длительное blocking read может означать отсутствие пакета, а не задержку доставки.

Первая попытка наблюдения сохранена отдельно как невалидная для stage timing: пятый сборщик ожидал четыре занятых controller threads, включившись после завершения нагрузки. Исправлен только внешний диагностический driver (`max_workers=5`), добавлены overlap/nonempty gates; код продукта не менялся. Оба пакета завершили namespace/process cleanup, SHA/inventory archive и удаление собственных root/archive; global tracing controls, services и staged CLI сохранены. Новая серия сужает измеренную границу, но не устанавливает исходную стадию F023: неполное соответствие ACK и заметное изменение observer-present результата остаются ограничениями. Не повторять тяжёлый четырёхсобытийный observer для утверждения исходной причины. Q36 IN_PROGRESS; NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-stock-read-pilot-20261014.json`.

## 14 октября: наблюдение только небольших TUN-пакетов

Execution source 553db95a5ddf; тот же текущий CLI 6f617bdcab5a и неизменившийся product tree. Новые две sessions / четыре 8-секундных окна legacy one-carrier. Вместо всех read/consume записывались только stock net_dev_start_xmit для собственных qbench0 и skb_copy_datagram_iovec длиной 1..128 байт на двух проверенных server reader TID. Private buffer 8192 КиБ на CPU, без BPF/kprobe/strace и изменения продукта.

При downlink 100 Мбит/с / unlimited получены 34346/35357 событий, trace loss=0. Это соответственно в 4,91/15,91 раза меньше записей прежнего четырёхсобытийного наблюдения; разные реальные сессии, не оценка ускорения. Для всех 14558/6090 small-copy событий найден последний предшествующий stock xmit с тем же hashed SKB и длиной. Прежние неизвестные origin contexts оказались в основном новыми потоками iperf3; их PID/comm и частое переиспользование pointer сохранены. Это соответствие не содержит IP/TCP-заголовка и не доказывает идентичность конкретному ACK. Copy event срабатывает до копирования в user buffer, не после завершения read.

Для 52 байт: 14475/6089 пар; driver-entry → copy-entry p50 0,010/0,017 мс, p99 1,210/7,177 мс, max 4,801/8,607 мс. В тех же коротких временных областях matched ACK p50 составил 0,922168/18,530356 мс; контроль с теми же относительными границами — 0,777087/11,157324 мс. Совпадение числа 6089 copy и matched ACK в unlimited-окне само по себе не устанавливает per-packet identity. Временные границы PCAP используют estimate offset из capture-ready; абсолютная погрешность не ограничена. Unmatched destination вне выбранной source-window — ожидаемое следствие сравнения с полной client capture, не счётчик потерь.

Whole-phase observed/control uplink: 346,90/299,33 Мбит/с при 100 Мбит/с и 99,14/175,02 при unlimited. Короткая ACK-медиана observed/control отличается на +18,67/+66,08%; одного контроля недостаточно, чтобы разделить observer overhead и межсессионную вариативность. Меньший поток записей не сделал наблюдение доказанно нейтральным. Исходная причина F023 не установлена; серию нельзя использовать для изменения очередей или production defaults. Не повторять ещё один вариант stock trace только ради формального закрытия: следующий этап должен проверять конкретную причинную гипотезу с сопоставимыми контролями.

Workload/ACK/trace qualification, actual overlap, SHA/inventory archive и полная очистка own namespace/process/root/archive/trace instance прошли; host/firewall/services/staged CLI и global trace controls сохранены. Обе kernel boundaries измерены только в observer-present пилоте, остальные очереди/crypto/socket/client delivery не измерены. Q36 IN_PROGRESS;33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

Source context: [Linux6.12 tun_put_user](https://raw.githubusercontent.com/torvalds/linux/refs/tags/v6.12/drivers/net/tun.c), [skb_copy_datagram_iter](https://raw.githubusercontent.com/torvalds/linux/v6.12/net/core/datagram.c), [stock event fields](https://raw.githubusercontent.com/torvalds/linux/v6.12/include/trace/events/skb.h). Это первичные upstream исходники, не доказательство byte equivalence текущего Debian module.

Evidence: `release/certification/evidence/q36-stock-small-copy-pilot-20261014.json`.

## 14 октября: причинные проверки TCP socket buffers без kernel trace

Execution source 989c99d60554, тот же CLI 6f617bdcab5a, unchanged product tree 8c50631e7c253ab6cacd0056098ecad83f393487. Две отдельные серии по 6sessions/12windows: tcp-plain-raw, legacy one-carrier, uplink unlimited плюс отдельный downlink 100M/unlimited, P1/8сек. Порядок условий и скоростей изменён во втором повторе. Kernel trace/BPF/kprobe/strace отсутствуют; ACK-only tcpdump, process sampler и ss одинаковы в обеих группах. Это не повтор всей 144-window матрицы и не production capacity.

Первая гипотеза: ограничение send buffer может убрать накопление ACK после серверного TUN. Сравнены **явный send=0 (kernel-auto)** и send=16384 при прежнем recv=262144. Внутренний case label `default` означает именно явный0, а не отсутствующий INI ключ: штатное значение Qeli — 262144. Эта серия не подменяет исторический stock-default baseline. Фактический small tb=32768 подтверждён во всех steady snapshots, контрольный tb больше32768. Клиентские INI templates идентичны кроме per-session pin; серверный template отличается ровно одним ключом.

| Повтор | Downlink | Uplink контроль / изменение, Мбит/с | ACK p50 контроль / изменение, мс |
|---|---|---|---|
| 1 | 100M | 252.62 / 265.06 | 0.818935 / 1.095095 |
| 1 | unlimited | 124.35 / 116.68 | 12.078993 / 30.485677 |
| 2 | 100M | 215.45 / 277.33 | 1.022938 / 0.999153 |
| 2 | unlimited | 207.75 / 144.98 | 9.931232 / 31.834243 |
| 3 | 100M | 295.20 / 252.41 | 0.860319 / 1.097533 |
| 3 | unlimited | 167.54 / 116.96 | 9.976525 / 37.260997 |

При unlimited tiny send ухудшил ACK p50 во всех3парах: 2.524–3.735×, paired median3.205×; uplink ratio0.698–0.938, median0.698. Это отвергает tiny-SO_SNDBUF как универсальный обход в проверенной топологии. Не доказывает отсутствия задержек в kernel send queue: уменьшение буфера может переносить backpressure в userspace. Defaults не меняются.

Вторая гипотеза: штатный фиксированный recv=262144 ограничивает окно приёма/рост TCP receive memory при duplex нагрузке. Сравнены **stock recv=262144** и recv=0; send=262144 явно одинаков в обеих группах. Stock rb=524288 и held tb=524288 подтверждены; auto rb составил 1272344–2838678 байт и превысил fixed capacity в каждом окне. Source parser baseline содержит вложенный tcp object и применяет field defaults; derive Default отдельно не является sparse-INI baseline.

| Повтор | Downlink | Uplink контроль / изменение, Мбит/с | ACK p50 контроль / изменение, мс |
|---|---|---|---|
| 1 | 100M | 317.60 / 445.26 | 0.841706 / 0.864485 |
| 1 | unlimited | 130.02 / 153.65 | 19.583172 / 15.429859 |
| 2 | 100M | 245.72 / 427.21 | 0.868478 / 0.884169 |
| 2 | unlimited | 150.18 / 117.20 | 11.769458 / 18.860308 |
| 3 | 100M | 306.22 / 480.69 | 1.184153 / 0.796215 |
| 3 | unlimited | 149.19 / 142.70 | 13.145937 / 14.609256 |

При встречных100M auto/fixed uplink вырос во всех3парах: 1.402–1.739×, median1.570×. Steady socket-drop growth fixed/auto по повторам=[(199, 0), (288, 0), (106, 0)]; это изменение skmem.d на наблюдаемом steady участке, не потери всего туннеля/всей фазы. ACK p50 ratio min/median/max=0.672/1.018/1.027. Подтверждено отдельное ограничение uplink фиксированной receive-capacity при этой нагрузке; оно не устранило исходную unlimited ACK-асимметрию.

Для unlimited paired uplink auto/fixed min/median/max=0.780/0.957/1.182; ACK p50 auto/fixed=0.788/1.111/1.602. Все исходные значения, unmatched header counts, клиентские drops и actual socket memory/steady receive-drop growth сохранены в evidence. Результат одного mode/shared2-vCPU стенда не устанавливает каждую внутреннюю стадию, не подтверждает исправление старого CLI и не обосновывает смену defaults во всех клиентах/транспортах.

Ненулевые SO_SNDBUF/SO_RCVBUF закрепляют соответствующий размер, отключая kernel autotuning для данного направления; 0 в серверных INI ключах оставляет socket option нетронутым. Исправлено описание в RU/EN CONFIG, отличающее отсутствие ключа (262144) от явного0 и серверные TCP ключи от клиентских UDP buffer keys. [Linux6.12 tcp_rmem/tcp_wmem](https://www.kernel.org/doc/html/v6.12/networking/ip-sysctl.html) — первичный источник semantics. Кода/артефактов продукта не меняли.

Обе серии прошли workload settings/roles, identical-header PCAP SHA/timestamps/zero capture loss, topology/source, full host/firewall/services/staged preservation и owned root/archive cleanup. Сохранены оба SHA/inventory backup; больших compiler caches нет. Q36 IN_PROGRESS;33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-server-sndbuf-causal-20261014.json`, `release/certification/evidence/q36-server-rcvbuf-causal-20261014.json`.

## 14 октября: приоритет клиентского TUN writer

Execution source e07c2597d446, тот же текущий CLI6f617bdcab5a/product tree8c50631e7c253ab6cacd0056098ecad83f393487. Новые6sessions/12windows: три пары обычного nice0 и nice−10 только client qeli-tun-writer; rawTCP legacy one-carrier, P1/8сек, два отдельных встречных iperf потока при downlink100M/unlimited. Во втором повторе порядок условий и скоростей обращён. Config templates одинаковы кроме per-session public pin; server requested send/recv262144, actual tb/rb524288 во всех steady snapshots. Новых socket/queue/default изменений нет.

До установки и до/после каждого окна проверены root/ns cookies, /proc exe SHA и путь, PID/TID starttime, уникальный writer comm, TUN fdinfo. Только writer получилnice−10; у остальных клиентских потоков nice0. Ordinary SCHED_OTHER и affinity0-1 сохранены; cgroup/autogroup unchanged внутри каждой session. Считываемый fair weight вырос именно на целевом TID; его scope — распределение CPU, не гарантия latency. [Linux per-thread nice semantics](https://man7.org/linux/man-pages/man2/setpriority.2.html).

| Повтор | Downlink | Uplink контроль / изменение, Мбит/с | ACK p50 контроль / изменение, мс |
|---|---|---|---|
| 1 | 100M | 237.77 / 210.45 | 0.968889 / 0.927177 |
| 1 | unlimited | 105.23 / 151.32 | 17.481170 / 15.946424 |
| 2 | 100M | 288.49 / 270.03 | 0.837279 / 0.871184 |
| 2 | unlimited | 164.18 / 139.85 | 13.142024 / 12.863850 |
| 3 | 100M | 296.41 / 347.55 | 0.867200 / 0.802712 |
| 3 | unlimited | 144.89 / 136.12 | 11.225342 / 14.199052 |

При unlimited paired uplink priority/default min/median/max=0.852/0.939/1.438; ACK p50=0.912/0.979/1.265. Устойчивого устранения задержки нет. Это отвергает повышение nice как достаточный обход в проверенной топологии, но не доказывает отсутствие очереди/ожидания в writer: меняется распределение CPU между его собственными соседними потоками, а не только время обработки конкретного ACK. Production priority/defaults не меняем.

Сохранены schedstat/tick deltas и флаг sched_schedstats. Их горизонт включает весь сбор данных, а не только8сек передачи; disabled scheduler statistics не означает zero runnable wait. Kernel trace/BPF/kprobe/strace не применялись. Passed workload/settings, same-header PCAP SHA/timestamps/capture loss0, topology/source, full host/firewall/services/staged preservation и SHA/inventory owned root/archive cleanup. После завершения own process вместе с изменённым TID удалён; чужие приоритеты не затрагивались.

Дальше — прямые диагностические timestamps на границах server TUN read → server wire handoff → client decrypt → client TUN write, с однозначным packet fingerprint, finite capture и одинаковой включённой/выключенной диагностикой. Одно совпадение размера пакета/счётчиков не заменяет packet identity. Read-only preflight подтвердил наличие Rust1.97 и local crate cache на .10; compiler не запускался, compiler/cache/core не менялись. Такой диагностический CLI должен иметь отдельную source/build qualification; он не является shipping CLI/native artifact. Q36 IN_PROGRESS;33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-client-writer-priority-20261014.json`.
