# Q37: documentation and test harnesses — 11 October 2026

<!-- normative-sync: q37-documentation-baseline-v2 -->

**Status: IN_PROGRESS.** [Results and hashes](../../../release/audit_q37_documentation_2026-10-11.json). Full Q37 is incomplete. Overall result remains 30/37 DONE (81.1%).

Q37-F001: the updater documentation edit removed versioned attestation examples from OPERATIONS while `sync_version.py` still required them in RU/EN. The baseline failed on two unmatched patterns. Both manuals now include `gh attestation verify` for the published amd64 0.8.1 package against litvinovtd/qeli; strict version checks remain mandatory. Five methods across RU/EN exercised current, stale, missing, development-versus-release examples and exact CRLF-preserving write repair. Actual `sync_version.py` and nine documentation checks PASS.

Q37-F002: `site/` is intentionally private/ignored; the test could not run from a separate Git checkout without its inputs. `--site-root PATH` selects the actual private website. Real RU/EN hub/install pages and clipboard behavior PASS; five negative scenarios (absent default inputs, invalid directory, unknown option, missing option value, removed event handlers in a temporary copy) refused. Five website input hashes match before/after. Missing website data never becomes SKIP/PASS. Website sources/publication were unchanged; this test does not qualify the whole static website.

The baseline also passed panel checks (11 templates/1160 RU strings), 118 editor checks and documentation. Additional 97 native harness tests and 18 certification gate tests PASS. These are tool unit/fixture checks, not actual A/B builds or successful release certification.


Q37-F003: [unreachable legacy implementation removal](../../../release/audit_q37_dead_legacy_2026-10-11.json). Removed6320 lines after unconditional refusal from41 Python/shell files. AST proves only module docstrings and future annotations can precede Python SystemExit; each shell stub contains only shebang, stderr echo and exit1. Names/refusal messages remain so historical commands cannot become active; two messages no longer advertise the removed format. Unused scripts/client-config.json (1510bytes) was deleted; its only reference is a frozen historical fuzz-source inventory, retained unchanged. Total239053bytes removed.

42 actual Python refusal stubs (exit1/RETIRED, no stdout), three shell guards and current generated client bindings checked. The existing retirement test now rejects a body after refusal. That actual test failed when reading original source from the preceding commit;38 current recipe methods PASS. No historical SSH/deploy bodies were executed. INI/parser/runtime/native/benchmark inputs are unchanged; API/internal-model/test/evidence JSON remains permitted. Full feature-aware dead-code review across all modules remains open.

**Open debt:** `native-libs/provenance.py --check` detects changed native inputs (recorded e3fb04a8, current 7d42e38f). Real independent A/B builds of current desktop/Android recipes, binary inventories and subsequent provenance update are required. The stock CLI used for Q35/Q36 does not refresh client libraries. Lab compilation waits for measurement completion to preserve run conditions.

Release certification reports 21 problems: its overall digest and all 20 automated Linux cases belong to another tree. The gate hashes the entire committed tree except its own evidence; new documentation/report commits also change that digest. Freeze final release inputs first, then execute and bind the matrix to that tree. Relabeling historical PASS under a new digest is prohibited. User-skipped physical platforms are not reported as tested.

Full Q35 endurance and the Q36 current data-plane matrix run separately; this report does not label their intermediate states PASS. Remaining full Q37 work includes key/default/example parity, OS/features/FFI-aware dead-code review and integration. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.
