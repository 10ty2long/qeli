# qeli 0.8.2 — release preparation / подготовка релиза

**Draft: not published. Черновик: версия не опубликована.**
The 0.8 line remains beta. Линейка 0.8 остаётся бета-версией.

**Language / Язык:** [English](#english) · [Русский](#русский).

## English

### Main changes since 0.8.1

- Configuration files use INI. The panel provides a form and an INI editor;
  its JSON configuration editor has been removed. JSON remains available for
  internal API messages, webhook payloads and diagnostic evidence.
- Client configuration parsing and validation are shared through the native
  core. Common conformance fixtures check the platform adapters; supported
  operating-system features still differ between clients.
- `routing.ipv6.mode = manual` supports deployments where the administrator
  manages external IPv6 forwarding and firewall policy. Qeli retains tunnel
  addressing and client routes and can manage the requested NDP proxy.
  `off`, `route` and `nat66` retain their separate responsibilities.
- The panel and server validate configuration changes before applying them.
  Revision checks protect saves from stale tabs; restore, history, password
  changes and reload reporting have additional consistency checks.
- Network setup, rollback and cleanup handle partial failure and cancellation
  more consistently. Client lifecycle, session ownership and native resource
  publication have targeted race and error-handling fixes.
- Native libraries were refreshed and checked for reproducibility, ABI/JNI
  compatibility and configuration conformance. The dependency update includes
  rustls 0.23.46. Release tooling checks source identity, runtime evidence,
  dependency notices and successful CI before admitting publication.

The complete change list is in [CHANGELOG.md](https://github.com/litvinovtd/qeli/blob/v0.8.2/CHANGELOG.md). The scoped audit
and its evidence are indexed in the [full audit plan](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/FULL-SYSTEM-AUDIT.md).

### Upgrade notes

1. Keep a backup of the current INI files and a copy of the installed version
   before upgrading. Retain the previous configuration with the previous binary
   for rollback; do not assume an older binary accepts newer settings.
2. Notification settings belong in `/etc/qeli/notify.ini`. There is no automatic
   import of `notify.json`. If a legacy file remains, transfer the required
   settings manually; the loader reports the conflict and leaves files intact.
   Restore rejects backups containing the legacy notification JSON sidecar.
3. Use `manual` only when external IPv6 forwarding and firewall rules are managed
   by the administrator. It does not enable them automatically. NDP proxy is
   supported with `manual` and `route`; `ndp_proxy = required` remains a startup
   requirement. See the [IPv6 configuration reference](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/manuals/CONFIG.md).
4. Recheck existing INI files against the current validator. Duplicate sections,
   invalid addresses and unsupported fields can now be rejected instead of being
   silently accepted. In the panel, distinguish a successful save from a change
   that requires a full restart; worker restart does not apply listener changes.
5. On Windows, an untrusted legacy service-profile store is not adopted
   automatically. Preserve the needed user data and explicitly configure the
   service from an elevated GUI when the trust check refuses the old store.

### Android and verification scope

The secure VPN policy and Android's system kill switch remain in place; there is
no DNS bypass compatibility mode. On the measured Android 14/API34 emulator image,
the automatic-family `DnsResolver` call without an explicit network returned
`ENONET` before sending a DNS query. An independent minimal VPN reproduced it.
Ordinary DNS, explicitly selected VPN-network queries and typed A/AAAA queries
have positive evidence in the tested scenarios. The user accepted this remaining
image-specific limitation; it has not been fixed or relabelled PASS. See
[diagnostics and scope](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/reports/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md).

All 37 audit sections were completed within the agreed scope. This does not
certify every physical device or OEM: excluded and unavailable physical checks
retain their original statuses. The final audit source passed all 19 required CI
jobs, and 20 required Linux runtime cases are supported by retained execution
evidence. These are distinct populations, not a combined test count. See the
[audit debt and accepted limits](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/eng/plans/AUDIT-DEBT.md).

### Artifacts and publication

These notes do not announce a release or provide 0.8.2 download links. The final
release commit must pass preflight and CI before tagging. Release packages,
checksums, signing and build provenance must be prepared and verified through
the corresponding platform release paths. A successful source CI run alone
does not provide a downloadable release asset set.

Development pushes to `dev` marked `[candidate]` in the commit message retain
tested candidate outputs in CI for seven days,
with the source commit in each artifact name. These are preparation artifacts,
not GitHub Release assets. Windows distribution archives are
`QeliWin-standalone.zip` and `QeliWin-net-required.zip`. Extract the chosen archive
in full and run its `QeliWin.exe`; retain `Legal/` and the driver notices beside it.
The macOS archives are `Qeli-macos-arm64.tar.gz` and `Qeli-macos-x86_64.tar.gz`.
Choose the matching architecture and extract with `tar -xzf <archive>` to preserve
executable permissions. Both bundles are ad-hoc signed;
the per-app extension requires separate Developer ID/provisioning and notarization.
The retained Android APK is unsigned and requires the release signing key before
it can serve as an upgrade package. Desktop/Android candidate packaging uses the
qualified committed native libraries rather than the development CI overrides.

## Русский

### Основные изменения относительно 0.8.1

- Файлы конфигурации используют INI. В панели доступны форма и INI-редактор;
  JSON-редактор конфигурации удалён. JSON сохраняется для служебного обмена API,
  webhook-сообщений и диагностических evidence.
- Разбор и валидация клиентских конфигов перенесены в общее нативное ядро.
  Общие conformance-fixtures проверяют платформенные адаптеры; набор функций
  операционной системы по-прежнему может различаться между клиентами.
- Добавлен `routing.ipv6.mode = manual` для развёртываний, в которых администратор
  управляет внешним IPv6 forwarding и firewall. За Qeli остаются адреса туннеля,
  клиентские маршруты и управление явно запрошенным NDP proxy.
  У `off`, `route` и `nat66` сохраняются отдельные обязанности.
- Панель и сервер проверяют изменения конфигурации до применения. Проверка ревизий
  защищает сохранение из устаревших вкладок; дополнены проверки restore, истории,
  смены паролей и сообщений о результате reload.
- Настройка сети, откат и очистка последовательнее обрабатывают частичные отказы
  и отмену. В lifecycle клиентов, владении сессиями и публикации нативных ресурсов
  исправлены подтверждённые гонки и ошибки обработки отказов.
- Нативные библиотеки обновлены и проверены на воспроизводимость, совместимость
  ABI/JNI и согласованность конфигурации. Зависимости включают rustls 0.23.46.
  Инструменты выпуска проверяют исходники, runtime-evidence, лицензии зависимостей
  и успешный CI перед допуском публикации.

Полный список изменений находится в [CHANGELOG.md](https://github.com/litvinovtd/qeli/blob/v0.8.2/CHANGELOG.md). Объём аудита
и ссылки на доказательства — в [плане полного аудита](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/FULL-SYSTEM-AUDIT.md).

### Порядок обновления

1. Сохраните текущие INI-файлы и установленную версию до обновления. Для отката
   храните прежнюю конфигурацию вместе с прежним бинарным файлом: старый бинарный
   файл не обязательно понимает новые настройки.
2. Настройки уведомлений находятся в `/etc/qeli/notify.ini`. Автоматического импорта
   `notify.json` нет. Если остался старый файл, перенесите нужные настройки вручную;
   загрузчик сообщает о конфликте и не меняет файлы. Restore отклоняет архивы
   со старым JSON-файлом уведомлений.
3. Выбирайте `manual`, когда внешними IPv6 forwarding и firewall управляет
   администратор. Этот режим не включает их автоматически. NDP proxy поддерживается
   в `manual` и `route`; `ndp_proxy = required` остаётся условием запуска.
   См. [справочник IPv6-конфигурации](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/manuals/CONFIG.md).
4. Проверьте существующие INI текущим валидатором. Повторные секции, некорректные
   адреса и неподдерживаемые поля могут теперь отклоняться вместо молчаливого
   принятия. В панели различайте успешное сохранение и изменение, требующее
   полного перезапуска: worker restart не применяет изменения listener.
5. В Windows старое недоверенное хранилище сервисного профиля не принимается
   автоматически. Сохраните нужные пользовательские данные и явно настройте
   службу из elevated GUI, если проверка доверия отвергает старое хранилище.

### Android и границы проверки

Сохраняются защищённая VPN-политика и системный kill switch Android; отдельного
DNS bypass-режима нет. На проверенном эмуляторе Android 14/API34 автоматический
выбор семейства адресов в `DnsResolver` без явно указанной сети возвращал `ENONET`
до отправки DNS-запроса. Независимый минимальный VPN воспроизвёл этот результат.
Обычный DNS, запросы с явно выбранной VPN-сетью и типизированные A/AAAA имеют
положительные результаты в проверенных сценариях. Пользователь принял оставшееся
ограничение данного образа; оно не исправлено и не переименовано в PASS.
См. [диагностику и границы результата](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/reports/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md).

Все 37 разделов аудита завершены в согласованном объёме. Это не подтверждает
каждое физическое устройство или OEM: исключённые и недоступные физические
проверки сохраняют исходные статусы. Итоговые исходники аудита прошли 19 обязательных
заданий CI, для 20 обязательных Linux runtime-сценариев сохранены доказательства
исполнения. Это разные наборы проверок, их числа не складываются.
См. [реестр техдолга и принятые ограничения](https://github.com/litvinovtd/qeli/blob/v0.8.2/docs/ru/plans/AUDIT-DEBT.md).

### Артефакты и публикация

Эти заметки не объявляют выпуск и не содержат ссылок на скачивание 0.8.2. Итоговый
релизный commit должен пройти preflight и CI до создания тега. Пакеты, контрольные
суммы, подписи и происхождение сборок необходимо подготовить и проверить через
соответствующие платформенные процедуры выпуска. Успешный CI исходников сам по
себе не предоставляет набор скачиваемых релизных артефактов.

Push в `dev` с пометкой `[candidate]` в сообщении коммита сохраняет проверенные
результаты сборки в CI на семь дней; имя каждого
артефакта содержит commit исходников. Это подготовительные файлы, а не assets
GitHub Release. Windows поставляется в `QeliWin-standalone.zip` и
`QeliWin-net-required.zip`. Распакуйте выбранный архив целиком и запустите его
`QeliWin.exe`, сохраняя `Legal/` и notices драйверов рядом.
Архивы macOS — `Qeli-macos-arm64.tar.gz` и `Qeli-macos-x86_64.tar.gz`.
Выберите нужную архитектуру и распакуйте `tar -xzf <archive>`, чтобы сохранить
права запуска. Оба пакета имеют ad-hoc подпись;
per-app extension требует отдельных Developer ID/provisioning и notarization.
Сохраняемый Android APK не подписан: для пакета обновления нужен релизный ключ.
При упаковке desktop/Android используются квалифицированные нативные библиотеки
из Git, а не временные development CI overrides.
