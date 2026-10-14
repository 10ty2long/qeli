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

## 14 октября: прямое измерение этапов ACK

Отдельный диагностический Linux CLI собран из source 776de2a63efc с семью приватными source-файлами для SHA256 fingerprints синтетических bare IPv4 ACK5203, CLOCK_MONOTONIC timestamps и finite try_lock ring262144. Репозиторий qeli/native не изменялся. Два selector-теста того же модуля выполнены в маленьком тестовом пакете; его заглушка atomic dump не проверяет запись файла. Настоящая Qeli-запись/SIGUSR1 проверена пилотом. Первая сборка отказала из-за отсутствующих conformance fixtures, две сборки всей cfg(test) библиотеки — по OOM; исходные FAIL и очищенные root/cache сохранены. Диагностический release профиль ThinLTO/codegen4 отличается от штатного FatLTO/codegen1. Это отдельный артефакт 4f550f22e274, не shipping/native adoption.

Новые3sessions/6windows: stock текущего CLI, диагностический CLI с probeOFF и тот же диагностический CLI с probeON. Legacy rawTCP one-carrier, две встречные P1/8сек iperf connections, downlink100M/unlimited. Server send/recv requested262144, actual tb/rb524288; INI одинаковы кроме server pin.

| Downlink | CLI / probe | Uplink, Mbps | Downlink, Mbps | ACK p50, ms |
|---|---|---:|---:|---:|
| 100000000 | stock | 319.36 | 100.08 | 0.822205 |
| 100000000 | off | 239.01 | 100.09 | 0.949443 |
| 100000000 | on | 251.26 | 100.04 | 0.911861 |
| unlimited | stock | 159.93 | 526.51 | 15.022166 |
| unlimited | off | 162.41 | 536.84 | 14.158821 |
| unlimited | on | 150.30 | 534.13 | 17.156824 |

Наблюдаемые соотношения: down=100000000 stock→off: uplink×0.748, ACKp50×1.155; down=100000000 off→on: uplink×1.051, ACKp50×0.960; down=unlimited stock→off: uplink×1.016, ACKp50×0.943; down=unlimited off→on: uplink×0.925, ACKp50×1.212. Один последовательный пилот не доказывает нейтральность измерения; stock/OFF также включает отличие сборки. Нельзя объединять эти результаты с предыдущими stock бенчмарками или считать причину исторической задержки закрытой.

По каждому ACK связаны полный одинаковый заголовок PCAP и семь SHA256 stage stamps. Дубликаты и неполные цепочки исключены из timing population с явными счётчиками в evidence; до стадии выводов допускаются только уникальные complete identities. Ring accounting: down=100000000: server events=121009 lost=103 full=0, client events=90732 lost=102 full=0; down=unlimited: server events=82600 lost=8 full=0, client events=61932 lost=24 full=0.

| Downlink | Unique complete ACKs | Segment | p50 / p99, ms |
|---|---:|---|---:|
| 100000000 | 30096 | server_read_to_dequeue | 0.122074 / 7.414881 |
| 100000000 | 30096 | server_dequeue_to_encrypt | 0.000719 / 0.001723 |
| 100000000 | 30096 | encrypt_to_client_plaintext | 0.190948 / 3.744683 |
| 100000000 | 30096 | client_plaintext_to_write_begin | 0.083340 / 2.079618 |
| 100000000 | 30096 | client_tun_write | 0.003847 / 0.109530 |
| unlimited | 20616 | server_read_to_dequeue | 14.311134 / 30.921991 |
| unlimited | 20616 | server_dequeue_to_encrypt | 0.000694 / 0.001425 |
| unlimited | 20616 | encrypt_to_client_plaintext | 0.694058 / 9.009626 |
| unlimited | 20616 | client_plaintext_to_write_begin | 0.702128 / 4.437142 |
| unlimited | 20616 | client_tun_write | 0.002850 / 0.115576 |

В unlimited окне server_read_to_dequeue был наибольшим сегментом для17453/20616 complete ACKs (84,66%); p50=14,311134мс против0,122074мс при100M. По source review этот интервал включает blocking_send/out_rx, выбор session/маршрута и handoff в stream rx; это не измерение одной очереди. Чтобы установить внутреннюю причину, нужно разделить эти серверные границы и снизить влияние измерения. OFF→ON unlimited ACKp50 вырос на21,17%; текущий пилот не доказывает нейтральность.

