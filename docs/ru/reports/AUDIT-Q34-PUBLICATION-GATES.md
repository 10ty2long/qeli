# Q34: допуск release source и Docker notices

<!-- normative-sync: q34-publication-gates-v1 -->

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
