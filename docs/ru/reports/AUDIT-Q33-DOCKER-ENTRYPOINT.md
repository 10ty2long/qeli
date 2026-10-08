# Q33: конфигурация Docker entrypoint и публикация при первом запуске

<!-- normative-sync: q33-docker-entrypoint-v1 -->

8 октября2026. F400–F403 / Q33-F003–F006. Q33 остаётся IN_PROGRESS: пакет проверяет только контейнерный entrypoint. Полные Debian install/upgrade/remove, rollback updater, сборка/образ/recreation/health Docker и остальные права файлов/командные пути требуют дальнейшего разбора и интеграции.

## Воспроизведённые ошибки и исправления

Четыре проверки старого поведения падают на dash и ash (восемь ожидаемых FAIL):

- Пустой argv приводит к `shift` и выходу вместо документированного server по умолчанию. Теперь сдвигается только переданный аргумент роли.
- Переданный `--config` добавляется после внедрённого default: Clap отвергает дубликат, а создание конфига затрагивает другой файл. До записи entrypoint разбирает `--config PATH`, `--config=PATH`, `-c PATH`, `-cPATH`, сохраняет argv и добавляет путь лишь при отсутствии CLI-опции. Явный CLI-путь имеет приоритет над QELI_CONFIG. Пропущенный/пустой/дублированный путь отвергается до записи. Help/version передаются executable без создания файлов.
- Новые config/users наследуют umask022 и получают0644. Теперь umask077 и mktemp0600; байты и режимы существующих файлов сохраняются.
- Висячая ссылка users.conf проходит прежнюю проверку отсутствия, и перенаправление создаёт её цель. Создание config/users отвергает ссылки и не обычные файлы, пишет временный файл рядом и публикует GNU `ln -T`, запрещающим перезапись и каталог как цель. Конкурирующий обычный файл сохраняется; конкурентная ссылка/каталог блокируют запуск. Ошибка copy/link не оставляет частично опубликованный конечный файл, временный файл очищается.

Публикация означает видимость полных байтов, а не fsync/устойчивость к отключению питания. SIGKILL может оставить временный файл. Для первого создания volume должен поддерживать hard links; неподдерживаемая/readonly FS безопасно отвергается. Привилегированные конкурентные изменения родительских каталогов или финального чтения вне этой проверки согласованной публикации. Существующие файлы не chmodятся и не переписываются.

## Выполненные проверки и границы

`python scripts/test_docker_entrypoint.py` требует Linux root/chroot. Необязательный QELI_ENTRYPOINT_ASH_ROOT задаёт распакованный ash/musl; QELI_ENTRYPOINT_TEST_SHELL выбирает shell, QELI_ENTRYPOINT_SOURCE — старые/текущие байты. Вместо qeli используется executable, записывающий argv; daemon не выполнялся.

Точный целый entrypoint выполнялся на .11 в временных chroot и отдельных mount/net/PID namespaces:17 методов на dash/ash,34 сценария и46 запусков PASS без skips. Проверены CLI-формы/приоритет, option-only/пустой запуск, сохранение состояния/прав, help без записей, ссылки/FIFO, детерминированный конкурентный file/link/directory creator, очистка при copy/link failure и shell-метасимволы как данные пути. Обе shell используют настоящие Debian coreutils для публикации; ash взят из сохранённого SHA-проверенного OpenWrt userspace, совместимость с BusyBox-утилитами не заявляется.17 платформенных skips Windows не квалифицируют runtime.

В существующий Linux CI job добавлена команда `sudo unshare --mount --net --pid --fork python3 -B scripts/test_docker_entrypoint.py`. Та же последовательность аргументов выполнена root на .11 в точном fixture двух файлов:17 методов dash/23 запуска PASS без skips. Проверены backup/input SHA и cleanup guards; PID777/845 сохранены. Локальная проверка не означает новый прогон Actions. Отдельный raw: `audit-debt-20260924/q33-docker-entrypoint-ci-20261008`.

Первые ошибки fixture (PATH без chroot, контекст генератора, повторный mkdir и отсутствующий cp) сохранены отдельно и не считаются продуктовыми результатами. После ремонта все четыре старые ошибки воспроизведены на обеих shell, финальный набор PASS. SHA конечных remote-входов совпадает с кандидатами скриптов. Backup SHA проверен локально до удаления собственного каталога с process/fd/mount/resolved-path guards. Service PID777/worker845 не изменились; VPN, host firewall, Docker daemon и production service не запускались/не перенастраивались.

Rust/native recipes, ABI и поставляемые cores не изменились; повтор несвязанных native-сборок не нужен. Доказательства: `release/certification/evidence/q33-docker-entrypoint-20261008.json`. Raw: `audit-debt-20260924/q33-docker-entrypoint-20261008`,25 файлов/2072139 байт; manifest SHA256 `8596d49f7129b2b32856cadd0f60ab6fd6c3ea0d288d21b1fdcc80effe283e46`.

Штатный bootstrap Rust SDK Q31 отложен по ресурсам, это не PASS и не compile FAIL; release source-cut/current Actions OPEN. Q29/Q30/user exclusions и D06 accepted limitation сохранены. Полный план29/37(78.4%),осталось восемь. Push/deploy/release не выполнялись.