Граница encrypt→client plaintext включает шифрование, очередь/передачу внешнего TCP и получение/расшифрование клиента. Отдельно retained server write completion и signed overlap: завершение локального async write может произойти после обработки у клиента, это не timestamp wire egress. Квантили отдельных сегментов нельзя складывать как одну latency percentile.

Passed workload/settings/lifetimes, actual binary/config/PCAP identities, stage causal ordering, buffer readbacks, полный host/firewall/services/staged preservation и root/archive SHA cleanup. Нагрузка и выгрузка диагностики закончены; временные сборки/кэши лабы удалены. BPF/kprobe/strace/kernel trace не применялись. Q36 остаётся IN_PROGRESS по подтверждению нейтральности и причины на штатной сборке;33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-ack-stage-pilot-20261014.json`.

## 14 октября: разделение серверных handoff границ

Private diagnostic source 8f5932cf9043, CLI c7f573dfa066, прежние ThinLTO/codegen4/opt3 и jemalloc. Три selector-теста того же модуля PASS; actual CLI/dump/PID/hash и bounded build cleanup проверены отдельно. Основной qeli/native не изменён. Номинальная выборка1/16 по low nibble IPv4 ID=0 выполняется до SHA/clock/lock. По PCAP проверен фактический отбор; это deterministic subset, не заявление о случайной несмещённой выборке всех ACK.

Новые6sessions/12windows: три пары того же бинарника OFF/ON; во втором повторе порядок условий и100M/unlimited обращён. P1/8сек, два встречных iperf соединения, raw TCP legacy one-carrier. Server send/recv262144, actual tb/rb524288, INI match кроме per-session pin. Штатную матрицу и stock-пилот повторно не запускали.

| Repeat | Downlink | Uplink OFF / ON, Mbps | ACK p50 OFF / ON, ms |
|---|---|---:|---:|
| 1 | 100000000 | 280.61 / 255.33 | 0.940306 / 0.930185 |
| 1 | unlimited | 123.59 / 173.95 | 16.699138 / 10.606932 |
| 2 | 100000000 | 189.45 / 371.11 | 1.216701 / 0.865150 |
| 2 | unlimited | 133.13 / 149.26 | 15.088365 / 11.580175 |
| 3 | 100000000 | 186.73 / 295.79 | 1.201702 / 0.843077 |
| 3 | unlimited | 136.48 / 180.26 | 13.779755 / 11.125608 |

| Repeat | Downlink | Complete sampled ACKs | Segment | p50 / p99, ms |
|---|---|---:|---|---:|
| 1 | 100000000 | 1810 | server_read_to_forwarder | 0.039255 / 2.667328 |
| 1 | 100000000 | 1810 | server_forwarder_to_stream_enqueue | 0.000744 / 0.008741 |
| 1 | 100000000 | 1810 | server_stream_queue | 0.035131 / 3.664136 |
| 1 | 100000000 | 1810 | client_plaintext_to_write_begin | 0.082666 / 1.873564 |
| 1 | unlimited | 1451 | server_read_to_forwarder | 0.211360 / 9.907375 |
| 1 | unlimited | 1451 | server_forwarder_to_stream_enqueue | 0.000728 / 0.008007 |
| 1 | unlimited | 1451 | server_stream_queue | 6.019919 / 36.660742 |
| 1 | unlimited | 1451 | client_plaintext_to_write_begin | 0.680580 / 4.067728 |
| 2 | unlimited | 1224 | server_read_to_forwarder | 0.202277 / 9.924692 |
| 2 | unlimited | 1224 | server_forwarder_to_stream_enqueue | 0.000737 / 0.001996 |
| 2 | unlimited | 1224 | server_stream_queue | 6.237488 / 35.223276 |
| 2 | unlimited | 1224 | client_plaintext_to_write_begin | 0.795184 / 4.249312 |
| 2 | 100000000 | 2629 | server_read_to_forwarder | 0.038608 / 3.063380 |
| 2 | 100000000 | 2629 | server_forwarder_to_stream_enqueue | 0.000752 / 0.008709 |
| 2 | 100000000 | 2629 | server_stream_queue | 0.046379 / 2.991839 |
| 2 | 100000000 | 2629 | client_plaintext_to_write_begin | 0.083706 / 1.821141 |
| 3 | 100000000 | 2126 | server_read_to_forwarder | 0.041898 / 2.392429 |
| 3 | 100000000 | 2126 | server_forwarder_to_stream_enqueue | 0.000737 / 0.009073 |
| 3 | 100000000 | 2126 | server_stream_queue | 0.035224 / 4.422004 |
| 3 | 100000000 | 2126 | client_plaintext_to_write_begin | 0.075225 / 2.182222 |
| 3 | unlimited | 1589 | server_read_to_forwarder | 0.203898 / 11.892317 |
| 3 | unlimited | 1589 | server_forwarder_to_stream_enqueue | 0.000708 / 0.007804 |
| 3 | unlimited | 1589 | server_stream_queue | 6.076858 / 34.961849 |
| 3 | unlimited | 1589 | client_plaintext_to_write_begin | 0.847020 / 5.155170 |

Теперь server read→forwarder измеряет blocking_send/out_rx; forwarder→enqueue включает session/route lookup, stream choice и pool copy; enqueue→stream dequeue показывает время до rx.recv, включая scheduling. Это девять границ для одного полного одинакового заголовка, а не корреляция по размеру/общим счётчикам. Не складывать квантили сегментов. Retained complete/excluded/unsampled/lost/full counts, observed horizons и signed async-write overlap доступны в evidence.

В трёх unlimited ON-повторах stream queue p50=6,019919/6,237488/6,076858мс; при100M=0,035131/0,046379/0,035224мс. TUN→forwarder unlimited p50=0,202277–0,211360мс, route/copy=0,000708–0,000737мс. Stream queue был наибольшим отдельным сегментом у61,76–64,85% complete sampled ACKs. Всего10829 complete identities; server ring потерял2 отметки try_lock, client lost0, full0 везде. Фактический отбор6,2497–6,2532%; sampled/full PCAP p50 ratio=0,99255–1,02936 в шести ON-окнах. Это проверка данной выборки, не общая гарантия её несмещённости.

Unlimited ON/OFF ACKp50 ratio=0,63518/0,76749/0,80739: медиана снижается на23,25%, uplink ratio median=1,32085. Даже редкий сборщик не доказал нейтральность; абсолютные6мс нельзя выдавать за точную latency штатного CLI. Следующий осмысленный контроль — обслуживание FIFO/согласованный recordizer, с проверкой реально доступных стадий в MUX; очередной вариант трассировщика или повтор общей матрицы не требуется.

OFF/ON ratios и каждый повтор сохранены раздельно: снижение overhead не означает автоматическую нейтральность. Нельзя объединять новый diagnostic artifact с shipping CLI или предыдущим unsampled pilot. Временные процессы, root/cache/archive удалены; host/firewall/services/staged/source сохранены. BPF/kprobe/strace/kernel trace не применялись. Q36 IN_PROGRESS;33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-server-handoff-runtime-20261014.json`.

