# Лицензирование qeli

Это монорепозиторий с **несколькими лицензиями по каталогам**. GitHub в шапке
показывает только одну (корневую — AGPL-3.0); полная карта — здесь.

## Карта лицензий

| Путь | Лицензия | SPDX | Что это |
|---|---|---|---|
| `/` (по умолчанию для всего, что не отмечено ниже) | GNU AGPL v3 | `AGPL-3.0-only` | копилефт, сетевая оговорка §13 |
| `qeli/` (Rust-ядро + сервер + realtls-FFI) | GNU AGPL v3 | `AGPL-3.0-only` | ядро протокола и серверная часть |
| `qeli-android/` | MPL 2.0 | `MPL-2.0` | исходники Android-клиента (UI/glue) |
| `qeli-win/` | MPL 2.0 | `MPL-2.0` | исходники Windows-клиента (UI/glue) |
| `qeli-mac/` | MPL 2.0 | `MPL-2.0` | исходники macOS-клиента (UI/glue) |
| `qeli-ios/` | MPL 2.0 | `MPL-2.0` | исходники iOS-клиента (UI/glue) |
| `qeli/third-party-notices/rust/` | по upstream | — | неизменённые тексты сторонних Cargo-зависимостей; их лицензии не наследуют AGPL каталога |
| `native-libs/third-party/` | по upstream | — | сторонние бинари (напр. Wintun) под их лицензиями |

Тексты: корень и `qeli/` — `LICENSE` (AGPL-3.0); каждый клиентский каталог — свой
`LICENSE` (MPL-2.0).

