# Q34/Q25 — память при 100 release handover

<!-- normative-sync: audit-q34-release-soak-v1 -->

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