## 14 октября: recordizer и серверная FIFO

Execution source 770ef7e14bb9; тот же private ThinLTO/codegen4 diagnostic CLI c7f573dfa066, без новой компиляции и изменений qeli/native. Новые6sessions/12windows, три пары server obf.recordizer.policy=off/required при ON диагностики обоих endpoints. Порядок условий и100M/unlimited обращён во втором повторе. Two simultaneous P1/8сек inner TCP connections, rawTCP one-carrier; фактические legacy_packet_per_record/packet_mux_v1, PID/outer socket/actual tb-rb524288 и исходные INI проверены. Единственные INI delta: server recordizer policy и per-session pin.

| Repeat | Downlink | Uplink legacy / MUX, Mbps | ACK p50 legacy / MUX, ms | Stream queue p50 legacy / MUX, ms |
|---|---|---:|---:|---:|
| 1 | 100000000 | 289.72 / 1175.55 | 0.922641 / 1.064547 | 0.049031 / 0.006099 |
| 1 | unlimited | 103.35 / 421.51 | 29.316383 / 3.159679 | 27.069859 / 0.048029 |
| 2 | 100000000 | 290.31 / 1077.67 | 0.908023 / 1.157373 | 0.044805 / 0.007760 |
| 2 | unlimited | 140.48 / 367.76 | 14.558145 / 3.486144 | 10.075171 / 0.061062 |
| 3 | 100000000 | 214.79 / 1222.21 | 1.012247 / 1.014984 | 0.045017 / 0.005577 |
| 3 | unlimited | 188.59 / 433.51 | 10.517356 / 3.285821 | 2.677874 / 0.049269 |