> **Не отмечены отдельно, поэтому наследуют корневой AGPL-3.0:** `qeli-shared/` (общее
> C#-ядро, на которое ссылаются MPL-клиенты Windows и macOS) и `qeli-openwrt/`. Своего
> файла `LICENSE` у них нет. Если для них задумывалась MPL, как у остальных клиентских
> каталогов, нужно положить туда `LICENSE` и добавить строку в таблицу — до этого
> действует правило по умолчанию.

## Важно: общее нативное ядро (FFI) и его влияние на клиенты

Все клиенты бандлят нативную библиотеку **`libqeli`** (`libqeli.so` / `qeli.dll` /
`libqeli.dylib`), собранную из кода `qeli/` под **AGPL-3.0**. Поэтому:

- **Исходники клиента** (UI/платформенный glue в `qeli-*/`) — под **MPL-2.0**: их можно
  переиспользовать пермиссивно, например со СВОИМ backend, **без** ядра `qeli`.
- **Распространяемое приложение целиком** (клиент **вместе с** бандленым `libqeli`)
  для третьих лиц является производной от AGPL-кода и распространяется на условиях
  **AGPL-3.0**. MPL на клиентских исходниках этого не отменяет.

То есть MPL даёт ценность «переиспользуй UI отдельно», но «возьми наш клиент с нашим
ядром и закрой» — нельзя (это AGPL).

## Копирайт и вклады

Правообладатель — владелец репозитория; он сохраняет права на собственный код и
вправе выпускать официальные сборки и **отдельные закрытые компоненты** (например
коммерческий control-plane — это его собственный код, не производный от вкладов).

**Вклады принимаются по модели DCO, без CLA** (см. [CONTRIBUTING.md](CONTRIBUTING.md)):
каждый вклад входит под лицензией своего каталога (AGPL-3.0 для `qeli/`, MPL-2.0 для
клиентов) — «inbound = outbound». Авторские права контрибьюторы сохраняют за собой.

Монетизация не требует продажи ядра под проприетарной лицензией (см. модель: хостинг +
отдельный закрытый control-plane + поддержка), поэтому **двойное лицензирование ядра
не ведётся и CLA не нужен**. Если когда-нибудь понадобится продавать само ядро под
закрытой лицензией — это потребует согласия всех контрибьюторов либо введения CLA на
тот момент (ретроактивно сложнее).

## Сторонние зависимости

Rust-крейты (rustls/ring/ml-kem/…), .NET-пакеты (Avalonia, SkiaSharp, BouncyCastle,
QRCoder, …), Kotlin/Android-библиотеки — каждая под СВОЕЙ лицензией.
Для Rust CI `security-audit` запускает `cargo deny --all-features --locked check
licenses bans sources` с политикой `qeli/deny.toml`: новые неразрешённые лицензии
блокируют проверку. Исключение `AGPL-3.0-only` ограничено самим крейтом `qeli`
и отражает существующую лицензию ядра; лицензии файлов и клиентов не меняются.
`cargo audit --deny warnings` отдельно проверяет RustSec и отозванные версии;
версии обоих инструментов закреплены в workflow. Проверки запускаются из `qeli/`.

Эта автоматизация проверяет Rust-граф зависимостей. .NET/Kotlin-зависимости и
сторонние нативные бинари (Wintun и пр., `native-libs/third-party/`) требуют
отдельной проверки upstream-лицензий при изменении или выпуске пакета.

## Тексты в Android APK

Сборка каждого варианта APK включает канонические AGPL/MPL тексты, Apache 2.0,
реестр Maven runtime-зависимостей и найденные в исходных JAR/AAR LICENSE/NOTICE.
Инвентарь версий проверяется при сборке; CI отдельно сверяет индекс и SHA256
текстов в готовом APK. См. [порядок проверки и обновления](qeli-android/README.md).

APK также включает неизменённый общий комплект Cargo legal resources. Gate
проверяет его побайтное совпадение с репозиторием. Доступность соответствующих
исходников, toolchain notices и полное release admission проверяются отдельно.
Лицензии каталогов не изменяются.

## Общий комплект Cargo legal resources

`qeli/third-party-notices/rust/` содержит тексты upstream из всех registry-пакетов
закреплённого Cargo.lock: LICENSE/NOTICE/COPYING/COPYRIGHT/UNLICENSE/AUTHORS,
каталоги LICENSES и объявленный `license-file`. SPDX expressions сохранены
из опубликованных Cargo.toml. Это полный lockfile, включая server/build/test
и зависимости других targets; наличие записи не означает linkage в каждом клиенте.
Оригинальные байты не нормализуются; `INDEX.tsv` хранит SHA256 и origin.
Для архивов без текстов используются только отдельно проверенные supplements
с VCS-коммита, записанного в опубликованном `.cargo_vcs_info.json`.

Проверка без Cargo/сети: `python3 scripts/native_licenses.py`. Обновление графа
требует обновить комплект; старый Cargo.lock binding блокирует проверку.
Генерация принимает каталог `.crate` архивов и каталог reviewed supplements
(`receipts.json` плюс payload-файлы) и пишет только в новый каталог:

```bash
python3 scripts/native_licenses.py --archives /path/to/crates --supplements /path/to/reviewed-supplements --bundle /path/to/new-bundle
```

SHA каждого архива должен совпадать с Cargo.lock. Receipt фиксирует name/version,
VCS commit, upstream path/URL, payload path и SHA256; коммит и URL сверяются
с архивом. После review заменить общий комплект, сохранив upstream bytes.
JSON здесь — build metadata, не формат конфигурации VPN.

Android: `assets/licenses/native-rust/`; Windows/macOS: `Legal/NativeRust/`
рядом с опубликованным executable; iOS: folder resource `rust` в app;
Debian: `/usr/share/doc/qeli/licenses/rust/`; OpenWrt:
`/usr/share/licenses/qeli/rust/`; Keenetic: `qeli-native-legal.zip` рядом
с отдельными бинарями. Windows/macOS дополнительно копируют MPL клиента;
общий комплект содержит AGPL ядра. При распространении сохранять sidecar/resources.
CI проверяет общий комплект, а APK gate сверяет также готовые APK.
Rust std/toolchain, source publication и подписи остаются отдельными
проверками; этот комплект не является заявлением полной юридической квалификации.


## Общий комплект .NET legal resources

`qeli-shared/third-party-notices/dotnet/` содержит notices всех reviewed NuGet
lock-графов desktop-клиентов и четырёх runtime-пакетов default publish из
`qeli-shared/third-party-notices/nuget-runtime.tsv`. Это superset, включая
Debug/tools/Linux/другие RID; это не перечень linkage каждого executable.
Windows/macOS publish сохраняет весь каталог как `Legal/DotNet/`, рядом с
`Legal/NativeRust/` и MPL клиента. При создании Windows архивов сохранять весь
`Legal`; macOS app включает его в `Contents/MacOS/Legal/`. CI проверяет оба
RID/Windows-варианта после publish; `build_app.sh` проверяет notices до сборки app.

Проверка без SDK/сети: `python3 scripts/managed_licenses.py`. Она связывает bundle
со всеми reviewed `packages*.lock.json`, `global.json` и runtime TSV и проверяет
индекс/SHA, nuspec identity/license, supplements и отдельную OFL Inter.
NuGet license expression не заменяет найденные upstream LICENSE/NOTICE/copyright.
В `Avalonia.Fonts.Inter` код объявлен MIT, а встроенные шрифты — SIL OFL 1.1;
сохранены оба текста и copyright из реальных font name records.

Обновление: обновить locks и reviewed runtime TSV после квалификации default
publish графов, проверить архивы по official NuGet catalog SHA512, подготовить
`admissions.tsv` (header `id\tversion\tcontent_hash\tarchive_sha256\tpath`,
нижний регистр id/version, абсолютный path) и reviewed upstream resources.
Signed `.nupkg` SHA512 архива отличается от NuGet `contentHash`: генератор
вычисляет contentHash через официальный API NuGet из **точного** SDK global.json
и проверяет CMS signature integrity. Это не проверка certificate trust/revocation.
SDK helper создаётся вне checkout, не имеет PackageReferences и собирается с
пустым списком NuGet packageSources.

```bash
python3 scripts/managed_licenses.py --admissions /path/to/admissions.tsv --upstream /path/to/reviewed-upstream --output /path/to/new-bundle --dotnet /path/to/pinned-sdk/dotnet
```

Генератор пишет только в новый каталог. `receipts.json` имеет schema=1 и rows с
repository (`owner/repo`), commit, upstream_path, exact raw URL, sha256, payload
и binding=`nuspec-vcs`; все receipts должны использоваться. Commit/repository
сверяются с опубликованным nuspec. Исходные байты не нормализуются. Для текущего
MicroCom.Runtime 0.11.0 нет packaged VCS commit: supplement явно обозначен
`publisher-declaration; no nuspec-vcs commit`, historical LICENSE совпадает с
MIT/copyright издателя; точная source-to-binary привязка не утверждается.

Inter дополнительно требует `inter-font-proof.json` (status=PASS, rows с path,
url, sha256, git_blob, names) и шесть upstream TTF с исходными basename. Генератор
перепроверяет Git blob/SHA, SFNT name records 0/5/13/14 и наличие точных байтов во
всех packaged TFM DLL; OFL receipt связывается с git-version шрифта. JSON здесь
служит build evidence; конфигурации VPN остаются INI.

После review заменить общий bundle, выполнить checker/tests и publish для двух
Windows и двух macOS вариантов, проверить готовые `Legal/DotNet` каталоги.
`build_app.sh` использует Python 3.11+ (`PYTHON` позволяет выбрать executable).
Иные framework/runtime overrides, toolchain notices, source publication,
сертификатные подписи и full release admission квалифицируются отдельно.

## SPDX-заголовки (план)

При следующем проходе рекомендуется добавить в каждый файл строку
`SPDX-License-Identifier: AGPL-3.0-only` (в `qeli/`) или `MPL-2.0` (в клиентах) и
принять стандарт **REUSE** (`reuse lint` в CI) для машиночитаемой проверки. Выбор
`-only` против `-or-later` для AGPL — на усмотрение правообладателя (FSF рекомендует
`-or-later`; `-only` даёт больше контроля).
