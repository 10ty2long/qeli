# Q29: Android VPN recovery after process death

13 October 2026: Qeli now has a private same-process `VpnRecoveryService`. Android can restart its sticky record with a null intent; it requests recovery once after checking persisted connection intent and VPN consent. The primary service restores the encrypted INI snapshot of the requested connection and ignores duplicate callbacks when a controller is already active. Subsequent profile selection does not select the recovery configuration.

The snapshot uses the existing AES-GCM/Android Keystore store, separate from profile metadata. No credentials are added to plaintext preferences or restart intents. Snapshot persistence failure rejects a new connection/reconfiguration. Disconnect and onRevoke stop the recovery record and erase the snapshot. Both foreground records share the existing notification and Disconnect action. No timer, polling, BPF or change to system lockdown is involved.

Validation: 201 JVM tests, lint with no Error/Fatal, release R8, both unchanged JNI binaries and inferred instrumentation API rules. Five targeted manifest/encryption Android tests passed on Android14/API34 x86_64. The instrumented application was then uninstalled; authentication and network checks used the ordinary production R8 APK with a lab certificate. All original ZIP entries were preserved, with only standard signature entries added. Real release signing was not qualified.

Actual SIGKILL recovery: PID 2202 → 3107, authenticated NetworkPlan in 2.99s, one new VPN generation and independent-UID payload confirmed by the server from the VPN subnet. UI Disconnect produced no service resurrection for 30s and lockdown blocked traffic; reconnect passed IPv4/IPv6 TCP/UDP. Force-stop produced no resurrection for 30s, blocked traffic and successful manual recovery. Settings revocation persisted desired=false and left both services/TUN absent after 30s. Host services and base AVD userdata were preserved; the entire owned runtime root was archived and removed.

These are bounded observations on one API34 image, not universal Android/OEM restart timing guarantees. Earlier failed controls and historical redelivery failure remain retained. The standalone Java prototype's two successful recoveries (2.64/1.86s) are separate evidence; the product result above uses actual Qeli.

Q29 remains IN_PROGRESS: no transparent workaround for third-party generic/null DnsResolver ENONET was found, and the user did not accept it as a limitation. `allowBypass` remains disabled. Physical platforms, whole audit, hosted CI and publication remain unqualified. Plan: 33/37 (89.2%). NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q29-product-recovery-20261013.json`.
