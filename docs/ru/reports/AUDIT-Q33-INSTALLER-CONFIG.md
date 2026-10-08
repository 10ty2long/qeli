# Q33: ранние проверки installer и публикация подготовленного INI

<!-- normative-sync: q33-installer-config-v1 -->

8 октября 2026. F432–F436 / Q33-F035–F039. Q33 остаётся IN_PROGRESS.

## Находки и исправления

- **F432 / Q33-F035:** явный PUBLIC_HOST и параметры публичной панели проверялись после изменений config/identity/users. Теперь отказ происходит до изменений dependencies/payload. Auto-host проверяется до подготовки config/identity, но dependencies/binary уже могут измениться. Прежняя проверка символов/длины защищает от инъекции, не заменяет полный валидатор hostname/IP/CIDR.
- **F433 / Q33-F036:** опечатки boolean opt-ins молча выбирали другую политику. QELI_FORCE_RECONFIG, QELI_PANEL_PUBLIC, QELI_VERIFY_ATTESTATION и QELI_ALLOW_UNVERIFIED принимают только0/1; unset/empty означает0. Иные непустые значения отклоняются до изменений.
- **F434 / Q33-F037:** генерация профиля и CLI/web rewrites меняли рабочий server.conf до final validation. Теперь все изменения конфига используют зарегистрированный приватный INI stage того же каталога. После успешного final check-config и запрошенного root service helper installer сохраняет предыдущий live file и публикует новый через rename. Ошибки generation/identity/users/validation/backup/publication сохраняют старый конфиг; fresh failed validation не публикует partial file. Обычный отказ/EXIT удаляет temp. Внешние hardlink bytes сохраняются; published config имеет владельца qeli и0600, backup — root0600.
- **F435 / Q33-F038:** отсутствующий/повторный selected profile header допускался, а отказ globals awk скрывался следующим вызовом. Теперь требуется ровно один выбранный header; оба awk явно передают ошибку. Полный stage по-прежнему проверяет Qeli check-config.
- **F436 / Q33-F039:** root service-user drop-in запрашивался до final config validation. Теперь helper вызывается после проверки; его отказ предотвращает публикацию/restart. Изменения самого helper не транзакционные. Default QELI_RUN_AS=qeli пока не сбрасывает прежний root override; service integration OPEN.

## Выполненные проверки и границы

Команда: `sudo unshare --mount --net --pid --fork python3 -B scripts/test_installer_config.py`. Нужны Linux root/chroot, Bash/coreutils/dpkg fixture tooling, jq и openssl. Точный полный installer и неизменённые production functions исполняются в приватных chroot/network/PID namespaces.

Baseline17 methods/22 whole scripts дают22 ожидаемых assertion failures без ошибок setup. Исправлено21method PASS без skipped:28 whole executions,3 positive flows/25 nonzero refusals; точная CI повторяет21/28. Проверены explicit/auto host, panel/boolean options, missing/duplicate profile, оба awk, sed, REALITY randomness, identity, частично записанный третий пользователь, validation, backup, rename и service-user failure. Positive flows: auto-address, forced replacement с external hardlink/backup, public IPv6 endpoint/panel/listener. Успешный return code baseline не означает успешные assertions.

Наблюдения filesystem bytes/inode/UID/mode, Bash/coreutils и openssl настоящие. Qeli CLI identity/users/web/check-config/service-user, accounts, HTTP, systemctl и kernel/tuning commands — явные seams. Real INI parser/Argon2/auth/identity/daemon-health/firewall/VPN/Docker не квалифицированы. Secret fixture теперь учитывает --config; один устаревший recipe assertion на текст ошибки удалён, поведение отказа/публикации/restart проверяет новая серия.

Secrets21/admission18/binary18/updater21 regression methods PASS (всего78), ShellCheck0.10.0 warning/bash syntax,38 mocked recipe tests, native14 SHA/source/A-B provenance и docs9 checks/676 Markdown PASS. Без изменений native/Rust/ABI/core и тяжёлой сборки. Локальная CI-команда не означает новый Actions run.

Отдельно готовится только публикация конфигурации. Users/identity/links/tuning/dependencies/binary/drop-in могут уже измениться. Recovery journal восстанавливает пароли, но не откатывает аккаунты. Ошибка restart/health после публикации не возвращает старый конфиг автоматически. All-or-nothing install, fsync/SIGKILL/power-loss durability и защита от замены ancestor directories не заявлены. Actual CLI/kernel/service integration, прежний root override и Docker OPEN.

Evidence: `release/certification/evidence/q33-installer-config-20261008.json`. Raw `audit-debt-20260924/q33-installer-config-20261008` SEALED:65 файлов/6839726 байт; manifest SHA256 `840af7bd3d22831ecda9dff23f38192b99a3ba1f7cb978aa660cc70911ed177e`. Собственный .11 root SHA-backed-up/guarded-cleaned; service777/845 не менялись, .10 не затрагивался.

Общий план29/37 DONE(78.4%),осталось8. Q31 3/5/capacity-deferred bootstrap/source-cut/current Actions, Q29 SIGKILL/ENONET, Q30 Apple/callback, USER_EXCLUDED платформы и D06 ACCEPTED сохранены. Локальный dev commit, без push/deploy/release.