Совпавшие заголовки PCAP связаны с шестью общими отметками: server read→forwarder→enqueue→dequeue→client TUN begin→done. Всего36046 complete sampled ACK identities; остальные/повторные/пропущенные/непопавшие в1/16 выборку сохранены с точными счётчиками. MUX объединяет несколько пакетов в record: прежних per-ACK crypto/plaintext/write-completion timestamps в этой ветке нет. Их не заменяли временем целого record. Участок dequeue→client TUN включает recordization, encryption, внешний TCP, decode и клиентскую очередь вместе.

Paired MUX/legacy min/median/max: uplink100M 3.712/4.057/5.690; uplink unlimited 2.299/2.618/4.078; ACKp50 unlimited 0.108/0.239/0.312. Это причинный контроль политики batching в данном стенде, не отдельного размера очереди: меняются morphology/deadline, обслуживание writer и buffering. Прошлая OFF/ON серия показала влияние диагностики; одинаковое ON обеих веток не доказывает нейтральность или equivalence observers, поскольку набор доступных отметок различается. Default/config/release artifact не меняли. Нельзя объединять с shipping numbers и заявлять DPI equivalence.

Workload/negotiation/topology/headers/common-stage ordering/buffers/source/host/firewall/services/staged и SHA inventory/archive cleanup PASS; все own ресурсы лабы удалены. Документированы actual drops/retransmits, исходные исторические FAIL сохранены. Q36 IN_PROGRESS по границам принятия shipping причины;33/37 (89,2%),NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-recordizer-handoff-20261014.json`.

## 14 октября: штатный CLI и итог Q36

После private localization выполнены ещё6sessions/12windows на штатном CLI6f617bdcab5a без QELI_ACK_PROBE, SIGUSR1 и новой компиляции. Три пары legacy/MUX при том же одном carrier и policy-only INI delta; PCAP/ss controls остаются. Абсолютного zero-overhead утверждения нет. Runtime generation/source/config/PID/socket/matching headers/receiver roles/host/staged/root/archive cleanup PASS.

| Repeat | Downlink | Uplink legacy / MUX, Mbps | ACK p50 legacy / MUX, ms |
|---|---|---:|---:|
| 1 | 100000000 | 283.48 / 1240.52 | 0.773993 / 0.989687 |
| 1 | unlimited | 156.61 / 403.49 | 13.882233 / 3.277941 |
| 2 | 100000000 | 144.76 / 1240.37 | 1.197513 / 1.012411 |
| 2 | unlimited | 157.38 / 431.46 | 17.154599 / 3.166323 |
| 3 | 100000000 | 281.17 / 1235.29 | 0.874503 / 0.956438 |
| 3 | unlimited | 150.82 / 432.10 | 13.001141 / 3.119505 |

Unlimited paired MUX/legacy median uplink=2.742×, downlink=2.043×, ACKp50=0.236×. При downlink100M ACKp50 median=1.094×: batching/deadline имеет цену для коротких ACK даже при меньшей очереди. Это подтверждает эффект политики на рабочем бинарнике; private queue timestamps не переименованы в shipping stamps. Shared per-stream FIFO в legacy задерживает ACK вместе с downlink data; batching повышает скорость обслуживания очереди. Не включали required глобально: он меняет morphology и требует совместимого клиента.

[Итог пяти критериев Q36](AUDIT-Q36-FINAL.md). Benchmark audit DONE в указанном объёме; finite VM data не являются capacity/performance budget PASS. Исходные FAIL, kernel FULL_RING overload и legacy queue drops сохранены. Дальнейший latency tuning — отдельная оптимизация, не основание повторять завершённые проверки.34/37 (91,9%); открытыQ29/Q31/Q34. NO_PUSH_DEPLOY_RELEASE.
