# Q29: актуальные VPN-интеграции и надёжность Android-стенда

<!-- normative-sync: q29-current-vpn-v4 -->

12 октября2026. Q29 IN_PROGRESS. План32/37 (86,5%),5 разделов открыты.

Текущее ядро проверено на Android14/API34 x86_64: семь matching-R8 instrumented
VPN-интеграций PASS. Использованы CLI52511ceb… и APK77c12051… с JNI01da54a3…;
126 канонических Android build inputs совпадают. Target не debuggable, native
bytes совпадают с ранее квалифицированной текущей сборкой. JVM/native A/B не
повторялись; новых больших compiler caches нет.

| Сценарий | Свежий результат |
| --- | --- |
| TCP, UDP fake-TLS, QUIC × split/full | 6 actual instrumented PASS |
| IPv4/IPv6 × TCP/UDP sockets | 72 независимых server echo receipts, точные size/SHA; TCP16384B, UDP32/257/1024B |
| System DNS через VPN | 6 уникальных имён, A/AAAA question/answer receipts с TUN source |
| Full kill-switch без системного lockdown | 1 actual refusal PASS; не доказательство включённого lockdown |
| Конечные service/TUN и AndroidRuntime | Отсутствуют service/TUN; нет FATAL EXCEPTION |
| Память гостя | Запрошено1536MiB с явным lowram, MemTotal=1505648KiB |

Время финального gate: 130.93s. Сокеты обычные, без process binding
и без явного Network.bindSocket в routed cases, но source/readiness preflight
сохранён. Instrumentation работает в контексте целевого приложения: это не
квалификация отдельного non-VPN-owner UID и не проверка самого первого сокета
после CONNECTED. Профиль создаёт instrumentation, а не UI/SAF. Это подписанная
R8-пара со специальными правилами совместимого runner; ordinary production-rules
APK, arm64/OEM, long physical sessions не квалифицированы этим запуском.

## Исправленная обвязка

