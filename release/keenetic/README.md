# qeli-client на Keenetic (Entware) — IPv4/IPv6 деплой

Запуск qeli-VPN-клиента на роутере Keenetic как шлюза для всего LAN.
**Подробный пошаговый гайд (по шагам, с проверкой туннеля) — [docs/*/manuals/KEENETIC-DEPLOY.md](../../docs/ru/manuals/KEENETIC-DEPLOY.md).**
План и обоснование порта — [docs/*/reference/KEENETIC-PORT.md](../../docs/ru/reference/KEENETIC-PORT.md).

> ✅ Клиент проверен на живом Keenetic и работает. Скрипты в этой папке остаются
> **шаблонами**: проверь имена интерфейсов и поведение firewall под свою
> модель и версию прошивки.

## Предусловия на роутере
- Установлен **Entware** (opkg, `/opt`).
- Включён компонент **VPN** в KeeneticOS (чтобы был `/dev/net/tun`).
- Есть SSH-доступ.

## Сборка бинарей (на лабе .10)
```sh
python scripts/build_keenetic.py --sync
# → release/keenetic/qeli-client-keenetic-aarch64 и qeli-client-keenetic-mipsel
```

## Установка (на роутере)
Скопируй всю папку `release/keenetic/` на роутер (scp) и запусти:
```sh
sh install-keenetic.sh      # определит арку, поставит ip-full/iptables, разложит файлы
vi /opt/etc/qeli/client.conf
/opt/etc/init.d/S99qeli start
tail -f /opt/var/log/qeli-client.log   # ждём 'Auth OK'
```

## Режим шлюза (весь LAN через VPN)
- В `client.conf`: `gateway = true` (full-tunnel) и `dns = off` (не трогать DNS роутера).
- Рекомендуемый конфиг использует `gateway_nat = true`: само ядро qeli ставит и проверяет
  IPv4/IPv6 forwarding, MASQUERADE, FORWARD и MSS clamp, а при остановке восстанавливает
  изменённые sysctl. `S99qeli` сохраняет `GATEWAY=yes` только как fallback для старых
  конфигов без `gateway_nat`/`forward`.
- Legacy fallback ждёт до 60 секунд атомарно опубликованный после AUTH/NetworkPlan файл,
  включает firewall только для реально выданных IPv4/IPv6 и не угадывает семейство по
  `ipv6 = auto`. На RA/SLAAC WAN он сохраняет `accept_ra`, ставит `2` до IPv6 forwarding и
  восстанавливает прежнее значение при остановке.
- Для inner IPv6 оставь `ipv6 = auto`, используй `required` для fail-closed или `off` для
  принудительного IPv4. Для dual-stack router-mode нужен `ip6tables`; без него `auto`
  согласует IPv4, а `required` откажет до настройки интерфейса.
- При необходимости ограничь NAT реальными `lan_subnet` и `lan_subnet_ipv6` своего LAN.
  Имя LAN-бриджа для legacy fallback (`LAN_IF`, обычно `br0`) проверь через `ip a`.

## Выбор режима под железо
- **MIPS** (MT7621/7628, без AES-NI): `fake-tls` / `obfs` / `plain` (ChaCha20). Потолок —
  десятки Мбит. `reality-tls` очень медленный (двойной AEAD).
- **ARM** (Cortex-A53, crypto-ext): можно `reality-tls`; скорость в разы выше.

## Удаление
```sh
/opt/etc/init.d/S99qeli stop
rm -f /opt/etc/init.d/S99qeli /opt/bin/qeli-client
rm -rf /opt/etc/qeli
```

## Ошибки установки (development0.8.2)

Установщик предпочитает canonical имена `qeli-client-keenetic-aarch64/mipsel`,
старые `qeli-client-aarch64/mipsel` поддерживает для ручных bundles. Ошибка установки
обязательных ip-full/iptables прекращает установку; отсутствие ip6tables явно
предупреждается и требует проверить IPv6 перед запуском.

Существующий INI сохраняется, доступ ограничивается0600, каталог —0700. Копии
готовятся до замены; каждый файл публикуется атомарно, но весь bundle не является
транзакцией. После отказа публикации устранить ошибку и повторить установщик.
Работающий VPN автоматически не перезапускается. Это проверено shell/filesystem
fixtures, не настоящим opkg/прошивкой/ELF. См. [Q31](../../docs/ru/reports/AUDIT-Q31-OPENWRT-CONTROLS.md).

