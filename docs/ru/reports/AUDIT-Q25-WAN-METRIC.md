# Q25-F126: выбор WAN по метрике default route

Дата: 25 сентября 2026. База: `f35631c6`. D06 остаётся **IN_PROGRESS**.

`gateway/wan.rs` брал первый `dev` из `ip route show default`. При нескольких
маршрутах это зависело от порядка вывода и могло поставить MARK/NAT/FORWARD
exit-node на запасном WAN. [ip-route(8)](https://man7.org/linux/man-pages/man8/ip-route.8.html)
определяет меньшую метрику как более предпочтительную. Теперь парсер выбирает
минимальную метрику среди `default` с `dev`; отсутствие `metric` означает ноль,
а некорректная метрика отбрасывает запись. IPv4 и IPv6 используют один алгоритм;
`route get` остаётся fallback при отсутствии пригодной default route.

Регрессии проверяют обратный порядок вывода, отсутствие и повреждение метрики.
В изолированной packet-лабе `.11` ядро выбрало WAN с метрикой 50 вместо WAN с
600. Правила только для WAN с 600 блокировали пакет guard-правилом (счётчик 1).
После ручного добавления MARK/NAT/FORWARD для выбранного WAN приёмник увидел
NAT-адрес `192.0.2.2`, не адрес клиента `10.0.0.2`. Скрипт и логи:
`C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260925/tun-attach-context-phase/wan-metric-packet.sh`,
`wanmetricpacket.log`, `wanmetric.log`. Проверка прошла в частных network/mount/PID
namespaces; установленные сервисы не менялись.


Проверки кода на `.11`: 7/7 адресных WAN-тестов, 113/113 gateway-тестов
и `cargo fmt --check` — PASS. Двуязычный `scripts/check_docs.py`: 9/9 PASS.

Остаток D06: `refresh_exit_paths_if_active` вызывается при VPN path COMMIT.
Linux physical-path sampler наблюдает интерфейс до VPN-сервера. Если он остаётся
прежним, а отдельный exit WAN переключается, COMMIT может не произойти: guard
блокирует трафик до следующей смены пути или перезапуска профиля. Требуется
отдельный монитор exit WAN, согласованный с NetworkPlan owner и teardown.
Rename/name-reuse — граница [Q25-A125](AUDIT-Q25-WAN-NAME-REUSE.md);
policy routing — [Q25-F124](AUDIT-Q25-EXIT-POLICY-ROUTING.md).
