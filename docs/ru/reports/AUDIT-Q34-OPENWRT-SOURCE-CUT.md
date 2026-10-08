# Q34: точный локальный source cut OpenWrt

<!-- normative-sync: q34-openwrt-source-cut-v1 -->

Дата: 9 октября 2026.

## F483 / Q34-F037

Прежний pin dd0b736276b9ad5b89d117abd1bfe324210f3b26 содержал другой qeli tree.
Production source gate ожидаемо отклонял его. Рецепт теперь ссылается на точный
полный локальный dev commit f4b80a9a3052e9ec12c640036928d543f931dafa и SHA256 d6c112641de77b446d3a32a702221b247556b873c11f648087aebb57d3b209c5
настоящего SDK-архива qeli-0.8.2.tar.zst. Commit не создавался заново: полный
исходный commit/tree перенесён Git pack, отсутствующие родители обозначены
shallow boundary. Все 3131 tracked files после SDK prepare совпали с Git,
включая исходники, лицензии и notices. Native digest и 14 артефактов неизменны.

Официальный OpenWrt24.10.4 SDK проверен по SHA256
229e871f734a2cee5ce3ad6a3e98d3836b0899bfdeaea4d9c2c5cc7b1fce1407.
Его metadata/scripts/host tools перенесены без изменения байтов; неиспользуемые
целевой toolchain/build trees исключены. Частные gawk/dependency extraction и
pinned Rust feed/config использованы для download/prepare без установки в host.
Это проверка исходного архива; компилятор, Cargo, bootstrap и IPK не запускались.
[Официальные checksum](https://downloads.openwrt.org/releases/24.10.4/targets/x86/64/sha256sums).

## Проверки и границы

Два настоящих SDK download, разные собственные Git file URLs, пустые DL_DIR:
одинаковые 26381218 байт и SHA256. Первый Git download при PKG_SOURCE_MIRROR=0 не проверял старый hash;
это не допуск к prepare. Измеренный hash затем успешно проверил
production verify-source.sh в SDK prepare. Второй download использовал
новый hash в рецепте; байты независимо сверены после создания архива.
SDK source archive не подменялся git archive или вручную созданным tar.

Public URL остался https://github.com/litvinovtd/qeli.git. Read-only запрос
GitHub для f4b80a9a3052e9ec12c640036928d543f931dafa вернул HTTP422 No commit found; опубликованный dev
был 83c07f5c31178c67a005f20f59975e14528ab73a. Перед использованием публичного
feed необходимо опубликовать точный pinned commit. Source cut закрыт локально;
public clone/наличие revision и current Actions остаются OPEN. Более поздняя
правка qeli tree требует нового cut/hash. Версия пакета остаётся 0.8.2.

75 методов PASS:25source gate,12whole-preflight,38recipe. Три прежних
source-suite skips (POSIX symlink/FIFO и opt-in baseline) сохранены. Настоящий
gate на committed production recipe принял реальный SDK archive и отклонил
изменённый/отсутствующий архив. Начальный стенд не имел sdk/tmp: исходный FAIL66
сохранён, создан только свой каталог. Два SSH launch-channel timeout10s
сохранены как диагностика controller; jobs продолжились и успешно завершились,
повторно не запускались. Rust/compiler/runtime tests не подменены этими fixtures.

Лаба .11: только собственный временный root, jobs1, без изменений рабочих
сервисов, системных packages/firewall/sysctl. Исходные service/PID/hash проверены
после работы; recovery/archive и guarded cleanup приведены в raw evidence.
Лаба .10 не использовалась. Credential supplied through stdin only; raw and decoded archive credential scan PASS.

Общий план: Q34 IN_PROGRESS; 30/37 DONE (81,1%), 7 осталось. Q29 SIGKILL
FAIL/ENONET, Q30/Q31, F459 braces OPEN, D06 ACCEPTED_LIMITATION и USER_EXCLUDED
сохранены. Новые compiler/native A/B, platform/runtime/hostedCI/signatures/release
не заявлены. LOCAL_DEV_COMMIT; NO_PUSH_DEPLOY_RELEASE.

## Evidence

`q34-openwrt-source-cut-20261009`: SEALED 81 files / 261832009 bytes; manifest SHA256 `3e57c16820e13a9177f2e8339779d50d09f41573357ae6eed5e6bfa21b8aaa98`.
