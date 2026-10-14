# Q36: одинаковая суммарная нагрузка P1/P4 и счётчики TUN

13 октября 2026. Выполнены 6 независимых сессий / 60 окон по 8 с на TCP raw и QUIC: UDP-полезная нагрузка 100/150/200/300/400 Мбит/с **на направление** при одном и четырёх потоках. Для P4 скорость одного потока разделена на 4, поэтому суммарная нагрузка совпадает с P1. Второй повтор меняет порядок режимов и нагрузки. Все три повторения, потери и выбросы сохранены.

Условия: private sibling namespaces одной двухъядерной VM .10, outer/inner IPv4, TUN 1400/carrier 1500, datagram 1200, affinity 0–1/nice 0. Использован прежний квалифицированный CLI 52511cebd43a, без пересборки. Все 916 data-plane input совпадают с прежним source applicability; последние изменения относятся к документации и Android. Совпадение с новым полным stock-бинарником не заявлено. Defaults не менялись.

Медианы трёх независимых сессий; ↑ клиент→сервер, ↓ сервер→клиент. Полные samples/min/max/spread/CPU/RSS/UDP counters сохранены в evidence.

|Mode|Mbps/direction|P1 delivered ↑/↓|P4 delivered ↑/↓|P1 loss ↑/↓ %|P4 loss ↑/↓ %|
|---|---:|---:|---:|---:|---:|
|tcp-plain-raw|100|99.99/99.99|99.98/99.99|0.00/0.00|0.00/0.00|
|tcp-plain-raw|150|149.99/149.99|149.99/149.99|0.00/0.00|0.00/0.00|
|tcp-plain-raw|200|199.98/199.98|199.98/199.97|0.00/0.00|0.00/0.00|
|tcp-plain-raw|300|292.13/298.98|213.02/286.56|0.15/0.00|28.80/2.70|
|tcp-plain-raw|400|253.84/390.11|202.80/284.90|36.39/1.99|49.14/26.90|
|udp-quic|100|99.99/99.99|99.99/100.00|0.00/0.00|0.00/0.00|
|udp-quic|150|149.98/149.98|149.98/149.98|0.00/0.00|0.00/0.00|
|udp-quic|200|199.98/199.97|199.98/199.98|0.00/0.00|0.00/0.00|
|udp-quic|300|281.94/289.48|276.21/152.87|5.88/0.00|7.81/43.04|
|udp-quic|400|277.83/212.26|219.21/145.15|28.73/39.45|45.05/53.03|

При 100–200 Мбит/с на направление потерь не было во всех 36 соответствующих окнах. При 300–400 Мбит/с потери и влияние числа потоков измерены при одинаковой предложенной нагрузке. Это граница проверенных коротких окон данного стенда; не гарантированная максимальная производительность продукта, не полный набор 12 режимов и не долгосрочная capacity-квалификация. Прежний P4=400+400 и P1=100+100 из общего бенчмарка сохранён отдельно и с этим экспериментом не смешан.

## Где зарегистрированы потери

Отдельно выполнены 3 свежие TCP raw-сессии / 12 окон: UDP 300 Мбит/с на направление и TCP bidir при P1/P4. До/после окон прочитаны ip-link64-bit packet/drop counters и native client status, с 2-секундными idle границами. PID/start ticks, TUN ifindex/MTU/qdisc/txqlen, status generation/plan и монотонность проверены. Ни трассировки пакетов, ни BPF, ни изменения очередей нет. CPU sampler прежний; нулевая стоимость наблюдения не заявлена.

|Payload|Streams|Delivered ↑/↓ Mbps|Client TUN TX drops|Server TUN TX drops|Native TCP uplink drops|
|---|---:|---:|---:|---:|---:|
|udp|1|289.94/299.95|7798|0|0|
|udp|4|217.14/294.15|68508|0|0|
|tcp|1|170.76/551.97|226|499|0|
|tcp|4|149.78/567.39|1159|1449|0|

Клиентский TUN действительно зарегистрировал 257250 TX drops суммарно в этих 12 окнах; внутренний TCP uplink Qeli потерь не зарегистрировал. TUN TX — направление kernel→reader; эти счётчики отделены от server session drops и socket RcvbufErrors. Это локализация зарегистрированных потерь на границах, не доказательство времени каждой стадии и не точное сопоставление каждого потерянного payload с отдельным kernel event. TCP-асимметрия остаётся отдельным результатом; её окончательная причина этим не объявляется.

