# Q25-F216: единый bootstrap INI для Linux CLI

4 октября 2026. Q25 остаётся **IN_PROGRESS**; это проверенное исправление границы запуска, а не закрытие всего раздела.

<!-- normative-sync: audit-q25-cli-bootstrap-v1 -->

Отдельный `qeli-client` читал INI до запуска общего загрузчика через неограниченный `read_to_string`: FIFO без writer останавливал процесс, а недоверенный конфиг мог выбрать файл лога. `check-config --client` также обходил ограниченное чтение. Полный `qeli client` использовал серверный bootstrap с лимитом 16 МиБ и отдельным частичным разбором секции логирования: слишком большой или ошибочный клиентский профиль успевал открыть файл, а регистр допустимых ключей менял поведение.

Оба Linux entry point теперь используют `client::logging_bootstrap`: общий regular-file snapshot с пределом **262144 байта**, строгий клиентский INI-парсер и неизменяемую оценку доверия того же открытого inode. Неверный профиль остаётся на stderr; недоверенный профиль сохраняет допустимые level/time_format, но не выбирает file sink. CLI проверки получает zeroizing-текст из того же загрузчика. Удалён отдельный частичный парсер standalone-бинарника. Серверный bootstrap, INI-ключи, wire и ABI не менялись. Новый модуль ограничен `target_os = linux`.

На приватных исходниках с проверенными SHA348 compilation inputs: Rust 1.97.0, один Cargo job, offline/locked. Linux lib-набор: **2412 PASS, 60 ignored**; пять новых тестов покрывают FIFO, точный лимит, неверный профиль, регистр и права/симлинк. Строгий Clippy `--all-targets --features transport-core-ffi,client-bin -- -D warnings` и обе реальные Linux CLI сборки PASS.

`scripts/audit_linux_cli_bootstrap.py` запускает реальные процессы только в новых NET/mount/PID namespaces. База воспроизвела **10 ошибочных сценариев**. Исправление: **24 сценария / 46 проверок PASS**, в том числе ровно 256 КиБ и превышение на один байт, доверенный/недоверенный log sink, mixed-case INI, FIFO, каталог и отсутствующий файл. Точный private network before/after совпал; максимальная длительность исправленного CLI-сценария — 0,057 с. Это pre-connect проверки; успешное VPN-подключение ими не подтверждается.

Первичная обвязка сборок и первые CLI-обвязки завершились nonzero из-за изменений трёх внешних legacy firewall-правил. Эти FAIL сохранены. При пассивном наблюдении без тестов правила также появляются/исчезают с периодом около пяти секунд; источник пока не установлен. Итоговый `runtime-fixed-r2` прошёл и внутренние проверки, и **строгое полное сравнение host snapshots** без исключения правил. PID 745/901 и SHA работающего сервера сохранены; установленный сервис не заменялся.

Исходные логи и manifests: `C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q25-cli-20261004/`. `fixed-build/result.json` содержит PASS отдельных Cargo-команд, но `host_restored=false`: это не общий PASS обвязки. Итоговый CLI wrapper: `runtime-fixed-r2/wrapper.json`, `qualification_status=PASS`. Бинарники: standalone `a2cf8daa6dbb4da484a3306c86e11f2eb5ab9bef33e62e0cb0ca683c7de4cbce`, daemon `fba890ab7af1698780d35e4e5e64cd0b60119d21c2ed330a14a13405412f4240`.

Следующая граница Q25: финальная сверка Linux lifecycle/recovery с ранее закрытыми D01–D05/D09/D10/D13 и согласование native provenance после изменения исходников. Это не обещание нового router runtime: пользователь исключил роутерный стенд.
