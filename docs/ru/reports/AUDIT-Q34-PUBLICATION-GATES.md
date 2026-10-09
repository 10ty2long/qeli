# Q34: допуск release source и Docker notices

<!-- normative-sync: q34-publication-gates-v5 -->

Дата завершения проверок: 9 октября 2026. Packet начал собираться 8 октября.

## F474 / Q34-F028: настоящий тег и CI на точном source commit

Прежний shell проверял `git rev-list -n1 "$TAG"`, HEAD и version, но короткое имя
может обозначать ветку. Реальный Git baseline принимает branch `v0.8.2` без тега.
Release workflow также не вызывал exact-commit CI admission перед сборкой/upload.
Теперь checkout задаёт `refs/tags/...`, общий read-only helper проверяет actual tag
на HEAD и committed Cargo version, clean source и отсутствие Git trust flags,
которые скрывают изменения. Before build требуется CI success на exact commit.
GH query использует `--commit`, missing/stale/failed/query failure блокирует допуск.
Source identity перепроверяется после CI query. Output пишется только после допуска;
тесты проверяют lightweight/annotated tags, branch collisions, dirty/untracked,
assume-unchanged/skip-worktree, HEAD drift, missing Git/timeout и negative CI.
Full `release_preflight.py` остаётся отдельным gate всех платформ/SDK/certification.
Старые теги без нового helper/notice policy автоматически не квалифицируются.

## F475 / Q34-F029: release/Docker compiler pin

Release использовал Rust action без toolchain value; Docker использовал движущийся
`rust:1-bookworm`. Это расходилось с reviewed Rust1.97.0 notices и native policy.
Release берёт version из общего DEFAULT_RUST_TOOLCHAIN. Docker builder1.97.0 и
Debian runtime закреплены по multiarch digest. Official Docker Hub HTTPS metadata
содержит amd64,arm64/v8,arm/v7. Два registry manifest-body запроса дали401 и сохранены;
whole manifest/body/layer pull этим не квалифицированы. Digest взят из official tag
metadata, проверка actual pull выполняется Docker при будущей сборке. Live APT
repositories не закреплены snapshot; полная воспроизводимость образа не заявлена.

## F476 / Q34-F030: common notices в Docker и extracted DEB

Docker runtime не копировал AGPL/common Cargo/toolchain legal resources. Теперь
сохраняет root LICENSE/LICENSING и весь533-file bundle в `/usr/share/doc/qeli`.
Context ignore не исключает эти paths. Docker publication проверяет общий bundle
до login/build. Release smoke gate проверяет actual extracted DEB notices до
checksum/attestation/upload. Эта партия проверяет source recipe и common bundle,
не заявляет новую whole Docker/native build или container VPN runtime.

## F477 / Q34-F031: CI trigger для всех workflows

Push/PR paths включали только `ci.yml`; изменения release-attest/docker-publish
без изменения другого covered path могли не запустить CI. Оба фильтра теперь
включают `.github/workflows/**`. Added release tag/CI contract suites выполняются
в CI. Реальный YAML parser проверил все три изменённых workflow и новые fields;
GitHub expression evaluation/trigger execution не заменяются локальным parser.

## Проверки и пределы

- 86 methods PASS:17 release tag,19 release CI,12 whole preflight,38 existing native recipe contracts. Final17 включает обнаруженный при review Git trust-flag bypass и новый отказ; остальные final inputs не менялись после их прогона.
- Actual baseline shell/real Git branch-only acceptance reproduced;final actual CLI branch-only/version negatives and matching tag positive PASS.
- Official SHA-qualified PyYAML6.0.3 pure Python BaseLoader реально разобрал три workflow; parser только в собственном scratch, system packages не изменялись.
- Official Docker Hub tag metadata/digests/platforms сохранены; registry401 не переписаны в PASS. Read-only exact-commit CI query для исходного local dev3cd67555 вернул пустой список; current-source Actions qualification OPEN, ничего не dispatch/push.
- Common native/toolchain/managed bundles,protected files,14 native artifact SHA и nativee3fb проверены. Docs/check/diff и source scope приведены в raw evidence. Лаба не затронута; новое ядро/APK/publish/SDK/router/Mac/device не собирались.

GitHub уже предоставляет source archives для release tag; отдельный duplicate
source asset не создаётся. Полный source commit фиксируется build evidence.
[GitHub source archives](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases).
Это не доказательство matching source third-party binaries/downstream patches.
Matching source publication,immutable OpenWrt SDK source cut,MicroCom exact binding,
whole Docker pull/build/current-source Actions/signatures/full release остаются OPEN.
Q29 SIGKILL FAIL/ENONET,Q30/Q31,F459 braces,D06 accepted,USER_EXCLUDED сохранены.
Q34 IN_PROGRESS,30/37 DONE81,1%,осталось7. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## Evidence

