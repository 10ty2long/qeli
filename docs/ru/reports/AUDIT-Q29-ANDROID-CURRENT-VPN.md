# Q29: актуальные VPN-интеграции и надёжность Android-стенда

<!-- normative-sync: q29-current-vpn-v1 -->

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
