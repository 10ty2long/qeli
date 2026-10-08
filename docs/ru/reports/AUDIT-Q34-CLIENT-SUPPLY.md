# Q34: зависимости сборки клиентов и подписи Windows

<!-- normative-sync: q34-client-supply-v1 -->

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