Legacy gateway использует правила с тегом `qeli-keenetic-legacy` и forwarding
checkpoint `version=2`. Ошибка очистки сохраняет состояние и блокирует перезапуск.
Старый checkpoint без версии требует ручной сверки; он не удаляется автоматически.
Перед заменой работающего legacy-шаблона остановить его предыдущим скриптом и
проверить правила/sysctl. Подробности: [восстановление при обновлении](../../docs/ru/manuals/KEENETIC-DEPLOY.md).

PID-файл нового шаблона хранит `PID start_ticks`. До обновления остановить клиент
предыдущим установленным шаблоном и проверить завершение: установщик блокирует
существующий PID/pending record. Stop ждёт TERM перед очисткой; при timeout состояние
сохраняется, restart не запускает ещё один клиент. Старые single-PID записи требуют
сверки владельца. Status:0 running,3 stopped,4 unverified. Шаблон не удаляет TUN
напрямую. [Порядок остановки и обновления](../../docs/ru/manuals/KEENETIC-DEPLOY.md).

Gateway fallback больше не разбирает INI в shell. При GATEWAY=yes без OpkgTun
шаблон читает `qeli-client --config ... --print-gateway-owner`: core для
gateway_nat/forward/exit_node, legacy при выключенных flags. Ответ/ошибка
проверяются до запуска; старый binary без команды отказывает. Обновлять binary
и шаблон вместе; проверка и запуск используют общую приватную копию INI. Для OpkgTun или
GATEWAY=no этот compatibility query не нужен. [Подробности](../../docs/ru/manuals/KEENETIC-DEPLOY.md).

## Общий lifecycle (development0.8.2)

Базовый installer доставляет обязательную библиотеку
/opt/etc/qeli/lifecycle.sh (0600). Обновляй её вместе с binary/init и установленным
wan.d hook; старые скрипты общей блокировки не соблюдают. Init, hook и installer
используют /var/run/qeli.lifecycle.lock. Занятый вызов возвращает ошибку — повторить
после завершения владельца. Обычный выход/сигналы освобождают lock; после SIGKILL
нужна сверка отсутствия init/hook/installer-владельца перед rmdir пустого каталога.
Recovery-файлы PID/plan/forwarding/pending не удалять. Hook-события не очередятся;
обнаруженная смена плана прерывает L3/save/receipt и оставляет pending. Атомарность
core-поколений/ABA/firmware этим не доказана.

Maintainer builds now always upload into private0700 /var/tmp/qeli-router-keenetic-XXXXXX
checkouts, with separate target/rlib restriction and one compiler job. --sync is
compatible but optional. Retained directories require reviewed per-run cleanup.
SHA256-bound ELF admission precedes atomic local publication; it checks class,
machine, little endian and absence of interpreter/shared dependencies, not full
ISA/musl/firmware compatibility or reproducibility. Old /opt/qeli-src is unused.
See [deployment manual](../../docs/eng/manuals/KEENETIC-DEPLOY.md) for limits.

Both router helpers now share target aliases, explicit Rust1.97.0/MIPS nightly-2026-06-10,
Zig0.13.0/cargo-zigbuild0.23.0 admission and build commands. Encoded/ambient flags
cannot shadow recipe RUSTFLAGS; MIPS soft-float remains explicit. Wrong identity
stops setup; installer/target false success fails verification. Shared caches/global
Cargo config/PATH and actual cross/firmware behavior remain separately qualified.

Installation now keeps same-directory backups of the previous binary/init/helper
and a0600 /opt/etc/qeli/install-pending recovery record while publishing code.
Ordinary publication failures and a handled termination signal restore old bytes
and modes (or remove new code that was originally absent). A failed restore keeps
remaining backups and the marker. Installer retry, both updated init starts and
the updated active wan.d hook refuse it; stop remains available. Existing INI
contents are preserved. A newly published example and stricter600 config mode
may remain after a late error; package/dependency changes are not rolled back.

If install-pending remains, keep the client stopped and inspect the record and
remaining backups before any start or retry. Verify no init/hook/installer owns
the lifecycle action before repairing the complete compatible binary/init/helper
set. Empty backup fields mean the target was originally absent; a missing named
backup may already have been restored and does not authorize blind file deletion.
After verifying recovery, remove install-pending and, if SIGKILL left it, only the
empty lifecycle lock directory. Do not delete PID/plan/forwarding recovery records.
Older installed scripts may not honor this marker: manual stop/review remains
mandatory. No automatic SIGKILL/power-loss recovery or fsync-backed bundle
transaction is claimed. Symlink/nonregular publication targets and a linked
/opt/etc/qeli directory are rejected before package updates. Root/admin path
replacement during installation remains outside this cooperative protocol.

