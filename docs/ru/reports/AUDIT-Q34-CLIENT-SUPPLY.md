# Q34: зависимости сборки клиентов и подписи Windows

<!-- normative-sync: q34-client-supply-v3 -->

8 октября 2026. F455–F458 / Q34-F009–F012 закрыты в указанном ниже объёме. F459 / Q34-F013 остаётся OPEN. Q34 IN_PROGRESS; полный аудит 30/37 DONE (81.1%), осталось7. Только локальные изменения; без push, deployment и release.

## Находки и исправления

| Находка | Результат |
|---|---|
| F455 / Q34-F009 | npm audit обнаружил selector-parser GHSA-rj75-hqrm-r3gf и source-map-js GHSA-68fv-2mgg-jv7q. Точные overrides7.1.6 и1.2.2 изменили только две записи lockfile. Чистые npm ci --ignore-scripts и сборка CSS PASS; встроенный CSS побайтно прежний. |
| F456 / Q34-F010 | Версия Gradle wrapper закреплена, но checksum скачиваемого архива отсутствовал. Добавлен официальный SHA256 Gradle9.7.1 acd53f1edaf02f1a8ff99879f8a34b302661a057d9b063ae9e35b552f804d20a. Настоящий wrapper принимает проверенный официальный ZIP и отклоняет изменённый ZIP до запуска Gradle. Проверка wrapper-JAR в CI сохранена. |
| F457 / Q34-F011 | Предупреждения NuGet audit не останавливали обычный restore/build. Корневой Directory.Build.props проверяет прямые и транзитивные пакеты начиная с low; NU1900–NU1905 являются ошибками. NuGet.Config задаёт явный официальный источник advisory, не меняя package sources. Три Release restore/build PASS; реальные fixtures с уязвимым пакетом и недоступным источником advisory отклоняются. |
| F458 / Q34-F012 | Windows gate проверял подписи только Wintun. Расширена существующая проверка обеих копий официального WinDivert2.2.2-A: хеши DLL/драйвера/лицензии/отдельных notices, FileVersion драйвера2.2, Valid Authenticode и thumbprint043589F75FCE2795E7F2CC3E526D46784D5DDAB3. Положительный сценарий и четыре реальных отказа на повреждённых/отсутствующих файлах PASS. |
| F459 / Q34-F013 OPEN | Для braces GHSA-vfj7-8cjw-p6xm исправленной опубликованной версии на момент проверки нет: latest3.0.3 и Tailwind v3-lts3.4.19 сохраняют зависимость. Текущий npm audit возвращает1: пять затронутых записей графа от одного advisory. Подавления и заявления о нуле уязвимостей нет; жёсткий npm gate ещё не закрыт. |

npm-пакеты собирают доверенные CSS/templates репозитория и не используются как runtime-парсер VPN/панели. Клиентские INI не менялись. Миграция на Tailwind4 меняет контракт сборки/конфигурации и требует отдельной проверки совместимости CSS. Пробная неопубликованная braces3.0.4 отклонена npm и в репозиторий не попала. Открытый пункт нужно пересмотреть после upstream-исправления либо квалифицированной замены.

## Выполненные проверки и границы

Собственные свежие package/audit cache NuGet, SDK10.0.300: Release build/restore Windows, managed macOS cross-build и QeliConformance PASS. В этих трёх Release-графах известных уязвимостей не обнаружено, включая транзитивные пакеты. Это не квалификация Debug/headless/Linux-зависимостей, актуальности SDK или отсутствия неизвестных дефектов. Обязательный общий conformance с полным tracked-корпусом PASS; selftest Windows-адаптеров PASS. Локальный PacketCodec benchmark:1400bytes,20000iterations,260.953ms,102.328MiB/s; это codec gate, не скорость VPN. Первая неполная scratch-копия провалила восемь обязательных fixture-проверок; полный корпус повторно прошёл их без отключения требований.

Настоящий fixture System.Text.Json8.0.0 отклонён с NU1903. Недоступный только в дочернем fixture источник advisory отклонён с NU1900/NU1905. Глобальные источники NuGet и сетевые настройки ОС не редактировались. Первый запуск SDK сообщил о настройке development HTTPS certificate; исходный лог сохранён. Политика действует для проектов внутри репозитория; отдельные урезанные source-копии должны сохранять корневые файлы политики сборки. Lockfiles .NET и политика SDK остаются отдельной работой Q34.

