# Q34/Q25 — память при 100 release handover

<!-- normative-sync: audit-q34-release-soak-v2 -->
Исторический checkpoint24сентября; статусы долгов ниже относятся к той дате. Текущий [реестр](../plans/AUDIT-DEBT.md) учитывает последующие закрытия; новый Q35 scope приведён в конце.


24 сентября 2026. Rust 1.97.0, Linux x86_64, `release --features jemalloc`;
исходники ядра соответствуют `ea87fd49`. Это developer integration, а не итоговая
сертификация 0.8.2, benchmark пропускной способности или A/B reproducibility.

## Измерения

По 100 переключений TCP fake-tls и UDP quic-shape, sample каждые 10 переключений.
Оба сценария: **15/15 утверждений PASS**, всего **30/30**; exit codes `0/0`.
Сохранены одна аутентифицированная сессия, исходные процессы и TUN, связность,
ровно 100 COMMIT клиента/сервера, один точный carrier bypass.

| Транспорт / процесс | RSS baseline → final, KiB | Прирост final / sampled peak, KiB | fd baseline → final | socket fd baseline → final |
|---|---:|---:|---:|---:|
| TCP client | 47736 → 49496 | 1760 / 1904 | 15 → 15 | 5 → 5 |
| TCP server | 53644 → 56036 | 2392 / 2392 | 18 → 18 | 6 → 6 |
| UDP client | 49592 → 50292 | 700 / 3764 | 15 → 16 | 5 → 6 |
| UDP server | 61668 → 74248 | 12580 / 12580 | 18 → 18 | 6 → 6 |

В UDP один дополнительный клиентский socket остаётся стабильным от первого sample
до конца; это не нулевой прирост. Pending candidate=0, CID aliases=3 после handover;
TCP orphan=0. Прежний критерий роста RSS **32768 KiB не изменён**.
Скрипты теперь печатают baseline, final и sampled peak delta: исходные данные
для расчёта сохраняются. Bash syntax и 6 Python harness tests PASS.

[Предыдущий debug FAIL](AUDIT-Q34-LINUX-MATRIX.md) остаётся в evidence.
У debug и этого release отличаются allocator/build profile и ревизия исходников;
результат не доказывает единственную причину прежнего роста и не переписывает
его в PASS. Он подтверждает критерий именно для измеренного release executable.

## Воспроизводимость и оставшийся объём

Worker SHA256: `93ee5dce2fc0e1066d7a1c5943b02dfaca608c1bee67323cabd6e9b52dc3b027`.
Source archive: `4ac2dbae65e440b872ff4a15222e12e60927c238d955ce02872fd448ff12d3a7`
(320 файлов; gateway fix включён, следующий route deadline fix ещё не включён).
Лаба `.11`, приватные net/mount/PID namespaces; сервер `.10` не изменён.

Evidence: `C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/`:
`release-gateway.log`, `roaming-release-soak-100/` с tcp/udp logs, rc, рабочими
сценариями и worker SHA; `packet-matrix-phase/roaming-scripts-measured-sha256.json`,
`harness-checks/` и `release-soak-phase/evidence.json`.

D13 остаётся IN_PROGRESS: нужны stop/reconnect и отказные/многопрофильные сценарии,
снимки threads/tasks, routes/firewall/journals до/после и итоговый снимок исходников.
D11/D12/D14 также не закрыты. Windows VM/Mac/iOS/router runtime пропущен по решению
пользователя. [Реестр](../plans/AUDIT-DEBT.md).

## Q35: допуск soak-прогонов, 10 октября 2026

F506 / Q35-F003: проверка положительных целых принимала00/000/08/0100
и произвольно большие строки цифр. Bash по-разному трактует такие значения в
test и арифметике. Точный прежний TCP helper с iterations=00 пропустил цикл и
прошёл8финальных resource checks. В fixture использован настоящий /proc текущего
Bash, контролируемые ip/control-counter/worker-selector ответы; ни одного
реального VPN-переключения этот baseline не утверждает. Нулевой цикл и его
ложный допуск воспроизведены исполнением исходного helper, а не копией условия.

Общий scripts/roaming_soak_parameters.sh теперь требует канонические десятичные
целые1..2147483647, без ведущих нулей, знаков, whitespace и overflow; sample_every
не превышает iterations. TCP/UDP helpers и три wrappers используют одну проверку.
Оба основных netns runners валидируют soak-параметры до mktemp, cleanup и
первой сетевой команды. Standalone helpers также проверяют параметры; отсутствие
общего helper вызывает exit/return2. Дефолты не менялись.

F507 / Q35-F004: прежняя UDP matrix с17QUIC/3adapter switches выдавала полный
ROAMING_UDP_RESOURCE_SOAK_MATRIX_PASS; общий endurance wrapper с100TCP также
выдавал TCP_10K_PASS. Это воспроизведено настоящими прежними Bash wrappers с
контролируемыми успешными child receipts; полноценная VPN-работа не заявлена.
Теперь standalone UDP matrix печатает qualification, фактические iteration/sample
counts и *_SMOKE при уменьшении QUIC ниже10000 или adapters ниже1000.
Успешный smoke по-прежнему завершает standalone wrapper с exit0, но не выдаёт
*_PASS marker. При достаточных counts и успешных children остаются прежние
endurance PASS markers. Ошибка child останавливает следующие modes.

roaming_resource_release_gate.sh — полный endurance gate: до первой фазы
отклоняет TCP/QUIC<10000, adapters<1000 и неверную частоту sampling. Он не
принимает укороченный smoke за полный тест. Отдельный обязательный
linux_roaming_release_soak.sh сохраняет minimum100 для TCP и UDP QUIC,
вычисление sample_every=iterations/10 и прежние строгие assertions; лишние
CLI аргументы отклоняются, путь executable с пробелами корректно quoted при
version probe. Таким образом, исправление не вводит10k требование вместо
согласованного короткого release gate.

28Linux методов без skips:14новых admission,4UDP wrapper,4endurance wrapper,
4worker identity и2control stats. Реальные Bash/Python/proc/filesystems;
network/child/counter seams контролируемые. Проверены минимумы/границы/overflow,
до-сетевой отказ parent/helper, missing helper, smoke/full labels, bounded100
dispatch, failure stops и прежние SHA/order guards.57Windows методов:
8IPv6 matrix contracts,38native recipes,11workflow checks; shell suites не
названы Windows PASS. Pinned actionlint принял4workflow, новые3suites включены
в CI; Bash syntax всех перенесённых scripts PASS.

Лаба.11: свой временный root, read-back source TAR и20inputs побайтно/SHA;
21known files удалены после exact-set/SHA/process/fd/mount guards. Service777/845
и CLI75bc сохранены, .10не использовалась. Первый генератор патча имел
IndentationError до записи/подключения; ошибка исправлена. Длительный100/10k,
30–60min/8h VPN-soak, peak между samples, task churn и актуальное hosted CI
этим пакетом не выполнены. Предыдущие core/native/runtime результаты имеют
прежний scope; ядро,14native,e3fb digest,protected/OpenWrt cut неизменны.
Q34/Q35 IN_PROGRESS,Q36TODO;30/37DONE81,1%,7остаётся. Q29SIGKILL/ENONET,
Q30/Q31,F459,D06accepted/USER_EXCLUDED сохранены.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F506-F507 evidence: q35-soak-admission-20261010; SEALED 38 files / 1646848 bytes; manifest SHA256 c3feba0ad1c0e010693765f4142a3011a88ac17afa0ef13f9089b38166109f75.