`q34-publication-gates-20261008`: SEALED 73 files / 1725534 bytes; manifest SHA256 `ce00677e40dae1fa6d7fee5067f2df5a2d5c5c8db5f3154c8c779411070df9cd`.

## 9 октября: F490 / Q34-F044 — защита Docker latest от старого релиза

На настоящем Git-репозитории с `v0.8.2` и `v0.9.0` воспроизведено:
прежний CLI выдаёт `stable_release=true` для `v0.8.2`, а workflow использует
этот флаг для `latest`. Повторный запуск старой версии мог откатить канал.

`docker_publish_ref.py` теперь отдельно выдаёт `latest_release`. Для стабильного
тега требуется полный Git-репозиторий, наличие самого тега и отсутствие более
высокой стабильной версии в полученном списке тегов. Сравнение числовое по
MAJOR/MINOR/PATCH; build metadata не меняет приоритет. RC и ветки получают false,
не обращаясь к списку тегов; посторонние и некорректные теги его не определяют.
Старый стабильный релиз сохраняет публикацию версии и SHA, но не `latest`.
Отказ Git, timeout, shallow checkout или отсутствующий тег блокируют публикацию
до записи outputs и до login/build. Прежний допуск Cargo version / exact-commit CI
сохраняется отдельно: выбор `latest` его не заменяет.