Все raw receiver JSON, cookies/stream rates/duration, process CPU/RSS и счётчики проверены независимо. Полный host/production-service/staged-binary guard PASS; все три собственных root и архивы SHA-проверены и удалены. Новые Rust/native сборки не выполнялись. Q36 IN_PROGRESS; общий 33/37(89,2%). NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-equal-load-20261013.json` и `q36-queue-boundaries-20261013.json`.

## Парная проверка длины клиентской очереди

Ещё 3 сессии / 12 окон: в каждой той же generation сравнили txqueuelen 500 и 1000, только на собственном TUN. UDP 300 Мбит/с P4 и TCP bidir P1; второй повтор меняет порядок. Product defaults не менялись.

|Payload/streams|Client queue|Delivered ↑/↓ Mbps|Receiver loss ↑/↓ %|Client TUN TX drops|
|---|---:|---:|---:|---:|
|udp_p4|500|202.47/288.21|32.34/2.61|80910|
|udp_p4|1000|212.56/276.37|28.58/5.96|71633|
|tcp_p1|500|170.29/580.71|—|384|
|tcp_p1|1000|218.61/320.10|—|211|

В 3/3 пар TCP суммарный goodput при 1000 оказался ниже 500. Медианы парных изменений: uplink +27.01%, downlink -44.88%, сумма -25.10%. Это измеренное влияние очереди на распределение скорости. Увеличение не квалифицировано как общий performance fix; defaults сохранены, окончательная причина исходной TCP-асимметрии остаётся открытой. Все индивидуальные пары сохранены.

Доказательство парного вмешательства: `release/certification/evidence/q36-queue-length-control-20261013.json`.


## Отдельные направления TCP: 13 октября

После проверки длины очереди выполнены **3 свежие сессии / 9 окон по 8 секунд**: TCP raw, P1 upload, download и bidir в одном поколении каждой сессии; второй повтор меняет порядок. Исходные очереди и настройки сохранены. Условия и CLI прежние: IPv4/IPv4, два vCPU одной VM, affinity 0–1/nice 0, TUN1400/carrier1500, CLI52511cebd43a и 916 совпадающих native/config/helper inputs. Трассировок, BPF и новой сборки нет. Предыдущие статистические популяции не объединяются с этой.

Медианы трёх сессий; CPU процесса дан в процентах одного ядра. Прочерк обозначает направление, в котором полезная нагрузка не запрашивалась, а не измеренный ноль.

|Window|Delivered ↑ Mbps|Delivered ↓ Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Server TUN TX drops|
|---|---:|---:|---:|---:|---:|---:|
|Upload only|139.93|—|16.33|19.67|53.00|0.00|
|Download only|—|692.55|78.11|73.33|0.00|1019.00|
|Bidirectional|172.11|567.44|79.22|77.88|270.00|298.00|

Upload остаётся низким и без одновременного download. При отдельном upload CPU обоих процессов значительно ниже одного полностью занятого ядра; поэтому объяснение одной конкуренцией CPU при bidir не подтверждено. Это средние CPU за окно, не доказательство отсутствия кратковременного насыщения.

Во всех трёх отдельных upload-окнах sender retransmits равны client TUN TX drops: **34/62/53**. В отдельных download-окнах они равны server TUN TX drops: **1034/1019/952**. В bidir такого общего равенства нет: TUN переносит также ACK/control. Прикладные native TCP uplink/downlink drop counters клиента не выросли во всех девяти окнах. Совпадение суммарных чисел связывает эти результаты с потерями TUN, но не устанавливает соответствие каждого пакета, причину driver drop, стадию ACK delay или исправление TCP-асимметрии. Короткие окна не исключают влияния congestion/slow-start; TCP-over-TCP и пределы буферизации пока являются гипотезами.

Source review: клиент и сервер передают полную шифрованную запись через write_all_flush; гипотеза отдельной записи header/body не подтверждена. Клиентский TUN использует ограниченный пул чтения до64 buffers /4MiB и fq_codel/500, серверный — общий32MiB пул и multiqueue/1000. Различие само по себе не доказывает причину. Неподтверждённого увеличения очередей/пулов в продукте нет.

Сборщик первоначально записал локальные логи в Windows CP1251/CRLF. Первый UTF8-only qualifier отказал; второе исправление отказало на нечисловых TCP-only loss/jitter=None. Оба отказа сохранены. Финальный qualifier читает полные исходные UTF8 логи из SHA-проверенного архива и подтверждает точный префикс локального снимка после нормализации перевода строк. None сохранены в samples без нулевой подстановки и числовых summaries. Все 38 raw JSON /6 логов связаны SHA; сетевые проверки не повторялись. Будущий внешний шаблон сборщика теперь пишет UTF8 bytes явно; шаблон не объявлен повторно исполненным runtime. Общий benchmark helper уже сохраняет worker logs в UTF8 и не изменён.

Полный source/host/service/staged CLI/archive/cleanup gate PASS; собственные процессы, namespaces и удалённый root/archive удалены. Квалификация CURRENT_SAME_SESSION_TCP_DIRECTION_CONTROL_QUALIFIED. Q36 остаётся открыт, общий итог33/37 (89,2%); NO_PUSH_DEPLOY_RELEASE.


## Длительность upload и темп отправителя: 13 октября

Ещё **6 свежих сессий /12 окон** проверили две независимые гипотезы. Сначала три парные сессии сравнили TCPraw P1 upload8/32с. Затем три другие — unlimited и iperf TCP producer300Mbps в восьмисекундных окнах. В каждой паре PID/NetworkPlan/generation/TUN совпадают, повтор2 меняет порядок. IPv4/IPv4, прежняя двухъядерная VM, affinity0–1/nice0, MTU1400/1500, CLI52511cebd43a;916 frozen inputs прежние. Native product, очереди и qdisc не менялись; новых builds/caches/tracers/BPF нет. Между экспериментами и с прежними популяциями samples не объединяются.

### Удлинение окна

Медианы трёх сессий; CPU дан в процентах одного ядра.

|Seconds|Receiver Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Retransmits|
|---|---:|---:|---:|---:|---:|
|8|173.19|20.78|25.55|43.00|43.00|
|32|165.15|20.18|24.76|25.00|25.00|

За32с скорость до уровня отдельного download не восстановилась. Во второй половине32с sender intervals тоже содержат retransmits во всех трёх повторах:6/54/7; потери не ограничены стартом. Посекундные sender bytes/retransmits/TCP_INFO(cwnd, RTT, rttvar, PMTU) сохранены с проверкой непрерывности интервалов, точного совпадения сумм и отсутствия omitted/warmup. Sender interval bitrate не подменяет доставленную скорость: receiver summary квалифицирован отдельно. Этот32-секундный контроль не исключает любое более длительное изменение congestion и не является capacity/soak.

### Темп TCP producer

Изменён только параметр iperf `-b 300000000`; это программное управление темпом тестового отправителя до TUN, не лимит/исправление в Qeli. Обычный TCP producer работает без `-b`; начальный трафик не исключён.

|TCP producer|Receiver Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Retransmits|
|---|---:|---:|---:|---:|---:|
|Unlimited|145.99|18.78|22.89|50.00|50.00|
|300Mbps|300.06|36.11|40.22|0.00|0.00|

В paced варианте **3/3 окна доставили300,03–300,07Mbps, без retransmits и TUN TX drops**; native client TCP drop counters также0. В unlimited —132,16–161,10Mbps, с kernel TX drops/retransmits50/8/63. Медиана парного прироста receiver скорости —2.06×. Следовательно, проверенный uplink способен передавать300Mbps; прежние≈146Mbps не являются неизменным вычислительным пределом этой сборки. Зависимость от режима отправки подтверждена, но помикропакетная burst-причина и конкретный runtime drop reason не измерены. Это проверенная мера для данного тестового producer, не безопасное универсальное преобразование трафика чужих приложений или максимальный throughput. Defaults бенчмарка не заменены paced нагрузкой, чтобы не скрывать unlimited результат.

### Проверка upstream TUN

В [upstream6.12.105](https://github.com/gregkh/linux/blob/v6.12.105/drivers/net/tun.c) заполненное pointer ring может увеличить TX dropped. Этот же счётчик включает другие ошибки, поэтому агрегат не доказывает FULL_RING. Изменение txqueuelen пересчитывает и driver ring. Версия совпадает с номером ядра лабы, но точное соответствие Debian-сборке не проверено. [Upstream backpressure-патч отменили](https://github.com/torvalds/linux/commit/f11b48aa674b475f196bede7d69593c050107fc5) после регрессии другого теста; его здесь не применяли. Это source review, не новая квалификация ядра.

Оба runtime/source/host/service/staged CLI/archive/cleanup gates PASS. Все собственные ресурсы удалены после SHA-архивирования; UTF8 логи проверены с первой попытки. Для каждого эксперимента26 raw JSON/6 worker logs связаны SHA; sender telemetry полностью внутри исходных iperf JSON. Q36 остаётся открыт: практическая producer-мера установлена, итог33/37(89,2%) не изменён. Далее уместен парный Linux TUN pacing/qdisc контроль с upload+bidir, до изменения продукта; большие очереди уже показали отрицательный tradeoff. NO_PUSH_DEPLOY_RELEASE.


## Клиентский TUN: fq_codel против fq, 13 октября

**3 свежие сессии /12 окон по8с**, TCPraw P1 unlimited upload и bidir, outer/innerIPv4. В каждой сессии изменён только планировщик собственного client TUN: исходные параметры fq_codel против стандартных fq. Поколение, PID, NetworkPlan, MTU1400/1500, txqueuelen500, CLI52511cebd43a и916 native/config/helper inputs сохранены; повтор2 меняет порядок. Параметры двух классов естественно различаются — это сравнение вариантов, не изоляция одного алгоритма при одинаковых числовых настройках. Product/kernel source и defaults не изменены; builds/tracers/BPF отсутствуют.

Медианы трёх сессий; прочерк — направление без запрошенной полезной нагрузки. Полные receiver, retransmits, CPU/RSS, TUN/native и tc-s-JSON counters сохранены.

|Client qdisc|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Client qdisc drops|
|---|---|---:|---:|---:|---:|
|fq_codel|tcp_up|136.09|—|50.00|0.00|
|fq_codel|tcp_bidir|160.49|562.79|242.00|0.00|
|fq|tcp_up|131.34|—|36.00|0.00|
|fq|tcp_bidir|167.05|563.19|249.00|0.00|

Для решения используются изменения внутри каждой пары, а не отношение отдельных медиан. Bidir sum складывает оба receiver одной пары.

|Repeat|Upload change %|Bidir sum change %|
|---|---:|---:|
|1|-18.96|-11.88|
|2|-15.46|+8.94|
|3|+1.31|-7.71|

Медиана парного upload change **-15.46%**, bidir sum **-7.71%**. Один bidir повтор улучшился; два ухудшились. Все повторы сохранены. Замена default на fq **не обоснована**, поэтому fq_codel оставлен.

FQ применялся: kind/handle/options проверены, throttled counter вырос во всех6 FQ-окнах (1657/2611/2069/4002/1896/3570). Все12 qdisc-drop deltas и client native TCP-drop deltas равны0, но kernel TUN TX losses сохранились. Это не устранение потерь и не доказательство конкретного FULL_RING event. По [upstream iproute2 manual](https://man7.org/linux/man-pages/man8/tc-fq.8.html) FQ учитывает TCP pacing и socket flows; эти возможности не гарантируют исправление данного TUN. Результат относится к TCPraw/P1/данной VM, не всем режимам/платформам.

### Исправленная ошибка контроллера

Две исходные попытки сохранили2+2 fq_codel окна и отказали до FQ-измерений. Контроллер запрашивал fq с handle1, уже занятым fq_codel. Отказ сохранился с успешно загруженным модулем; прежнее объяснение автозагрузкой **отозвано**. [Source6.12.105](https://github.com/gregkh/linux/blob/v6.12.105/net/sched/sch_api.c) проверяет соответствие типа существующему handle. Исправленный контроллер использует fq_codel1/fq2 и до нагрузки проверяет1→2→1, во всех трёх сессиях PASS. Оба исходных FAIL остаются отдельными; их4 окна не включены в новую популяцию. Это ошибка измерительной обвязки, не обнаруженный баг Qeli.

Штатный sch_fq временно загружен явно. После удаления всех собственных процессов/namespaces/root/archive модуль выгружен при refcnt0. Исходный module set, global qdiscs/default, provider SHA, services, staged CLI, полный host/source и SHA-archive/cleanup gates PASS. Обе отказавшие попытки и отдельная module-проба также полностью очищены; нового техдолга стенда нет.50 raw JSON/6 worker logs квалифицированы с первой попытки. Документация сохранит negative candidate, не выдавая его за performance fix/PASS.

Q36 IN_PROGRESS, итог33/37(89,2%). Смена планировщика исключена как подтверждённое универсальное исправление. Producer pacing300Mbps остаётся ранее измеренной мерой только для тестового отправителя; следующий адресный кандидат может ограничивать темп до TUN без участия приложения, с проверкой bidir tradeoff. NO_PUSH_DEPLOY_RELEASE.


## Ограничение темпа перед TUN, 13 октября

**3 новые парные сессии / 12 окон по 8 секунд:** исходный client TUN fq_codel против fq maxrate 300Mbit на поток. TCPraw P1 upload и bidir, внешний и внутренний IPv4, та же VM с двумя vCPU. Отправляющее приложение без лимита: iperf target_bitrate=0, параметр `-b` отсутствует. Менялся только qdisc собственного client TUN; исходные параметры fq_codel, txqueuelen500, MTU1400/1500, generation/PID/NetworkPlan и все916 зафиксированных native/config/helper inputs сохранены. Повтор2 меняет порядок. Новых сборок, трассировки, BPF и изменений продукта нет.

`tc -j` подтвердил maxrate=37500000 байт/с до нагрузки и на обеих границах каждого окна. Это ограничение **каждого потока**, а не суммарного трафика профиля. [Исходный код Linux6.12.105](https://github.com/gregkh/linux/blob/v6.12.105/net/sched/sch_fq.c) обновляет время следующего пакета при включённом максимуме и с TCP EDT; upstream source review не доказывает побайтовое совпадение с собранным Debian kernel. Runtime separately фиксирует SHA штатного модуля. Лимит относится к пакетам в планировщике, поэтому полезные TCP-данные могут идти медленнее300Mbps.

Медианы трёх сессий; прочерк означает отсутствие запрошенной полезной нагрузки в направлении. Исходные receiver, TCP-повторы, CPU/RSS и счётчики сохранены, выбросов не исключали.

|Client qdisc|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client qdisc drops|
|---|---|---:|---:|---:|---:|---:|
|original fq_codel|tcp_up|137.51|—|36.00|0.00|0.00|
|original fq_codel|tcp_bidir|167.11|534.53|274.00|643.00|0.00|
|fq maxrate 300Mbit/flow|tcp_up|288.60|—|0.00|0.00|0.00|
|fq maxrate 300Mbit/flow|tcp_bidir|184.54|532.82|380.00|679.00|0.00|

Все три capped upload окна дали288,56–288,84Mbps без TUN TX drops на обоих концах и без TCP-повторов. Baseline upload:133,86–156,27Mbps, drops/retransmits36/28/115. Это воспроизводимая мера против upload-провала на данном стенде без участия приложения. Все12 client qdisc-drop и native client TCP-drop deltas равны0; FQ throttled вырос во всех6 capped окнах, что подтверждает применение ограничения.

Парные изменения рассчитаны внутри каждой сессии; bidir sum складывает оба receiver одной пары. Отношения отдельных медиан не используются для решения.

|Repeat|Upload change %|Bidir upload change %|Bidir download change %|Bidir sum change %|
|---|---:|---:|---:|---:|
|1|+115.77|-6.33|+11.58|+7.74|
|2|+109.84|+15.68|-4.53|+0.26|
|3|+84.69|+10.20|-0.32|+2.19|

Медиана парного upload gain **+109.84%**, bidir sum **+2.19%**. Однако во всех трёх capped bidir окнах client TUN TX drops380/523/335 и server TUN TX drops679/429/705; upload-повторы185/375/252. Потери в обоих направлениях не устранены; в одном повторе upload снизился, в другом download снизился. Лимит300Mbit и fq **не вводятся по умолчанию**. Сочетание смены класса и ограничения темпа является сравнением конфигураций, не изоляцией единственного параметра. Прежние unlimited fq и producer pacing остаются отдельными популяциями.

Перед нагрузкой переключение fq_codel1→fq2→fq_codel1 PASS3/3 с точным восстановлением исходных fq_codel options. После SHA-архивирования удалены собственные процессы, namespaces, root и удалённый архив. Временно добавленный sch_fq выгружен при refcnt0; исходные module set, global qdisc/default, services, provider/staged CLI SHA и полный host/source gates PASS. Квалификация50 raw JSON /6 worker logs прошла с первой попытки. Нового долга по очистке стенда нет.

Q36 остаётся IN_PROGRESS: ограничение темпа помогает upload, но общего исправления TUN/ACK-потерь и подтверждённого предела производительности нет. Агрегатный TX dropped не определяет точную причину FULL_RING. Общий итог33/37(89,2%), NO_PUSH_DEPLOY_RELEASE.


## Согласованный recordizer против legacy, 13 октября

**6 свежих сессий /18 окон по8с**, три пары `off`/`required`, TCPraw P1 upload/download/bidir, outer/innerIPv4. Порядок сессий и направлений меняется в повторе2. Та же CLI52511cebd43a,916 native inputs и неизменные9 helpers; original MTU1400/1500, qdisc и очереди. Отправитель без лимита, без warmup/omit. Сравнение между отдельными поколениями соединения, не переключение внутри работающего соединения. Native/kernel/defaults не менялись, сборок/пейсинга/трассировки/BPF нет.

Новые профили задают `prefer` в [конструкторе](../../../qeli/src/config/server.rs); старый baseline и [бенчмарк](../../../scripts/benchmark.py) используют `off`. Здесь `required` заставляет успешно согласовать режим, исключая незаметный fallback: NetworkPlan показывает packet_mux_v1 во всех3 required сессиях и legacy_packet_per_record во всех3 off. Client log независимо подтверждает PACKET_MUX_V1. SHA-архивированные INI проверены: единственное заданное поле recordizer — policy; остальные параметры исходные: batch2–8ms,16frames, случайная морфология размера и fragmentation. Это проверка работающего режима; не отдельный runtime тест fallback политики prefer.

Медианы трёх сессий; прочерк — нет запрошенной полезной нагрузки в направлении. Все raw receiver/retransmit/CPU/RSS/TUN/native counters сохранены, без исключённых повторов.

|Policy|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client CPU %|Server CPU %|
|---|---|---:|---:|---:|---:|---:|---:|
|off|tcp_up|159.51|—|90.00|0.00|21.55|17.89|
|off|tcp_down|—|686.43|0.00|500.00|73.11|78.88|
|off|tcp_bidir|154.29|600.28|432.00|697.00|77.66|79.77|
|required|tcp_up|1421.05|—|182.00|0.00|71.44|72.00|
|required|tcp_down|—|1530.85|0.00|318.00|71.89|67.33|
|required|tcp_bidir|436.40|1189.07|501.00|93.00|76.22|73.11|

Парные изменения между согласованными сессиями каждого повтора. Bidir sum — сумма обоих receiver внутри соответствующей сессии.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|
|---|---:|---:|---:|---:|---:|
|1|+938.33|+120.53|+208.41|+98.09|+118.45|
|2|+753.27|+142.25|+140.67|+118.82|+124.28|
|3|+795.19|+115.73|+194.31|+91.36|+112.41|

Медианы парных изменений: upload **+795.19%**, download **+120.53%**, bidir sum **+118.45%**. Полная таблица сохраняет различия направлений; общий эффект нельзя переносить на каждый отдельный поток, все транспортные режимы или платформы. Число окон с TUN/native потерями записано отдельно в analysis evidence; нулевая медиана не означает отсутствие потерь во всех повторах. Это не универсальный performance/capacity PASS и не основание автоматически менять default.

Source review [client writer](../../../qeli/src/client/mod.rs) подтверждает освобождение текущего TunPacket до ожидания outer write **в обеих ветках**. Гипотеза удержания текущего буфера во время записи не подтверждается. [Linux pump](../../../qeli/src/transport_core/linux_tun.rs) имеет максимум64 read buffers при nominal channel4096; это ограниченная память/обратное давление, не само по себе доказанный баг. Recordizer меняет группировку, размеры, фрагментацию и сроки flush одновременно; результат не определяет точную задержку ACK или событие FULL_RING.

Source/generation/negotiated-mode/full host/services/staged CLI/SHA archive/cleanup gates PASS. Собственные процессы, namespaces, root и удалённый архив удалены;74 raw JSON/12 worker logs квалифицированы. Нового долга очистки нет. Q36 IN_PROGRESS, итог33/37(89,2%), NO_PUSH_DEPLOY_RELEASE.

Исходная попытка сохранила3 baseline окна, затем worker отказал до required измерений: слишком длинный путь контрольного Unix-сокета (SUN_LEN). R2 использует короткие имена каталогов и проверяет длину пути до запуска. Исходный FAIL и3 окна сохранены отдельно, в таблицы R2 не включены; обе попытки полностью очищены. Это исправление обвязки эксперимента, не баг recordizer.


## Срок отправки recordizer:2–8 мс против1 мс,13 октября

**6 новых сессий /18 окон по8с**, три пары required recordizer с исходным deadline2–8ms и fixed1ms. TCPraw P1 upload/download/bidir, outer/innerIPv4, та же2vCPU VM. Изменены только два INI delay поля; max_packets16, случайные размеры, фрагментация и прочие параметры сохранены. В повторе2 обратный порядок сессий/нагрузок. В обоих условиях iperf записывает интервалы1с; отправитель без лимита и omit/warmup. CLI52511cebd43a,916 native inputs и9 helpers прежние, MTU/очереди/qdisc/affinity не менялись. Это новая отдельная популяция, не объединённая с прежними off/required или pacing числами.

PACKET_MUX_V1 подтверждён NetworkPlan и client log во всех6 сессиях. SHA-архивированные INI проверены: required в обоих условиях; только fixed1ms добавляет `obf.recordizer.batch.delay_min_ms=1` и `obf.recordizer.batch.delay_max_ms=1`. Нативных сборок, изменения продукта/default, трассировки/BPF нет.

Медианы трёх сессий. RTT-столбец сначала берёт медиану **интервальных TCP_INFO снимков отправителя на клиенте**, затем медиану трёх сессий. Это не распределение RTT каждого пакета и не точная задержка ACK. В reverse направлении клиент не наблюдает TCP_INFO удалённого отправителя: N/A, без подстановки0.

|Deadline|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Median sampled client-sender RTT µs|
|---|---|---:|---:|---:|---:|---:|
|default2–8ms|tcp_up|1416.59|—|173.00|0.00|3221.50|
|default2–8ms|tcp_down|—|1492.80|0.00|380.00|N/A|
|default2–8ms|tcp_bidir|443.35|1186.68|499.00|165.00|7166.00|
|fixed1ms|tcp_up|1364.67|—|250.00|0.00|3687.00|
|fixed1ms|tcp_down|—|1441.48|0.00|220.00|N/A|
|fixed1ms|tcp_bidir|412.69|1238.19|404.00|238.00|7404.50|

Отдельно до нагрузки отправлены30 ICMP probes на сессию. Их задержки и средние значения проверены по raw ping; это idle ICMP, не RTT под TCP-нагрузкой.

|Deadline|Idle ICMP RTT: median of3 session averages ms|
|---|---:|
|default|13.770|
|1ms|4.919|

Парные изменения внутри каждого повтора; bidir sum складывает оба receiver соответствующей сессии.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|Idle ICMP average change %|
|---|---:|---:|---:|---:|---:|---:|
|1|+2.92|-3.71|-14.39|+5.96|+0.34|-63.83|
|2|-7.89|-7.04|+21.01|+13.57|+15.61|-64.28|
|3|+11.39|+7.04|-6.92|+2.62|+0.06|-66.04|

Медианы парных изменений: upload **+2.92%**, download **-3.71%**, bidir sum **+0.34%**, idle ICMP average **-64.28%**. Все повторы, retransmits, TUN/native drops и CPU/RSS сохранены; нулевая медиана не означает отсутствие потерь в каждом окне. Отдельные loss-window counts находятся в analysis evidence.

[Source review](../../../qeli/src/protocol/recordizer.rs): неполная запись имеет общий deadline без ACK-only классификации; full/count limits могут отправить её раньше. Поэтому настройка2–8ms не добавляет такую задержку безусловно к каждому пакету. Fixed1ms сохраняет пакетирование, но убирает случайность самого deadline; равноценность против DPI этим тестом не подтверждена. Default не изменён. Сравнение проверяет влияние срока, а не конкретное событие FULL_RING/стадию ACK или универсальную capacity.

Все source/generation/host/services/staged CLI/SHA archive/cleanup gates PASS. Собственные процессы/namespaces/root/удалённый архив удалены;80 raw JSON/12 worker logs квалифицированы. Нового долга очистки нет. Q36 IN_PROGRESS,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.


## Фиксированные1 и2 TCP-соединения,13 октября

**Q36-F030 (P2), REPRODUCED / FIX_PENDING:** в3/3 пар двух соединений upload проваливается; все native uplink drops имеют причину QueueFull, а не PoolExhausted/Oversize/NoCarrier. При одном соединении во всех9 окнах native uplink drops0. Для4MiB зарезервированы64 слота по65535 байт; reserve formula при width2 даёт всего21 пакет на очередь. С recordizer уже действует предел inner packet negotiatedMTU+64, здесь1464 байта. Предлагаемое исправление должно использовать этот существующий предел для размера слота, сохранив общий бюджет и резервы остальных carriers/candidate; пока не применено и не проверено. Exact capture конкретного ACK-пакета и kernel FULL_RING этим не заявляется.

**6 новых сессий /18 окон по8с**, три пары1/2 внешних TCP-carriers, TCPraw P1 upload/download/bidir, outer/innerIPv4, та же2vCPU VM. В обоих INI multipath enabled=true/adaptive=false, recordizer required с исходным deadline2–8ms. Отличается только max_streams1/2. В повторе2 обратный порядок сессий/нагрузок. CLI52511cebd43a,916 native inputs/9 helpers, MTU1400/1500, qdisc/очереди/affinity прежние. Отправитель без лимита, без omit/warmup. Новая популяция, прежние recordizer/pacing числа не объединяются.

План max_streams сам по себе недостаточен: реальные ESTABLISHED сокеты проверены по native PID и outer endpoint на обоих концах до нагрузки и на каждой границе окна. Во всех42 снимках пары endpoint зеркальны, фактическая ширина1/2 и tuples сохраняются. PACKET_MUX_V1 подтверждён NetworkPlan/client log; width2 дополнительно подтверждён сообщением о двух активных fixed streams. SHA-архивированные INI проверены. Новых сборок, трассировки, BPF, продуктовых/default изменений нет.

[Flow hash](../../../qeli/src/protocol/mod.rs) закрепляет каждый внутренний поток за одним carrier, не распределяет пакеты одного потока round-robin. Каждая сторона хеширует своё outbound направление самостоятельно; разные потоки могут столкнуться на одном carrier. [Bonded path](../../../qeli/src/client/mod.rs) также включает ограниченный пул копирования4MiB и другие размеры очередей, освобождая исходный TunPacket после копирования. Поэтому сравнение не изолирует одну ACK-стадию; наличие двух сокетов не доказывает конкретное размещение каждого inner flow. Queue Full учитывается как native drop без переприкрепления к другому живому потоку.

Медианы трёх сессий; все повторы/retransmits/CPU/RSS/kernel/native counters сохранены. RTT — медиана интервальных TCP_INFO снимков client sender, затем медиана3 сессий. Reverse sender RTT N/A, без0-imputation; это не распределение RTT каждого пакета.

|Carriers|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client native uplink drops|Client native downlink drops|Median sampled client-sender RTT µs|
|---|---|---:|---:|---:|---:|---:|---:|---:|
|1|tcp_up|1433.04|—|167.00|0.00|0.00|0.00|3369.50|
|1|tcp_down|—|1506.22|0.00|231.00|0.00|0.00|N/A|
|1|tcp_bidir|425.55|1192.89|319.00|309.00|0.00|0.00|7015.50|
|2|tcp_up|24.59|—|0.00|0.00|148.00|0.00|8565.00|
|2|tcp_down|—|1494.84|0.00|187.00|34348.00|0.00|N/A|
|2|tcp_bidir|20.80|1490.85|0.00|257.00|37085.00|0.00|8564.00|

Отдельные30 idle ICMP probes на сессию:

|Carriers|Idle ICMP RTT: median of3 session averages ms|
|---|---:|
|1|13.729|
|2|13.020|

Парные изменения внутри каждого повтора; bidir sum складывает оба receiver соответствующей сессии.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|Idle ICMP average change %|
|---|---:|---:|---:|---:|---:|---:|
|1|-98.28|-16.51|-99.02|+20.71|-10.77|-5.88|
|2|-98.12|+0.13|-94.82|+22.45|-7.57|+1.91|
|3|-98.28|+0.14|-95.25|+29.39|-4.56|-0.25|

Медианы парных изменений: upload **-98.28%**, download **+0.13%**, bidir sum **-7.57%**. Точные native-loss samples/window counts находятся в analysis evidence;0 median не означает0 в каждом окне. Результат не объявляется универсальным исправлением потерь/асимметрии, ACK-приоритетом, пределом capacity или основанием автоматически включать multipath для всех профилей.

Все source/generation/actual socket/host/services/staged CLI/SHA archive/cleanup gates PASS. Собственные процессы/namespaces/root/удалённый архив удалены;80 raw JSON/12 worker logs квалифицированы. Default не изменён, нового долга очистки нет. Q36 IN_PROGRESS,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.


## Q36-F030: кандидат исправления QueueFull проверен в Linux,13 октября

**Статус: LINUX_CANDIDATE_VALIDATED; source adoption/native consumers pending.** Рабочий код пока прежний. Кандидат одного файла client/mod.rs выбирает размер bonded-copy слота по уже допустимому размеру пакета: MAX_TUNNEL_MTU для legacy и max_packet_bytes для mux, с нижней границей2KiB и существующим u16 limit. При MTU1400 получается2048 слотов по2KiB и682 места на width2 очередь вместо64 слотов/21 места. Общий storage budget4MiB и reserve helper для остальных carriers/временного candidate сохранены; fallback allocation и перепривязка Full-потока не добавлены.

В реальной изолированной сборке Rust1.97 выполнены четыре strict feature checks (default/server/client-bin/transport-core-ffi), **15 unit tests**:6 client uplink,8 buffer pool,1 server wire pool. Новый тест проверяет128 pinned пакетов размером1400/40 байт до освобождения очереди, FIFO без дропов и резерв после заполнения stalled queue; это синтетический unit burst, не capture настоящих ACK. Проверены packet bounds/byte budget, прежние Full/Closed/Oversize/PoolExhausted и освобождение памяти. Production release CLI собран с тем же factory profile/jemalloc, одним compiler job. Rustfmt нормализовал только candidate clientmod. Ни platform native artifact refresh, ни принятие исходников этим не подменены.

Сборка использует точно замороженный factory source916 файлов плюс один изменённый clientmod. Current source applicability сохраняет известные7 format/panel-resource отличий от factory; эквивалентность целого текущего CLI не заявляется. **Сервер во всех runtime сессиях — неизменный baseline52511cebd43a**, клиент baseline либо candidate524443061b95. Это отделяет исправление клиента от пересборки сервера.

**6 свежих сессий /18 окон по8с**, три пары baseline/candidate client при fixed2 carriers, required recordizer default2–8ms, TCPraw P1 upload/download/bidir, outer/innerIPv4 на2vCPU VM. В повторе2 обратный порядок сессий/нагрузок. В обоих случаях original INI/MTU/qdisc/очереди/affinity, unlimited caller/no omit,1s senderTCP_INFO и30 idleICMP/session. ProcPID binary SHA и реальные mirrored/stable socket tuples проверены на42 snapshots. Прежняя популяция1/2carriers в эти числа не включена.

Медианы3 сессий, все повторы и kernel/native counters сохранены:

|Client|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Native uplink drops|Client TUN TX drops|Server TUN TX drops|Client peak RSS KiB|
|---|---|---:|---:|---:|---:|---:|---:|
|baseline|tcp_up|24.34|—|127.00|0.00|0.00|59172.00|
|baseline|tcp_down|—|1500.03|36565.00|0.00|132.00|77948.00|
|baseline|tcp_bidir|29.72|1467.71|38521.00|0.00|189.00|81536.00|
|candidate|tcp_up|1278.97|—|0.00|217.00|0.00|57912.00|
|candidate|tcp_down|—|1438.19|0.00|0.00|108.00|78536.00|
|candidate|tcp_bidir|519.27|1104.03|0.00|703.00|166.00|78708.00|

Кандидат имеет **0 native uplink/downlink drops во всех9 окнах**; baseline имеет QueueFull в каждом окне, остальные причины0. Upload-only восстановлен во всех3 парах, median paired ratio **52.91×**. Наблюдаемый максимум RSS любого worker **88632KiB**, ниже действующего512MiB product budget; compiler memory limit1536MiB и временный build swap — отдельные параметры.

|Repeat|Upload ratio|Download change %|Bidir sum change %|
|---|---:|---:|---:|
|1|53.41×|+1.43|+8.16|
|2|52.91×|-4.67|+12.81|
|3|51.47×|-7.11|-7.14|

Median paired bidir sum **+8.16%**. В одном повторе сумма снизилась; направление download в bidir отдаёт часть пропускной способности восстановленному upload. Это не общий performance/capacity PASS, выбросов не исключали. Kernel TUN losses сохраняются; исправляется конкретный native QueueFull, а не объявляется установленной причина каждого kernel FULL_RING/ACK event.

Собственный swap удалён по UUID/inode после остановки unit, исходный swap inventory восстановлен. Owned compiler target/registry/source/root и runtime процессы/namespaces/root/remote archives удалены после SHA-проверки; services/staged CLI/full host/source gates PASS. Build qualification и runtime qualification независимы;80 raw JSON/12 logs runtime проверены с первой попытки.

Перед source adoption остаются legacy bonded/one-carrier проверки и обновление/проверка native consumers. Статус F030 не превращён в окончательный FIXED; product defaults прежние. Q36 IN_PROGRESS,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.


## Q36-F030: совместимость одного канала и незакрытый legacy

**PARTIAL35_QUALIFIED; source/native adoption pending.** 12 фактических сессий,36 попыток окон по8с: **35 пригодных измерений и1 недостоверное bidir-окно iperf3**. Проверены required recordizer/один канал и off/legacy/два канала, TCPraw P1 upload/download/bidir,outer/innerIPv4,original MTU/qdisc/очереди/affinity,unlimited producer/no omit. Сервер всегда exactbaseline52511cebd43a, клиент baseline или уже собранный candidate524443061b95; без новых сборок/изменения defaults. Предыдущие18окон fixed2mux не смешивали с этой популяцией.

Один канал:3полные пары,0native uplink/downlink drops во всех18окнах обоих вариантов. Upload median1447,89→1414,29Mbps; median paired upload−1,26%,download+4,55%,bidir sum+2,34%. Это сохранение поведения в проверенных сценариях, не гарантия нулевой разницы скорости при любой нагрузке; kernel TUN losses остаются.

Legacy:3пары upload/download. Median upload435,57→574,22Mbps,paired+32,27%;download736,23→687,54Mbps,paired−6,63%. QueueFull остаётся во всех8пригодных candidate windows;Oversize/PoolExhausted/NoCarrier0. Bidir candidate имеет только2пригодных повтора: отдельные значения сохранены, статистическая квалификация трёх повторов для этой ячейки отсутствует. Candidate legacy slot16602B/252слота даёт83места в width2 queue против21baseline и682mux приMTU1400. Максимальный accepted legacy packet bound не сужен. Синтетический128burst unit выполнен только дляmux; арифметика ёмкости не устанавливает событие каждого runtime дропа.

Таблица: N3 — медианы; N2 — два отдельных диагностических значения через slash, без вычисления статистики.

|Condition|Client|Phase|N|Up Mbps|Down Mbps|Native uplink drops|Client TUN TX drops|Server TUN TX drops|
|---|---|---|---:|---:|---:|---:|---:|---:|
|required/carriers-1|baseline|tcp_up|3|1447.89|—|0.00|174.00|0.00|
|required/carriers-1|baseline|tcp_down|3|—|1463.53|0.00|0.00|186.00|
|required/carriers-1|baseline|tcp_bidir|3|423.96|1160.85|0.00|645.00|271.00|
|required/carriers-1|candidate|tcp_up|3|1414.29|—|0.00|202.00|0.00|
|required/carriers-1|candidate|tcp_down|3|—|1549.98|0.00|0.00|442.00|
|required/carriers-1|candidate|tcp_bidir|3|457.97|1185.90|0.00|464.00|321.00|
|off/carriers-2|baseline|tcp_up|3|435.57|—|5059.00|0.00|0.00|
|off/carriers-2|baseline|tcp_down|3|—|736.23|18448.00|0.00|0.00|
|off/carriers-2|baseline|tcp_bidir|3|36.27|657.31|22031.00|0.00|0.00|
|off/carriers-2|candidate|tcp_up|3|574.22|—|2706.00|0.00|0.00|
|off/carriers-2|candidate|tcp_down|3|—|687.54|6350.00|0.00|0.00|
|off/carriers-2|candidate|tcp_bidir|2|115.06/118.70|622.61/633.70|6893.00/8282.00|0.00/0.00|0.00/0.00|

|Condition|Repeat|Upload change %|Download change %|Bidir sum change %|
|---|---:|---:|---:|---:|
|required/carriers-1|1|-1.26|+4.55|+9.64|
|required/carriers-1|2|-4.78|+2.01|+0.40|
|required/carriers-1|3|-0.35|+8.45|+2.34|
|off/carriers-2|1|+7.21|-16.26|+7.38|
|off/carriers-2|2|+32.27|-6.61|+6.70|
|off/carriers-2|3|+35.09|-6.63|role-invalid|

Первый контроллер завершился по общему бюджету900с после11полных сессий/33окон. TimeoutError сохранён. Выполнена только недостающая legacy candidate session:upload/download завершились, а bidir вернул0receiver bytes в обоих направлениях. По rawJSON и архивному server iperf log доказаны TX/TX наclientport33208 иRX/RX на33220. В остальных11bidir role assignments противоположные и корректные. [Сервер iperf3 3.18](https://github.com/esnet/iperf/blob/3.18/src/iperf_server_api.c#L762-L769) назначает RX/TX по порядку принятия соединений, [клиент](https://github.com/esnet/iperf/blob/3.18/src/iperf_client_api.c#L324-L330) создаёт сначала TX,потомRX. Это объясняет отказ данного измерения; зависание Qeli им не доказано. Исходный FAIL не заменён успешным результатом,0не добавлен в статистику скорости, никаких повторов «до успеха» нет.

В r1-m1-c/upload final sender bytes1417281536 против суммы1sintervals1417150464: разница131072B,один configured block. Retransmits282совпадают. Оба счётчика/разные горизонты оставлены, скорость берётся из исходного receiver summary. Первый отказ точного byte-sum assert и последующий отказ population(N2) сохранены; дляN2 статистика не вычисляется, существующий минимум3samples сохранён. Остальные sender byte/retransmit sums точные.

Независимая квалификация35окон/157rawJSON/24logs и обеих исходных executions выполнена.82native socket snapshots пригодных окон verified stable/mirrored; failedbidir не имеет полного after-boundary snapshot и такой PASS не заявлен. Максимум RSSworker 95304KiB ниже512MiB. Структурный host совпадает между этапами; оба fullhost/firewall/routes/resolver/services/stagedCLI/source gates PASS, собственные процессы/namespaces/roots/remote archives SHAпроверены/удалены. Оба runtime FAIL и rawfailedwindow сохранены. Рабочий clientmod/native copies пока прежние.

Далее: устранить/учитывать конфликт ролей в измерителе перед новыми bidir-проверками, разобраться с оставшимся legacy QueueFull и выполнить nativeconsumer refresh/проверки при переносе патча. Q36 IN_PROGRESS,F030не окончательноFIXED,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.

## Проверка ролей TCP-bidir включена в измеритель

Новые TCP-bidir прогоны запрашивают `--get-server-output`. Общий парсер сопоставляет оба конца каждого соединения по IP/порту и требует противоположных TX/RX, а также полную карту из `2 × P` соединений. При TX/TX, RX/RX, отсутствующих или неоднозначных данных измерение отклоняется; raw iperf, счётчики и оба process sampler сохраняются до проверки. Порядок принятия сокетов больше не служит доказательством направления. Поддержаны server JSON/text и нормализация IPv4-mapped IPv6.

Offline report повторяет эту проверку для новых результатов с `tcp_bidir_stream_roles_required=true`; требования входят в ключ сравнимости прогонов. Старые записи без server output сохраняют прежний формат и не смешиваются с новой политикой. Если server output присутствует, его роли проверяются и у старой записи. UDP-bidir и однонаправленные проверки не менялись.

**86 тестов PASS и один самостоятельный живой TCPraw P1 bidir-прогон на .10 PASS.** iperf3.18 вернул server output text: clientport44654 TX ↔ server RX, clientport44662 RX ↔ server TX. Оба receiver summary приняты; 63,26/695,34Mbps — диагностика инструмента, не новая статистическая популяция или замена третьего legacy sample. Исторические TimeoutError/role-invalid FAIL и N2 остаются. Проверены точные PID/CLI SHA, стабильные зеркальные outer sockets, счётчики и process sampler. Fullhost/firewall/routes/resolver/services/stagedCLI восстановлены, собственные namespaces/root/remote archive удалены после SHA-проверки локального архива.

Изменены только измеритель/парсер/отчёт и тесты: Rust/INI/Cargo и 916 build/config inputs прежние, native digest прежний. Это отдельная квалификация исполняемого инструмента, не «Markdown-only» перенос результатов. Прежние 20 Linux certification executions сохраняют исходные даты/raw/CLI; они не используют изменённый bidir collector. Код патча F030/native consumer copies пока не перенесён; legacy QueueFull остаётся. Q36 IN_PROGRESS, 33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE.

## Проверенный расчёт слотов перенесён в исходники

Перенесён точный `client/mod.rs` из фактически собранного и проверенного кандидата: SHA256 `02690ef6b1666f87a3e70a4a2965bee0d9ebe154a0ed5cd4c7bd32124f922dc5`. Размер слота следует уже принятому пределу пакета: negotiated mux MTU+64, legacy MAX_TUNNEL_MTU16602; общий бюджет4MiB и резервирование очередей/привязка потоков прежние. Пакет сверх старого wire-предела не становится допустимым, значения INI/defaults/ABI не меняются.

Ранее исполненные15Rustтестов/4featurechecks,18fixed2mux окон и35пригодных compatibility окон относятся к этому точному файлу, но не доказывают идентичность всей текущей сборки. Свежая сборка текущего дерева, native A/B/exports/consumer checks и обновление применимости Linux certification **ожидаются**. Старые native binaries/сертификация сохраняют прежние SHA/источник; их нельзя объявлять пересобранными новым патчем. Остаточный legacy QueueFull/регрессия download и все исходные FAIL сохраняются. Q36 IN_PROGRESS,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.

## Native-перенос точного патча завершён локально

Текущее дерево собрано:4strictfeaturechecks,15Rustтестов и releaseCLI SHA256 `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`. Отдельная квалификация925inputs подтверждает завершённые этапы даже при поздней ошибке native-контроллера.

Сохранены Windows/macOS passA. При удалении собственного target-кэша `du` встретил исчезнувший файл и остановил первый контроллер; это отказ инструмента, не компиляции. Исходный FAIL/архив остаются. Новый счётчик размера допускает только исчезновение записи, сохраняет6GiBлимит и не скрывает другие ошибки.4Linuxрегрессии PASS. Первый запуск B был остановлен до компиляции из-за ожидания размера каталога после удаления; исправлено только тестовое ожидание, отказ сохранён. Выполнен только недостающий passB с независимыми source/cache/target после подтверждённой очистки старого root. Совпадение A/B побайтное. Одинаковый путь source/registry при новом объекте каталога сохраняет встроенные Cargo-пути; recipe/flags/epoch не менялись.

Windows/macOS:33CAPIexports наWindows и каждомmacOSslice,22реальныхWindowsABI/localhost-cancellation checks PASS. Android:ARM64/x64 независимыеA/B,33CAPI+21transportJNI+1ConfigJNI=55exports/ABI PASS.3новыхAPK собраны ипроверены, R8/signatures/native payload PASS;23JNI+2packagedINI вAPI34эмулеPASS.201неизменённыйJVMтест/lint подтверждены исходным XML/архивом;5Kotlin/XMLfiles отличаются толькоCRLF/LF, README — документация; новыхJVMexecutionsне заявлено. ОригинальныйAVD/SDK/userdata/сервисы/host/source сохранены; ownswap, roots, remote archives илокальнаяAPKprojection архивированы/SHAпроверены/удалены. Все исходныеархивы сохранены.

Четыре canonical libs и четыре consumer copies обновлены локально; checksum/provenance PASS, native source digest `0dfac80b50d2bdd1964817dee8ac87f7311cde63bf61efade7a40abd7e32b635`. PhysicalMac/iOS/router/WindowsnetworkVM не проверялись по принятому scope. StrictLinuxcertification всё ещё относится к старому дереву: **для Rust-изменения ожидается новая20-case матрица**, старые результаты не переобозначены. LegacyQueueFull/download-regression и role-invalid/TimeoutError остаются в отчёте; универсальногоzero-loss/performancePASS нет. Q36 IN_PROGRESS,33/37(89,2%),NO_PUSH_DEPLOY_RELEASE.

## 14 октября: свежая Linux-матрица нового ядра завершена

Все 20 обязательных критериев прошли на текущем CLI `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`: 19 самостоятельных запусков, 409 проверок и отдельный критерий утечек, выведенный из прошедших full/split сценариев. Проверены TCP, UDP fake-TLS, QUIC, внешние/внутренние IPv4/IPv6, split/full, настоящий systemd-resolved A/AAAA и forced TCP, MTU 1280/DATA_FRAG/PTB, TAP NDP/RA, legacy 0.7.16 в обоих направлениях и 100 смен сети для TCP плюс 100 для QUIC. Это ограниченная функциональная матрица; нового восьмичасового endurance-прогона не заявлено.

Это новый прогон после Rust-патча, а не перенос старого результата через изменение ядра. Исходный execution commit `55eb810865e290e427f6cd2b5838e4278aa304bf`, actual CLI build `e0ce217b630d9dfd7db5f59936bb41e6d68eb69d`; build inputs между ними совпадают. Все исходные логи и архивы проверены по SHA. Собственные root/archive/unit удалены; host/firewall/routes/DNS/accounts, рабочие процессы и staged CLI сохранены. Сертификация обновлена по свежим результатам; прежний manifest сохранён отдельно. Последующая применимость относится только к четырём Markdown-файлам этого отчёта/плана, даты и исходная идентичность прогона не меняются.

Обязательный долг функциональной проверки перенесённого Q36-F030 закрыт вместе с ранее выполненными native A/B, ABI/JNI и пакетными проверками. Legacy QueueFull, снижение legacy download на 6,63%, один bidir выброс и потери TUN остаются в результатах; общего zero-loss/performance PASS нет. Q36 остаётся IN_PROGRESS; Q29/Q31/Q34 также открыты. Итог 33/37 (89,2%), без push/deploy/release.

## 14 октября: новый current-product legacy/mux cohort

На текущем исходном commit `3a7e25dcdf1af6f3a761dfa883851df807535fb0` выполнены шесть новых сессий / 18 окон по 8 секунд: TCP raw P1, фиксированные два carrier, legacy `off` против recordizer `required`, по три парных повтора upload/download/bidir. Во втором повторе порядок протоколов и направлений обратный. Сервер и клиент во всех случаях — один уже собранный CLI `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`, actual build commit `e0ce217b630d9dfd7db5f59936bb41e6d68eb69d`. Полное Git-дерево `qeli/` совпало с build commit; 919 текущих input и точные measurement helpers сохранены. Новых компиляций, изменения MTU/defaults/очередей/qdisc/pacing не было.

Медианы трёх сессий; native/TUN drops ниже — суммы разностей счётчиков между границами вокруг девяти окон каждого протокола, не точное соответствие отдельным пакетам iperf.

|Protocol|Upload Mbps|Download Mbps|Bidir up/down Mbps|Native uplink/downlink drops|TUN TX drops client/server|
|---|---:|---:|---:|---:|---:|
|Legacy|583.38|709.27|105.73/666.36|43676/0|0/0|
|Recordizer|1214.62|1448.40|490.63/1166.87|0/0|1723/1684|

Recordizer относительно legacy: медиана парного изменения upload +108.20%, download +104.21%, bidir sum +113.10%. Все три samples, min/max/spread, direction tradeoffs, CPU/RSS/steal и счётчики сохранены. Наблюдавшийся максимальный RSS worker 95020 KiB. Это конечные same-VM окна двух sibling namespaces, не WAN/physical capacity или общий performance PASS.

Legacy native uplink потери (43676) во всех наблюдённых границах относятся к QueueFull; прочие uplink-причины и native downlink потери нулевые. При recordizer native uplink/downlink drops нулевые во всех девяти окнах, но потери TUN остаются в таблице. Это подтверждает оставшееся давление legacy-очереди на текущем продукте, не утечку pool и не исправление всех причин потерь. Допустимый legacy packet bound не сужен, очередь и контракт pinned-flow/nonblocking drop не менялись.

Все шесть bidir-окон допущены по противоположным TX/RX ролям и зеркальным endpoint tuples client/server output. На 42 границах сохранены два действительных native-PID carrier sockets; поколение VPN не менялось в пределах сессии. Сверены raw iperf summaries, interval sender bytes/retransmits и TCP_INFO RTT; reverse-only клиент не является отправителем, его sender RTT остаётся неизвестным. Различия финального и интервального byte horizon сохранены без исправления исходных JSON. Счётчики показывают наблюдённые границы потерь, а не точную причину каждого kernel drop.

Это новая популяция текущего продукта. Она не дописывается к историческому candidate/baseline набору, не заменяет role-invalid окно и не измеряет before/after эффект F030. Историческое снижение legacy download на 6,63% и N2 bidir остаются со своими исходными бинарниками и датами. Первая подготовка отказала до лабы из-за смешения whole-worker и отдельно квалифицированного CLI статусов; отказ сохранён. Следующий интерактивный контроллер исчез до завершения: одно окно сохранено отдельно, его ресурсы архивированы и очищены, host/staged CLI восстановлены. Финальный контроллер запускался скрыто вне интерактивного job; пароль передан только через stdin.

Фактический прогон занял 514.06 секунды при measurement budget 900 секунд. Полный host/firewall/routes/resolver/services и staged CLI сохранены. Собственные процессы/namespaces/root и удалённый архив удалены после проверки inode, PID, SHA и полного archive inventory. Оригинальный полный архив и raw сохранены. Долг актуального legacy/mux измерения закрыт в этом объёме; отсутствие всех потерь или закрытие Q36 не заявлено. Q29/Q31/Q34/Q36 открыты, 33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE. Evidence: `release/certification/evidence/q36-current-legacy-mux-20261014.json`.

## 14 октября: контракт legacy-очереди и границы счётчиков

Выполнена отдельная offline компонентная проверка на Linux x86_64, Rust 1.97.0: точные тела `bonded_uplink_pool_layout`, `prepare_bonded_uplink`, `queue_tcp_uplink`, выбора carrier и связанных типов извлечены из текущего исходника по SHA; BufferPool и UplinkCounters включены побайтно. Используются настоящие Tokio mpsc/Semaphore и PooledBuffer. Harness содержит отдельный Vec alias для неиспользуемого Tun-варианта и только структуру RuntimeConfig; все probe-пакеты идут через Bonded. Это проверка этих компонентов, не компиляция всего Qeli и не TUN/driver/crypto/wire runtime.

11 положительных тестов PASS: восемь существующих BufferPool, один UplinkCounters и два новых legacy probe. При 4 MiB storage budget / 16 602 байта получается 252 слота, а резерв для двух carriers/candidate оставляет 83 записи на pinned очередь. Контролируемый недренируемый burst из 128 пакетов по 1400/40 байт сохраняет первые 83 в FIFO и регистрирует ровно 45 QueueFull; PoolExhausted/Oversize/NoCarrier равны нулю, другой carrier не используется для перепривязки потока. После дренирования все 252 слота снова доступны. Это отсутствие потери pool-слотов на проверенном пути, не обещание отсутствия любых гонок или потерь при любой нагрузке.

Пакет из 16 602 байт побайтно помещается в PooledBuffer; 16 603 байта отклоняется как Oversize с возвратом слота. Намеренно уменьшенный до 2 KiB legacy-слот проваливает этот тест; таким сокращением нельзя исправлять QueueFull без изменения принятого контракта. Второй отрицательный контроль намеренно забывает буфер при Full: тест обнаруживает недостающие слоты после дренирования. Восстановленный исходник снова проходит обе проверки. Отрицательные изменения существовали только в собственных probe-проекциях, код продукта не менялся.

Уточнение интерпретации предыдущих измерений: `tcp_uplink_drops` — счётчики ограниченной передачи в TCP-очередь (pool, размер копии, Full, отсутствие carrier). Они не учитывают все отказы после dequeue, например отклонение recordizer или ошибку encryption/write. Поэтому ноль этого счётчика означает ноль зарегистрированных потерь в его области, а не полную доставку всех пакетов native data plane. Доставка, TUN drops и retransmits проверяются отдельно. В текущем legacy/mux отчёте остаются фактические значения и полный raw; никаких дополнительных нулей или performance PASS не добавлено.

Зависимости audit-harness взяты из 21 cached crate archive, каждый SHA совпал с каноническим Cargo.lock; реально скомпилированы 12 registry packages. Сеть не использовалась, jobs=1, приватные vendor/Cargo home/target, MemoryMax 768 MiB/SwapMax 256 MiB/CPUQuota 100%/runtime 900 секунд. Успешные 11 тестов и контроль размера не повторялись после отказа CRLF-подстановки второго контроля: выполнены только недостающий leak control и две восстановленные проверки. Исходный общий FAIL сохранён, успешные шаги двух идентичных по source/lock/toolchain пакетов допущены отдельно. Более ранний preflight отказал до создания root из-за отсутствующего staged-пути другого хоста; отсутствие пути сверено до/после на фактическом стенде.

Собственные проекты/target/vendor/cache/unit/remote archives очищены после SHA/inode/process guards; host/services/firewall/routes/resolver и глобальные crate archives сохранены. Полные локальные source/log archives остаются. Новые native/CLI сборки и VPN runtime не выполнялись, действующая сертификация их не переименована. Долг проверки legacy burst/accepted bound закрыт в этом компонентном объёме. Оставшиеся legacy pressure, TUN loss и исторические direction tradeoffs не объявлены исправленными. Q36 IN_PROGRESS; 33/37 (89,2%), NO_PUSH_DEPLOY_RELEASE. Evidence: `release/certification/evidence/q36-legacy-contract-20261014.json`.

## 14 октября: F031/F032 — старые сценарии и независимость фаз

Q36-F031: шесть perf_* scratch-сценариев не имели рабочего admission guard. Executable callers в tracked code/docs/workflows не обнаружены; ссылки есть только в исторических SHA inventories. Все шесть старых файлов с fixture-only переменными дошли до Paramiko constructor в контролируемой проверке; подставной constructor остановил исполнение BaseException до SSH/сокетов. Четыре сценария содержали host-wide process/firewall/network orchestration; ещё два выполняли старую production-диагностику. Реальные подключения и команды не выполнялись, встроенные значения профилей/реквизитов не публиковались в evidence.

Удалены718 строк устаревших реализаций perf_allmodes_prod, perf_ab_p3b1, perf_netns_cpu, perf_modes_test, perf_prod_live_monitor и perf_facts. Сохранены только три строки явного RETIRED отказа с указанием benchmark_isolated_lab.py, чтобы старый путь не превращался в молчаливый другой замер. Доступные credentials и прежний --allow-host-wide-benchmark не возобновляют эти сценарии. Активный isolated runner и guarded benchmark.py/bench_bonding.py прежние.

Q36-F032: aggregate проверял совпадение целой raw-популяции, поэтому одна скопированная фаза среди разных остальных фаз допускалась как независимый повтор. Отрицательный baseline получил tcp_up n3/median200 из двух разных raw измерений, когда одна фаза первого прогона была скопирована во второй и её summary согласован с копией. Исправленный guard проверяет SHA каждого одинакового mode/phase отдельно и отвергает повтор в AGGREGATED и MEASUREMENTS_ONLY. Одинаковые Mbps с разными raw-файлами остаются допустимыми; outliers и исходные результаты не исключаются.

На Windows и Linux по22 метода PASS:3 метода/14 subprocess invocations/6 imports защищают retired/guarded сценарии,19 report методов включают отдельные TCP/UDP phase copies и equal-rate distinct raws. Предыдущие ложные допуски и SSH constructor paths сохранены как baseline, не переписаны в PASS. Exact новый guard дополнительно применён к SHA18 raw-файлов ранее квалифицированных шести current legacy/mux sessions: по3 повтора off/required, дубликаты одинаковой фазы отсутствуют. Подставленная только в памяти копия одной настоящей phase identity отвергнута. Полный aggregate этих current diagnostic sessions не повторялся, throughput/statistics/raw байты не изменены.

Используемые прежним runtime population и sample_policy побайтно неизменны на уровне функций, изменён только aggregate admission. Не было новых Rust/native/CLI/VPN сборок или нагрузочных измерений. Оба own lab roots/archives удалены после SHA/process/inode guards; host/service/firewall/routes/resolver сохранены. Методическая ошибка и dead-code perimeter закрыты в этом объёме; legacy queue pressure, TUN losses/стадия TCP-асимметрии и полное Q36 acceptance остаются открытыми.33/37(89,2%),NO_PUSH_DEPLOY_RELEASE. Evidence: `release/certification/evidence/q36-perf-methodology-20261014.json`; raw: q36-retired-perf-scenarios-20261014 и q36-benchmark-phase-independence-20261014.

## 14 октября: полная статистическая матрица текущего CLI после F030

36 новых сессий /144 окна по8с: все12 транспортов, outer4/inner4, TCP/UDP bidir P1/P4, по3 независимых повтора. Сервер и клиент используют уже квалифицированный CLI6f617bdcab5a; actual build e0ce217b630d, полный Git qeli-tree совпадает с execution commit eacd0da0833a. Новых компиляций нет. Это отдельная популяция; прежние полные матрицы, legacy/mux и до/после F030 не объединены с ней. Во втором повторе порядок профилей обратный. Порядок фаз фактически одинаковый во всех повторах: добавленное разворачивание в обвязке повторило уже существующее и отменило его эффект. Исходный driver и фактический порядок сохранены; результаты не исключены, снижения зависимости от порядка фаз не заявляется. TCP producer без лимита, UDP100Mbps на stream/направление (P1=100+100, P4=400+400); MTU1400/1500, штатные исходные профили benchmark, без смены qdisc/affinity/defaults. Это sibling namespaces одной2vCPU VM, не межсерверная capacity.

Медианы трёх сессий; все48 групп, samples/min/max/spread, CPU/RSS/steal, receiver raw и счётчики session drops/UDP kernel RcvbufErrors сохранены без исключения выбросов. Эта матрица не измеряет точную стадию TUN TX drop или все native uplink handoff counters.

|Mode|TCP P1 up/down Mbps|TCP P4 up/down Mbps|UDP P4 loss up/down %|
|---|---:|---:|---:|
|tcp-faketls|192.18/521.44|151.11/545.60|43.70/33.39|
|tcp-frag|160.18/575.82|160.05/573.85|42.93/32.75|
|tcp-obfs|114.11/568.31|138.89/528.06|57.34/39.97|
|tcp-obfs-awg|168.25/503.30|125.45/553.62|54.52/39.88|
|tcp-padding|158.43/551.21|144.36/581.40|40.85/36.79|
|tcp-plain-raw|170.97/544.34|129.00/579.62|49.23/29.00|
|tcp-reality|154.08/586.74|150.94/504.39|48.47/30.60|
|tcp-reality-tls|254.34/663.98|218.77/654.62|35.48/7.46|
|udp-faketls|226.96/586.24|216.53/557.00|40.28/53.25|
|udp-faketls-awg|213.58/554.94|196.71/542.58|41.53/46.46|
|udp-padding|197.61/532.04|203.05/543.19|43.85/50.11|
|udp-quic|233.39/551.26|215.79/544.58|43.79/52.88|


Все72 TCP endpoint-role проверки, независимость144 phase raw, source/CLI/process/generation/MTU/config/counter admissions, полный host gate и SHA-инвентаризация/очистка собственного root/archive подтверждены. Ранее выполненные текущие Linux20 и native/ABI/JNI проверки сохраняют исходные факты; этот прогон добавляет измерения, не заменяет их. Первая попытка отказала до измерений из-за SUN_LEN; старый archive guard отказал на новом префиксе. Исходный FAIL сохранён, отдельная recovery архивировала и удалила root/archive с неизменными host/staged CLI. R2 сокращает root/cell labels, заранее проверяет длину socket path и допускает только точный новый префикс.

Критерии Q36 сопоставлены с прежними receipts: методическая review/dead-code работа F001–F032, негативные admissions, owning-module queue bounds/faults, роли/конкуренция и F030/F031/F032 ретесты имеют evidence. Current outer4/inner4 all-mode statistics теперь выполнены; прежние IPv6-популяции остаются историческими измерениями своих бинарников. Источник kernel TUN losses и точная стадия TCP-асимметрии не установлены; общего zero-loss/performance PASS и закрытия Q36 нет.33/37(89,2%). Evidence: `release/certification/evidence/q36-current-slot-statistics-20261014.json`; полный packet: q36-current-slot-statistics-r2-20261014. NO_PUSH_DEPLOY_RELEASE.