Перед start/restart обновлённый шаблон проверяет конфигурацию общим парсером
бинарника: dev должен совпадать с TUN в шаблоне, device_type должен быть tun.
Обычный шаблон требует dev_attach=false; активный OpkgTun — dev_attach=true
и совпадение dev с OPKGTUN. В OpkgTun отключи gateway_nat, forward и exit_node:
L3/gateway принадлежат ndm. Проверка действует и при GATEWAY=no.

При несоответствии запуск отвергается до создания процесса и изменения старого
состояния. Согласуй INI и шаблон, затем повтори start. Обновляй binary и template
вместе: старый binary не понимает новые --expect-router-device/
--expect-router-attach и откажет в проверке. Проверка и запуск используют
одну приватную копию INI, описанную ниже.

## INI, проверенный перед запуском (development0.8.2)

Оба init-шаблона перед проверкой создают `/opt/var/run/qeli-launch/client.conf`
и передают этот же путь проверке ядра и клиенту. Исходный
`/opt/etc/qeli/client.conf` остаётся редактируемым; замена после проверки не меняет
конфигурацию текущего запуска. Повторный start работающего клиента сохраняет копию,
restart после успешного stop читает свежий исходный INI. Hook-переменная
`QELI_CONFIG_PATH` указывает на runtime-копию. Редактировать нужно исходный файл.

Требуются `stat -c`, `cp -p` и `mktemp`. Исходный INI должен быть обычным непустым
файлом без symlink, его каталог — root-owned0700. Установщик проверяет владельца
существующего каталога до package updates, затем ограничивает права0700;
не переназначает чужой каталог root. Runtime-каталог тоже root-owned0700;
ссылки, нестандартные targets и небезопасные права запрещают запуск.
Владелец и mode файла сохраняются при копировании: оно не повышает доверие ядра
к командам в конфиге. Ограниченный по размеру copy предшествует INI-парсеру;
точный лимит256KiB и правила INI остаются в ядре.

Успешный stop удаляет копию после подтверждённого выхода клиента и очистки сети.
При неудачном join/cleanup она сохраняется вместе с recovery state. Ошибка проверки
до запуска может оставить приватную копию; следующий допустимый start заменит её.
После SIGKILL/power loss проверяй владельца lifecycle и временные копии вручную.
Не удаляй runtime-copy у работающего или неопределённого клиента. Прямой запуск
binary вне шаблона не использует этот протокол. Замена приватных путей root/admin,
core plan generation/ABA и настоящая firmware этим исправлением не квалифицированы.

## Компилятор и промежуточные build files (development0.8.2)

Общий helper router-сборки теперь находит rustc выбранного pin через
`rustup which --toolchain ... rustc`, проверяет executable и явно передаёт его
Cargo. `CARGO_TARGET_DIR` и `CARGO_BUILD_BUILD_DIR` направлены в target собственного
запуска: обычные Cargo build.rustc/build.build-dir и ambient build-dir не могут
молча перенаправить эти два входа. Router CI использует ту же политику.
Linux gate `keenetic_verify.py` теперь требует Rust1.97.0 с установленным Clippy,
а не default host toolchain; ошибка поиска компилятора останавливает Cargo.
Wrapper/encoded Rust flags подчиняются общей политике.

Это ограниченная изоляция окружения. Cargo home/ancestor config, forced environment,
profile/source/linker overrides, PATH/shared registry/tool caches и clean независимые
Qeli A/B ещё не квалифицированы.7 real-Cargo methods используют маленькие Linux
fixtures без зависимостей; fresh Qeli cross build, Zig invocation, SDK Rust toolchain
и router ABI/runtime ими не подтверждаются. См. [Q31](../../docs/ru/reports/AUDIT-Q31-OPENWRT-CONTROLS.md).

## Поколение сетевого плана (development0.8.2)

Ядро добавляет `generation=<число>` в runtime-файл `$QELI_TUNIP_FILE` после MTU.
Первая строка адреса и поля ipv4/ipv6/mtu сохраняются. Это существующее поколение
аутентифицированного NetworkPlan, а не параметр INI. Hook сравнивает весь файл:
повторная аутентификация с теми же адресами и новым поколением требует нового
apply/save; A→B→A во время apply/save внутри одного адаптера оставляет pending
вместо устаревшего APPLIED receipt. Следующее событие может повторить применение.
Старые файлы без generation читаются, но не дают этого различения.

Счётчик локален адаптеру и может сбрасываться при его пересоздании. Поле не делает
всю настройку ndm атомарной и не закрывает смену после последней проверки,
координацию writer с lifecycle lock или root/admin replacement. Новые сборки
бинарника содержат поле; ранее опубликованные бинарники автоматически не обновляются.