Независимо скачан [официальный WinDivert2.2.2-A](https://github.com/basil00/WinDivert/releases/tag/v2.2.2); оба x64-файла совпали с bundled по SHA256. В официальном ZIP пользовательская DLL не подписана, SYS имеет Valid-подпись; gate проверяет подпись SYS и закрепляет байты DLL. Notices намеренно различаются и проверяются отдельно. Нет установки драйвера, WinDivert filter, VPN-маршрутизации либо квалификации Windows network VM. Положительный wrapper запускает только Gradle --version с собственным GRADLE_USER_HOME; проверки APK/runtime здесь нет.

Scratch/caches/output находятся в собственных локальных каталогах. SSH, лаба и секреты не использованы. Rust/native source digest прежний: e3fb04a80fc8c2e7334fa5187dacb497ca47b2a79954fcddae45b5d5a533052c; существующие four-target A/B, копии и14SHA подтверждены без пересборки неизменённых Rust inputs. app.css, CHANGELOG и users.html побайтно прежние. Ошибки fixture/encoding/notice-предположений записаны как ошибки harness, без объявления продуктовой регрессии.

`q34-client-supply-20261008`: SEALED 88 files / 404870232 bytes; manifest SHA256 `049420ad624cd500958d4b0a9dacefa18e4142f32ece4d814259da8e36e0e66a`.

Открытые части Q34: SDK, locking/verification .NET/Android, остальные dependency graphs, release payload и текущий exact-SHA Actions admission. Old14 debtDONE+D06accepted; Q29SIGKILLFAIL/ENONET, Q30Apple/callbackOPEN, Q31capacity/source-cut/currentActions и USER_EXCLUDED Mac/iOS/router/Windows network VM сохранены. Нет нового Android runtime/soak либо physical Mac результата.

Источники политики: [Gradle wrapper checksums](https://docs.gradle.org/current/userguide/gradle_wrapper.html#sec:verification), [NuGet audit](https://learn.microsoft.com/en-us/nuget/concepts/auditing-packages), [selector-parser advisory](https://github.com/postcss/postcss-selector-parser/security/advisories/GHSA-rj75-hqrm-r3gf), [braces advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm).

## 9 октября: F491 / Q34-F045 — APK-builder не должен останавливать чужой Gradle

`scripts/rebuild_apk.py` перед сборкой вызывал глобальный
`pkill -9 -f GradleDaemon` и удалял `/root/.gradle/caches/journal-1`, хотя
собственная Gradle-команда уже использует `--no-daemon`. Имя процесса не
обозначает владельца сборки; этот шаг мог убить другую задачу и повредить её cache.
Глобальная остановка и удаление общего журнала убраны. Gradle управляет своими
блокировками. Команды Rust/Gradle, offline, один worker, зеркалирование исходников,
проверка release-подписи и SHA-загрузка APK сохранены.

Исполнение вынесено в `build`/`main`: импорт больше не подключается к лабе,
ошибочные аргументы отклоняются до SSH, а `finally` закрывает SFTP и SSH при
ошибках открытия/сборки/закрытия. CLI по-прежнему принимает debug по умолчанию
или `--release`; ошибка сборки/подписи завершает CLI неуспешно. Новые тесты
подключены к постоянному CI.

В собственных PID/mount namespaces .11 прежняя точная команда реально убила
подставной сторонний `GradleDaemon` и свой shell, оба с returncode -9.
Shared-cache path был закрыт приватным bind mount; host-процессы и настоящий
cache не были доступны этому воспроизведению. Удаление журнала прежней командой
не заявлено: shell был убит раньше. Исправленная настоящая оркестрация запускает
контролируемый `gradlew`, который возвращает compile failure7; builder правильно
отказывает, а сторонний процесс остаётся жив и marker cache побайтно прежним.
Это проверка процесса и отказа, не запуск настоящего Gradle/compiler/APK.

На Windows:7 новых lifecycle/orchestration методов,38 native recipe и11 workflow
методов PASS. На Linux: те же7 новых методов и отдельное настоящее воспроизведение
конкуренции PASS. Официальный SHA-проверенный actionlint1.7.12 принял четыре
workflow. Входы и полные Linux-логи восстановлены с SHA, собственный каталог удалён
по точному списку после process/fd/mount guards; служба .11,PID777/845 и установленный
CLI75bc сохранены. .10 не использовалась. Исходники ядра/native recipe,14 native SHA,
protected files и прежний OpenWrt source cut неизменны; новый APK не собирался.

Граница: это устранение глобальной остановки и повреждения общего cache.
Builder по-прежнему использует существующие фиксированные source directories;
одновременные APK-сборки разных исходников в этих же directories этим пакетом
не квалифицированы. Release-проверка `apksigner` подтверждает действительную
подпись, но не ожидаемого издателя; текущий подписанный release payload и его
издатель остаются отдельным открытым требованием. F459 braces OPEN; подавлений нет.
Q34 IN_PROGRESS,30/37 DONE81,1%,7 разделов остаются;Q29 SIGKILL FAIL/ENONET,Q30/Q31,
D06 ACCEPTED_LIMITATION и USER_EXCLUDED сохранены. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F491 evidence: `q34-apk-builder-20261009`; SEALED 36 files / 1205550 bytes; manifest SHA256 `bf6a65063b2ee57f87813097cb2601d15d075dffa25ebf0a9007139abcaa8bf9`.


## 9 октября: F492 / Q34-F046 — отдельный полный snapshot для каждой APK-сборки

Закрыт дефект общего source workspace сборщика: разные вызовы записывали файлы
в `/root/android-project` и `/root/qeli-src`, поэтому могли смешать исходники и
выходы. Старый фильтр расширений также пропускал 11 tracked-файлов Android,
включая Gradle lockfiles, Java instrumentation receiver и legal inputs; conformance
помещался внутрь Android project вместо соседнего каталога.

Теперь сборщик до SSH требует чистый committed Git tree без скрытых trust flags,
проверяет native provenance и создаёт полный Git snapshot `qeli`, `qeli-android`,
`conformance`, `LICENSE`, `LICENSING.md`. Он сверяет список и Git blob каждого
члена архива, отказывает symlink/unsafe members и export-преобразованиям. На .11
каждый вызов получает собственный `/var/tmp/qeli-apk-build.XXXXXXXX`, свой
Cargo target и Gradle project state. Полный список SHA проверяется после передачи
и после сборки. Native JNI directory задаётся явно. APK и `BUILD.json` сохраняются
в `qeli-android/dist/<commit>-<run>/`, без перезаписи результатов другого вызова.
Неуспешный remote checkout сохраняется для диагностики; автоматической очистки нет.

CLI: debug по умолчанию, `--release` для release. По умолчанию Gradle offline;
`--online` явно разрешает недостающие зависимости при прежних strict locks/checksums.
`--gradle-user-home` и `--android-sdk` принимают абсолютные пути на удалённой лабе.
`--signing-properties` также remote path и допустим только с `--release`;
legacy default `/root/android-project/keystore.properties` остаётся только источником
ключевых параметров. Java Properties корректно разбирает escapes и относительный
`storeFile`; приватная копия с umask077 удаляется в finally. `apksigner` выбирается
из заданного SDK. Это проверено synthetic fixtures, без production release signing.

Реальный snapshot:1046 файлов,25896960 bytes, artifact source commit
`40c644229eb3f91429d6ead272e2428e2ebd320a`. Проверенный builder commit
`82d31cb3bd8567c386236f45668f7f0c6159a890` использует те же выбранные исходники.
Два настоящих каталога были развёрнуты независимо; изменение settings во втором
вызвало SHA-отказ, первый собрал реальный debug APK. Второй конкурентный compiler
не запускался. Свежий host Rust JNI собран за97,89s; финальная проверка использовала
его cache. JVM: 201 tests,0 failures/errors,0 skipped. APK SHA256
`1525e14c4098bca83e3f1557f29c5c9a1d1c85878042c1ea90ef97815b4027ee`; обе JNI совпали с snapshot, упаковка лицензий
прошла полноценный существующий gate; debug-подпись действительна.

71 Windows и22 финальных Linux regression methods PASS; официальный actionlint
проверил4workflow, docs724Markdown/9checks PASS. Сохранены первые отказы: отсутствующий
AGP в offline-cache, неполная SDK-копия harness и ENOSPC при локальном дубликате
Maven-cache. Locks/verification не отключались. Повторный APK использовал явно
разрешённый online-режим, приватные cache/wrapper/SDK из проверенных источников;
нужные Build Tools36.0/licenses скопированы read-only с существующей лабы;
Gradle скачал Platform Tools37.0.1 только в приватный SDK. Последовательное SFTP-чтение
host JNI остановлено уже после успешного build/publication; выход восстановлен через
ограниченный prefetch16 с независимой SHA-проверкой, без повторной компиляции.
Неполный новый архив удалён; освобождены только прежние собственные regenerable
publish-каталоги после проверки2619SHA и сохранности уникальных Qeli-файлов.

APK, host JNI, JUnit XML и все сохраняемые исходники/логи восстановлены с SHA,
оба own remote roots удалены после process/fd/mount guards. Compiler/dependency/SDK
cache исключены из recovery archive как воспроизводимые; их происхождение и hashes
зафиксированы отдельно. Служба .11,PID777/845, установленный CLI75bc сохранены;
.10 не использовалась. Ядро,14 native SHA,protected files/OpenWrt cut неизменны.
Нет установки APK, emulator/VPN runtime, release/expected-publisher qualification
или hosted Actions. Ожидаемый release-издатель и F459 braces OPEN остаются.
Q34 IN_PROGRESS;30/37 DONE81,1%,7 остаются;Q29 SIGKILL FAIL/ENONET,Q30/Q31,
D06 ACCEPTED_LIMITATION,USER_EXCLUDED сохранены.LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F492 evidence: `q34-apk-snapshot-20261009`; SEALED 127 files / 180541943 bytes; manifest SHA256 `5db226ad94b5c7e764973618383df4955b4b845abd36254b1d9d544907ee3f28`.
