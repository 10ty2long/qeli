# Q37: документация и тестовая обвязка — 11 октября 2026

<!-- normative-sync: q37-documentation-baseline-v2 -->

**Статус: IN_PROGRESS.** [Результаты и hashes](../../../release/audit_q37_documentation_2026-10-11.json). Полный Q37 ещё не завершён. Общий итог остаётся 30/37 DONE (81,1%).

Q37-F001: после правки updater из OPERATIONS исчезли versioned attestation examples, хотя `sync_version.py` продолжал требовать их в RU/EN. Исходная проверка упала на двух unmatched patterns. В оба руководства возвращён конкретный `gh attestation verify` для опубликованного amd64-пакета 0.8.1 и репозитория litvinovtd/qeli; строгость проверки сохранена. Пять методов на RU/EN проверили актуальный пример, старую версию, исчезнувшую команду, development-версию вместо релиза и точное восстановление версии с сохранением CRLF. Реальные `sync_version.py` и девять checks документации PASS.

Q37-F002: `site/` намеренно private/ignored; тест не мог работать из отдельного Git checkout без этих файлов. Добавлен `--site-root PATH` для настоящего каталога сайта. Проверены реальные RU/EN hub/установка/копирование команд; пять отрицательных сценариев (нет default-входов, неверный каталог, опечатка параметра, отсутствующее значение, удалённые обработчики в временной копии) отказали. Пять SHA входов сайта до/после совпали. Отсутствие сайта не превращается в SKIP/PASS. Исходники сайта и его публикация не менялись; полный статический сайт этим тестом не квалифицирован.

Исходная точка также дала PASS: панель 11 templates/1160 RU strings, 118 editor checks и документация. Дополнительные 97 тестов нативной обвязки и 18 certification gate tests PASS. Это unit/fixture проверки инструментов, не фактические A/B сборки и не успешная certification.


Q37-F003: [удаление недостижимых legacy реализаций](../../../release/audit_q37_dead_legacy_2026-10-11.json). В41 Python/shell файле убраны6320 строк за безусловным отказом. AST подтвердил, что до Python SystemExit допускаются только docstring и future annotations; shell имеет лишь shebang, stderr echo и exit1. Имена и сообщения отказа сохранены, чтобы старые команды не начинали работать; два сообщения больше не рекламируют удалённый формат. Удалён неиспользуемый scripts/client-config.json (1510bytes); единственная ссылка — frozen inventory прежнего fuzz-source snapshot, который сохранён как историческая запись. Всего убрано239053bytes.

Проверены42 настоящих Python refusal stubs (exit1/RETIRED, без stdout), три shell guards и актуальность generated client bindings. Существующий retirement test усилен: возврат тела после отказа теперь ломает проверку. Этот настоящий test отказал при подстановке оригинального source предыдущего commit;38 текущих recipe methods PASS. Никакие старые SSH/deploy тела не выполнялись. INI/parser/runtime/native/benchmark inputs не менялись; JSON для API, внутренней модели и test/evidence остаётся допустимым. Полный feature-aware dead-code review всех модулей ещё открыт.

**Открытый долг:** `native-libs/provenance.py --check` обнаружил изменение native inputs (recorded e3fb04a8, current 7d42e38f). Нужны реальные независимые A/B пересборки текущих desktop/Android recipes, сверка бинарных inventories и лишь затем обновление provenance. Штатная CLI-сборка для Q35/Q36 не обновляет библиотеки клиентов. Компиляция на лабе ждёт завершения измерений, чтобы не менять условия прогона.

Release certification дала 21 проблему: общий digest и 20 автоматизированных Linux cases относятся к другому дереву. Gate сверяет всё committed tree, кроме собственных evidence; даже новые docs/report commits меняют этот digest. Поэтому сначала фиксируется окончательное дерево release inputs, затем выполняется матрица на нём и обновляется evidence. Повторная запись старых PASS под новый digest запрещена. Пропущенные пользователем физические платформы не объявляются протестированными.

Q35 полный endurance и Q36 текущая data-plane матрица выполняются отдельно; здесь их промежуточные состояния не объявлены PASS. Для полного Q37 ещё нужны keys/defaults/examples parity, dead-code review с учётом OS/features/FFI и integration. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.