В workflow общая concurrency group сериализует публикации разных refs;
`cancel-in-progress: false` сохраняет уже выполняющийся запуск. Одной
сериализации недостаточно: GitHub не гарантирует порядок очереди, поэтому
необходима и проверка максимальной версии. GitHub может отменить прежний
ожидающий запуск при появлении следующего — сохранение всех ожидающих публикаций
не гарантируется. См. [официальные правила concurrency](https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency).
Автоматический `latest` metadata-action остаётся выключен;
явный тег использует новый флаг. См. [metadata-action](https://github.com/docker/metadata-action).

Проверено 16 методов на Windows и те же 16 на Linux: старый/новый тег,
числовое сравнение, RC/посторонние теги, равный приоритет build metadata,
ветки без Git-запроса, отсутствующий тег/репозиторий, настоящий shallow clone,
отказ/timeout с сохранением outputs и настоящий CLI. Ещё 59 связанных методов
release admission / CI / preflight / workflow gate прошли на Windows.
Официальный SHA-проверенный actionlint1.7.12 принял все четыре workflow.
Linux-входы и полный лог восстановлены и проверены SHA; собственный каталог
удалён после проверки процессов, файловых дескрипторов и mount points, только
по точному списку файлов без рекурсивного удаления. Служба .11, PID777/845 и SHA
установленного Qeli сохранены; .10 не использовалась.

Граница: правило выбирает максимальный полученный стабильный Git-тег, а не
опрашивает состояние GHCR. Более новый тег, даже до успешной сборки, запрещает
старому менять `latest`. Требуются полный актуальный fetch и сохранение стабильных
тегов; удаление/перемещение тегов, внешние записи в GHCR и восстановление старого
workflow вне этого изменения не квалифицированы. Старые refs используют свой
workflow и не получают исправление задним числом. Реальные push, hosted Actions,
подписи и multiarch-сборка не выполнялись. Ядро, native артефакты и OpenWrt source
cut неизменны; прежние свидетельства не являются новым прогоном.

F490 закрыт в этих границах. Q34 IN_PROGRESS; 30/37 DONE (81,1%), 7 разделов
остаются. Q29 SIGKILL FAIL/ENONET, Q30/Q31, F459 braces OPEN, D06
ACCEPTED_LIMITATION и USER_EXCLUDED сохранены. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F490 evidence: `q34-docker-latest-20261009`; SEALED 31 files / 1049840 bytes; manifest SHA256 `61d93c90e2640e781acf6ef239b72ce1a0cce1fd5e3f5a7eafb8ba24816581d2`.


## 9 октября: F494 / Q34-F048 — единый допуск исходников release

Реальный Git воспроизвёл пропуск в старых preflight и certification:
assume-unchanged и skip-worktree скрывали изменение tracked-файла от status.
Preflight допускал прежний HEAD, certification возвращал digest committed tree,
хотя рабочий файл уже отличался. Проверка release tag запрещала эти флаги.

Теперь release_source.py используется всеми тремя входами. До внешних запросов
проверяются существующий commit, индексные флаги и чистота дерева с NUL-разделением
Git paths. Любой assume-unchanged/skip-worktree блокирует допуск, даже без изменения
файла. Preflight повторяет проверку между этапами и после certification.
Certification читает ls-tree по принятому commit и повторно проверяет identity:
смена HEAD, новый скрывающий флаг или видимое изменение блокируют результат.
Git queries ограничены 30 секундами; отсутствие Git, timeout и ненулевой exit
превращаются в отказ certification, а не в успешную проверку.

Для штатного run_ipv6_release_matrix.py --record разрешены новые untracked-файлы
только внутри release/certification/, поскольку этот каталог исключён из digest.
Изменение уже tracked evidence требует commit; untracked source вне этого каталога
блокирует certification. Игнорируемые build caches разрешены. Preflight и release
tag требуют чистое дерево без новых untracked evidence: сначала зафиксируй отчёт,
затем запускай release gate. Пользовательский --evidence вне certification также
нужно зафиксировать либо разместить в ignored output. Алгоритм digest для чистого
committed source сохранён; скрытые изменения manifest/evidence тоже отклоняются.
Проверка не защищает от привилегированного параллельного изменения ignored caches.

165 Windows методов и 33 Linux метода PASS: реальные Git fixtures, hidden flags,
смена HEAD, поздние флаги, query errors, новые evidence, tracked evidence edits,
ignored caches, orchestration и прежние tag/CI/router/OpenWrt/recipe contracts.
Существующие CI suites уже включают новые regression tests; workflow не менялся.
Внешние CI/lab build gates в orchestration контролируемые; настоящая CI-публикация,
SDK/router compile и runtime VPN этим прогоном не заявлены.

Linux source archive проверен SHA до исполнения; 16 известных inputs удалены
после SHA/process/fd/mount guards без recursive deletion. Initial cleanup generator
получил SyntaxError до исполнения из-за широкой замены FILES; helper encoding typo
отказал до SSH. Оба отказа сохранены, исправленная очистка завершена. Служба .11
777/845 и /usr/bin/qeli75bc сохранены; .10 не использовалась. Native14/source digest/
qeli tree/protected inputs неизменны; прежние execution proofs унаследованы без
Rust/APK/Maven rebuild. 41 certification cases и 95 прежних supplements сохранены
кроме source_digest bookkeeping; добавлен supplement96 этой ограниченной проверки.
Q34 IN_PROGRESS,30/37 DONE81,1%,осталось7; F459/Q29 SIGKILL FAIL-ENONET/Q30/Q31,
D06accepted,USER_EXCLUDED и expected release publisher OPEN сохранены.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F494 evidence: q34-release-source-trust-20261009; SEALED 51 files / 1287941 bytes; manifest SHA256 2c7e1d8ea5e65a74c96f6db525eed2fe05310a1ba8febdafc41d908a11fd39dc.


## 9 октября: F495 / Q34-F049 — certification в actual release workflows

Release DEB и release-tag Docker запускали release_tag.py --require-ci,
но не проверяли certification manifest. CI запускал unit-тесты валидатора,
а не admission самого manifest. На настоящем приватном Git fixture с тегом
v0.8.2 и committed failed обязательным linux.outer4.inner4.tcp.full старая
точная publisher-команда при контролируемом CI success вернула PASS/rc0 и
записала GITHUB_OUTPUT. Сам валидатор в этом же fixture отклонил failed-кейс.
Это подтверждённый пропуск, без настоящей публикации.

Оба publisher entrypoints теперь явно передают --require-certification вместе
с --require-ci. После tag/version/source admission общий validator проверяет
release/certification/VERSION.json по текущему source digest и обязательным
runtime case statuses. Missing, malformed, stale и failed evidence блокируют
допуск до CI query, DEB compile, Docker login/build/push и записи outputs.
После чтения manifest повторяется source/tag identity; HEAD drift или изменение
tracked manifest также блокируют допуск. Ошибки I/O/CertificationError дают rc1.
Успех требует одновременно certification и CI на exact commit.

Недоступные физические случаи сохраняют advisory-статус по действующей политике;
explicit failed physical case блокирует release. USER_EXCLUDED устройства не
объявляются проверенными. Branch/SHA Docker publication сохраняет свой отдельный
контракт: новый gate обязателен для release tags. Обычный read-only вызов
release_tag.py без --require-certification проверяет metadata, не заменяет
полный release admission. Для ручного допуска:
python scripts/release_tag.py --tag vVERSION --require-ci --require-certification
Заменить VERSION на committed Cargo version. Полный release_preflight.py остаётся
проверкой дополнительных SDK/router/source gates, а не заменяется этой командой.

132 Windows метода и 41 Linux метод PASS. Проверены настоящий offline CLI,
Git tags и committed JSON manifests, failed automated/physical кейсы, advisory
unavailable, missing/stale/malformed manifests, I/O refusal, смена HEAD/manifest,
отсутствие новых outputs при отказе, positive metadata и CI refusal.
CI success в orchestration контролируемый; реальные GH query/build/upload/OIDC
в этих тестах не исполнялись. Семантический pinned actionlint принял все четыре
workflow; существующий CI suite уже включает новые тесты. Два workflows явно
проверены на ordering gate перед compilation/login.

В .11 выполнены только приватные Python/Git fixtures. Восемь source inputs и
их archive проверены SHA; девять файлов удалены без recursive deletion после
process/fd/mount guards. Original service777/845 и CLI75bc сохранены, .10 не
использовалась. Native14/qeli tree/protected inputs/OpenWrt source cut неизменны;
прежняя квалификация наследуется без Rust/APK/Maven rebuild.41 cases и96 прежних
certification supplements сохранены кроме source_digest bookkeeping; добавлен97.
Это не новый PASS actual release/OIDC/signatures/current-source Actions и не
перезапуск всей runtime matrix. F459 по [первичному advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) всё ещё без published
patch, не подавлен. Q34 IN_PROGRESS,30/37DONE81,1%,осталось7;Q29 SIGKILL FAIL-ENONET,
Q30/Q31,D06accepted,USER_EXCLUDED и прочие open пункты сохранены.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F495 evidence: q34-publisher-certification-20261009; SEALED 42 files / 1211464 bytes; manifest SHA256 d11345555e523dc6296de351daf95a0a2d8b942c4d672527a7cbd43fd5c80837.


## 9 октября: F496 / Q34-F050 — отказ упаковки без подписанта

build_app.sh объявлял подписант обязательным, но при отсутствии codesign и
rcodesign лишь печатал предупреждение. Затем создавал неподписанный архив и
сообщал Done. Воспроизводитель выполняет прежний Bash-скрипт с настоящими
файлами и tar, контролируемыми compiler/sign tools: 8 методов дали 3 ожидаемых
провала assertions — обе архитектуры и запуск native bootstrap до отказа.

Теперь подписант выбирается до native build, publish и замены выходных файлов.
При отсутствии обоих инструментов скрипт возвращает ошибку; предыдущие bundle
и архив остаются нетронутыми. Приоритет codesign сохранён, ad-hoc ветка вызывает
выбранный путь инструмента. Ненулевой exit подписанта останавливает скрипт до
tar и сохраняет предыдущий архив. Уже собранный bundle при ошибке подписанта
может остаться в dist; его атомарная замена и параллельные сборки этим не исправлены.
Developer-ID/per-app/notarization сценарии не изменены.

57 Windows методов PASS: 8 app-signer, 38 native-recipes и 11 workflow contracts.
Все четыре workflows приняты pinned semantic actionlint; Bash syntax PASS.
В Linux .11 прошли те же 8 app-signer методов. Проверены arm64/x86_64, отсутствие
подписанта до native bootstrap, оба положительных пути, приоритет codesign,
ошибки обоих подписантов и неизвестная архитектура. Test suite включён в CI.
Настоящие файловые операции и tar проверены, compiler/sign tools контролируемые:
это не сборка payload, не проверка криптографической подписи и не запуск на Mac.
Apple runtime остаётся исключённым пользователем.

Два committed Linux source inputs и их TAR проверены SHA. Приватный каталог
с тремя известными файлами удалён после проверок набора, process/fd и mounts;
original service777/845 и CLI75bc неизменны, .10 не использовалась.
На локальном диске обнаружено отсутствие свободного места. Удалена только старая
распакованная копия SDK из audit-scratch: 5577 файлов /807336133 bytes, полный
SHA512 исходного ZIP, точный набор и SHA каждого файла совпали. Проверены resolved
path, отсутствие reparse points и использующих SDK процессов. Исходный ZIP,
SEALED packets и shared caches сохранены; освобождено около800MB.

Native source digest,14 artifact SHA,qeli tree,protected files и OpenWrt source
cut сохранены; их прежняя execution qualification наследуется без пересборок.
41 certification cases и97 прежних supplements сохранены кроме source_digest
bookkeeping; добавлен98 scoped supplement. Q34 IN_PROGRESS:30/37 DONE81,1%,7remain.
Прежние open/accepted/excluded пункты не закрыты этим исправлением.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F496 evidence: q34-app-signer-20261009; SEALED 35 files / 1203484 bytes; manifest SHA256 74dd6ad767b38be09ef2513e689c22c42bf4d37754e270684e7d6f30b85e5ea4.