1. Эмулятор36.6.11 повышал запрошенные768MiB до2560MB. На VM2GiB RAM сначала
   исчерпался own cgroup1400MiB RAM/512MiB swap, затем1400MiB/1536MiB: до APK,
   затем при UI-импорте. Теперь1536MiB/explicit `-lowram` и обязательный guest
   MemTotal guard до установки APK. Четыре адресных негативных/положительных
   метода PASS. [AOSP](https://android.googlesource.com/platform/external/qemu/+/emu-master-dev/android/android-emu/android/main-common.c) объясняет снятие нижней RAM-границы; эта ветка не
   доказана как точный исходник установленного build.
2. 24-sample cold burst закончился за0,854s до CONNECTED. После исходной серии
   теперь ограниченно ожидается CONNECTED и выполняется существующий fresh
   follow-up. Все наблюдавшиеся post-CONNECTED сокеты обоих серий должны пройти;
   отсутствие cold coverage сохраняется отдельно, не получает непрерывный PASS.
   Два новых метода проверяют late publication и отказ при failed follow-up.
3. Profiles tap не завершил переход, а helper сразу объявлял Import отсутствующим.
   До30s/5 fresh observations подтверждают кнопку; повторный tap только для
   unselected Profiles. Три метода отвергают selected-only ложный PASS и покрывают
   отложенный переход.
4. Системный ANR при Always-on принимали за неизвестное VPN confirmation.
   Общий UI helper сохраняет отдельный исходный XML, выбирает только системный
   Wait и требует свежий обычный экран за максимум3 попытки. Два метода проверяют
   preservation и отказ при повторяющемся ANR.

Вместе с прежними DNS/startup fixtures:23 helper methods PASS. Memory guard
исполнен в текущей Android-лабе; целиком исправленный cold UI сценарий **не прошёл**.
Последняя cold попытка остановилась при unpublished `/sdcard`: INI push получил
remote secure_mkdirs failed. Предыдущие ANR/UI/timing отказы сохранены. В одном
из них до FAIL получены реальные VPN replies, но это не полный acceptance PASS.
Седьмой routed gate имеет отдельный успешный scope и не закрывает эти отказы.

Старые system/data описания исправлены:768MiB означали запрос, а не измеренную
RAM. Сохранённые data/runtime-final, restart-control/r4 и resolver-diagnostic
логи сообщают2560MB. Исходные пакеты, даты и результаты не переписаны.

## Остаток

SIGKILL automatic recovery ранее FAIL также в минимальном независимом TUN
control; этот шаг его не повторял и не закрывает. Generic auto/null DnsResolver
ENONET сохранён; обычный system lookup в routed gate не является той же API-
операцией. Текущий cold UI startup/lockdown/recovery gate остаётся FAIL/непокрытым;
не следует продолжать одинаковые cold retries без admission внешнего storage и
достаточной стабильности системы. Lowram не принят за стандартную физическую
платформу. Исторические положительные матрицы сохраняют свои исходные inputs;
USER_SKIPPED/D06 unchanged. Q29 и весь аудит не завершены.

Все7 попыток сохранили host firewall/routes/resolver, production service и original
AVD userdata. Семь own roots удалены после SHA-верифицированных полных recovery
архивов, отсутствия процессов/fd/mount и сохранения host snapshot. Raw:
`audit-debt-20260924/q29-current-vpn-20261012`, `q29-current-vpn-r2-20261012`…
`q29-current-vpn-r7-20261012`. [Scoped evidence](../../../release/certification/evidence/q29-current-vpn-20261012.json).
`NO_PUSH_DEPLOY_RELEASE`.


## 12 октября: cold UI и независимый UID после storage admission

Новый исправленный прогон current matching-R8 пары прошёл за256,91s.
Приложение не debuggable; INI импортирован реальным UI/SAF, instrumentation
и private prefs injection не запускались.126 канонических Android build inputs,
CLI52511ceb…, APK77c12051… и native01da54a3… те же квалифицированные текущие
байты. Ordinary production-rules APK и физические устройства остаются вне scope.

После boot_completed `/sdcard` реально отсутствовал: readlink вёл к
`/storage/self/primary`, df сообщал No such file. Наблюдался и один10s ADB timeout.
Через16,995s обнаружены настоящий `/dev/fuse` на `/storage/emulated`,
`/storage/emulated/0` и shell write access. Это наблюдение одной загрузки,
не универсальное время готовности. Теперь общий helper ждёт максимум60s
с сохранением каждого состояния до APK install и непосредственно перед INI push.
`Download` создаётся только после допуска; после push SHA256 файла сверяется
до открытия file picker. Простое наличие каталога без FUSE не допускается.
[AOSP](https://source.android.com/docs/core/storage/scoped) описывает FUSE для
современного emulated storage; точные path/filesystem guards относятся к этому API34 AVD.

Пять новых storage методов и три UI transition метода PASS без skips.
Негативные случаи: underlying directory вместо mount, wrong user/ambiguous path,
transport failure, истечение общего budget без его продления, timeout и unknown
phase. Зависимый cold сценарий выполнен один раз после исправления; прежние
шесть FAIL сохранены, не переписаны. Guest MemTotal1505652KiB при запросе1536MiB
в пределах прежнего допуска с kernel reservations. Первоначальный local qualifier
ошибочно требовал старое значение1505648KiB; разница4KiB не является product FAIL.
Исправлена только проверка метаданных, runtime не повторялся.

| Обычный UID10149, VPN owner10148 | Свежие пробы / ответы |
| --- | --- |
| Physical baseline |48/48|
| Cold lockdown start |96/64|
| Startup follow-up |48/48|
| Connected steady |48/48|
| Переход force-stop |48/23 — ответы до завершения перехода|
| Установившийся stopped lockdown |48/0 — все блокированы|

Всего336 burst probes,231 успешный ответ и10 отдельных успешных сокетов.
Все241 server echo receipts независимо сопоставлены по protocol/family/payloadSHA
с журналом non-VPN-owner UID. IPv4/IPv6 TCP16384B и UDP257B при connected/manual
recovery прошли. Физический baseline и ответы во время переходов не называются
VPN leakage. После установившегося stop нет ни socket replies, ни DNS answers.

После Native NetworkPlan APPLIED первые сокеты ещё получили EACCES/Pending
connect failure: исходный FAIL_POST_PLAN_BLOCKING сохранён. Android CONNECTED
опубликован через535ms после APPLIED. Все80 наблюдавшихся post-CONNECTED сокетов
успешны:32 из cold burst и48 из follow-up, все четыре family/protocol сочетания.
Coverage=COLD_AND_FOLLOWUP; конечные samples не доказывают непрерывное отсутствие
потерь или самый первый возможный сокет в момент события.

Семь DNS cases PASS: raw physical baseline; system/raw connected; обе blocked
проверки отказа; system/raw после ручного восстановления. Generic DnsResolver
с явным active VPN теперь получает A+AAAA; typed A/AAAA с null и active network
также успешны. Generic `network=null` по-прежнему ENONET при действующем VPN.
Диагностика не превращает этот отказ в общий DNS PASS.

Current independent-UID cold UI/startup/lockdown/force-stop/manual recovery gate
закрыт в описанном scope. Автоматическая SIGKILL recovery — другая операция и
остаётся историческим FAIL, здесь не выполнялась. Q29 IN_PROGRESS;32/37(86,5%).
Userdata оригинального readonly AVD, PID777/845/service и все host firewall
backends/routes/resolver сохранены. Результаты SHA-qualified; собственный root
удалён после verified recovery и inode/path/process/fd/mount guards.
Новые Rust/JVM/native builds и большие compiler caches отсутствуют.
[Evidence](../../../release/certification/evidence/q29-storage-startup-20261012.json).
Raw: `audit-debt-20260924/q29-storage-startup-20261012`. NO_PUSH_DEPLOY_RELEASE.


## 12 октября: обычный production R8 APK, отдельный UID

Штатный `assembleRelease` APK прошёл runtime за 252.94s на Android14/API34/x86_64. В этой сборке нет `instrumentation-abi.pro` и `qeliTestBuildType=release`. Повторных Gradle/Rust/JVM сборок не было: использован уже собранный production-unsigned APK `ec05ed9330ca3db7cca6f296569e68b24f172f5c9d2a5d65667e80e717da288c`. Его подписали Android Debug сертификатом для установки в изолированном AVD; проверены подпись, 16KiB ZIP alignment и совпадение всех 1091 несигнатурных ZIP entries, включая DEX/resources/manifest и обе JNI. Подписанный SHA256 `3d8496b42251fd90109a4b87532acf8a1c8b16a1a07d85267ce6c08b2db7f2a7`. Настоящая release-подпись, физические/OEM/arm64 runtime этим не проверяются.

Приложение не debuggable. INI импортирован через UI/SAF; instrumentation и private prefs injection не запускались. Test APK использован только как standalone broadcast receiver с отдельным UID 10149, VPN owner UID 10148. Обычные TCP/UDP сокеты receiver не bind/protect; explicit Network binding применяется только в отдельно помеченной диагностике. Все126 канонических Android build inputs неизменны. JNI x86_64 `01da54a3d1b24fdbfc4d305697df582a6b8bec2e04985b0c7ce64a6277d0ea82`; arm64 байты квалифицированы сборкой, выполнение arm64 не заявляется.

| Фаза | Пробы / ответы |
| --- | --- |
| physical-baseline | 48/48 |
| cold-lockdown-start | 96/68 |
| startup-followup | 48/48 |
| connected-steady | 48/48 |
| force-stop | 48/8 |
| stopped-lockdown | 48/0 |

Все 230 echo receipts независимо сопоставлены с успешными ответами отдельного UID по protocol/family/payloadSHA: 220 burst replies и10 отдельных сокетов. При установившемся stopped lockdown все48 сокетов блокированы, DNS answers отсутствуют. Ручное force-stop/re-enable восстановление IPv4/IPv6 TCP/UDP и system/raw DNS прошло. Ответы до завершения перехода force-stop не считаются утечкой.

После Android CONNECTED успешны 80/80 наблюдавшихся сокетов: 32 cold и 48 follow-up, coverage=COLD_AND_FOLLOWUP. Это конечная выборка. Первые sampled сокеты после Native NetworkPlan APPLIED: `FAIL_POST_PLAN_BLOCKING`; исторические отказы сохраняются. Generic DnsResolver `network=null` по-прежнему даёт ENONET; explicit active VPN и typed A/AAAA null/active успешны. Автоматическая SIGKILL recovery здесь не выполнялась и остаётся историческим FAIL.

Обычный production R8 runtime gate закрыт в этом scope; это дополнение к прежним matching-R8 результатам. Q29 IN_PROGRESS;32/37(86,5%), пять разделов открыты. Original AVD userdata, рабочий service/PID, host firewall/routes/resolver сохранены. Собственный runtime root удалён после SHA-qualified backup и inode/path/process/fd/mount checks. [Evidence](../../../release/certification/evidence/q29-production-runtime-20261012.json). Raw:`audit-debt-20260924/q29-production-runtime-20261012`. NO_PUSH_DEPLOY_RELEASE.


<!-- q29-dns-selection-control-20261012 -->

**12 октября:** Q29: причина generic/null ENONET локализована на текущем API34 image. Ответ dnsproxyd и отдельный platform TUN с другим owner UID воспроизводят отказ; bypassable control получает A+AAAA. Typed/null и explicit VPN DNS идут через TUN. Product APK/JNI/default kill-switch неизменны; compatibility FAIL сохранён, SIGKILL и first-post-APPLIED открыты. [DNS comparison](AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md).32/37(86.5%).NO_PUSH_DEPLOY_RELEASE.
