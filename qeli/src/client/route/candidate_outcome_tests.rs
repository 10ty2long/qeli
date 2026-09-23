use super::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::process::{ExitStatus, Output};
use std::sync::{Arc, Mutex, MutexGuard};

type Executor = Box<dyn FnMut(&[String]) -> io::Result<Output>>;
thread_local! {
    static EXECUTOR: RefCell<Option<Executor>> = RefCell::new(None);
}

pub(super) fn command_output(args: &[String]) -> Option<io::Result<Output>> {
    EXECUTOR.with(|executor| executor.borrow_mut().as_mut().map(|run| run(args)))
}

fn output(success: bool, text: &str) -> io::Result<Output> {
    Ok(Output {
        status: ExitStatus::from_raw(if success { 0 } else { 256 }),
        stdout: if success {
            text.as_bytes().to_vec()
        } else {
            Vec::new()
        },
        stderr: if success {
            Vec::new()
        } else {
            text.as_bytes().to_vec()
        },
    })
}

#[derive(Clone, Copy)]
enum Fault {
    Reject,
    ApplyThenFail,
    ApplyThenIo(io::ErrorKind),
    UnchangedIo,
    Unreadable,
    Ambiguous,
    Foreign,
}

struct Kernel {
    routes: BTreeMap<String, Vec<String>>,
    calls: Vec<Vec<String>>,
    fail: Option<(String, String, Fault)>,
    fired: bool,
    fail_rollback: bool,
    extra_snapshot: Option<String>,
    query_error: bool,
    post_delete_query_error: bool,
    lie_delete: Option<bool>,
    race_before_delete: Option<(String, Vec<String>)>,
    operator_change_on_failure: Option<(String, Vec<String>)>,
}
impl Kernel {
    fn run(&mut self, raw: &[String]) -> io::Result<Output> {
        self.calls.push(raw.to_vec());
        let args = if raw.first().is_some_and(|s| s == "-6") {
            &raw[1..]
        } else {
            raw
        };
        assert_eq!(args[0], "route");
        let verb = args[1].as_str();
        if verb == "flush" {
            return output(true, "");
        }
        let remote = &args[if verb == "show" && args[2] == "exact" {
            3
        } else {
            2
        }];
        if verb == "show" {
            if self.query_error {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            let current = self
                .routes
                .get(remote)
                .map(|r| r.join(" "))
                .unwrap_or_default();
            if let Some(extra) = &self.extra_snapshot {
                return output(true, &format!("{current}\n{extra}"));
            }
            if self.fired && self.fail.as_ref().is_some_and(|(_, ip, _)| ip == remote) {
                match self.fail.as_ref().unwrap().2 {
                    Fault::Unreadable => return Err(io::ErrorKind::BrokenPipe.into()),
                    Fault::Ambiguous => {
                        return output(true, &format!("{current}\n{remote} dev other0"))
                    }
                    _ => {}
                }
            }
            return output(true, &current);
        }
        if verb == "get" {
            return output(
                true,
                &self.routes.get(remote).expect("FIB route exists").join(" "),
            );
        }
        assert!(matches!(verb, "add" | "replace" | "del"));
        let fail = !self.fired
            && self
                .fail
                .as_ref()
                .is_some_and(|(action, ip, _)| action == verb && ip == remote);
        if fail {
            self.fired = true;
            if let Some((ip, replacement)) = self.operator_change_on_failure.take() {
                self.routes.insert(ip, replacement);
            }
            match self.fail.as_ref().unwrap().2 {
                Fault::Reject | Fault::Unreadable | Fault::Ambiguous => {
                    return output(false, "fixture rejected")
                }
                Fault::UnchangedIo => return Err(io::ErrorKind::BrokenPipe.into()),
                Fault::Foreign => {
                    self.routes.insert(
                        remote.clone(),
                        vec![remote.clone(), "dev".into(), "operator0".into()],
                    );
                    return output(false, "fixture concurrent operator change");
                }
                _ => {}
            }
        } else if self.fired && self.fail_rollback && verb == "del" {
            return output(false, "fixture rollback failed");
        }
        if verb == "del" {
            if let Some(success) = self.lie_delete {
                return output(
                    success,
                    if success {
                        ""
                    } else {
                        "RTNETLINK answers: No such process"
                    },
                );
            }
            if self
                .race_before_delete
                .as_ref()
                .is_some_and(|(ip, _)| ip == remote)
            {
                let (ip, replacement) = self.race_before_delete.take().unwrap();
                self.routes.insert(ip, replacement);
            }
            if let Some(current) = self.routes.get(remote) {
                // Kernel-style selectors: IPv6 does not match preferred source on delete.
                for field in ["dev", "via", "metric", "proto", "src"] {
                    if field == "src" && raw[0] == "-6" {
                        continue;
                    }
                    if let Some(wanted) = args.windows(2).find(|p| p[0] == field) {
                        if !current.windows(2).any(|p| p == wanted) {
                            return output(false, "RTNETLINK answers: No such process");
                        }
                    }
                }
            }
            self.routes.remove(remote);
            if self.post_delete_query_error {
                self.query_error = true;
            }
        } else {
            self.routes.insert(remote.clone(), args[2..].to_vec());
        }
        if fail {
            match self.fail.as_ref().unwrap().2 {
                Fault::ApplyThenIo(kind) => return Err(kind.into()),
                Fault::ApplyThenFail => return output(false, "fixture lost completion"),
                _ => unreachable!(),
            }
        }
        output(true, "")
    }
}

struct Fixture {
    kernel: Arc<Mutex<Kernel>>,
    _guard: MutexGuard<'static, ()>,
}
impl Fixture {
    fn new(routes: Vec<Vec<String>>, fail: Option<(&str, IpAddr, Fault)>) -> Self {
        let guard = ROUTE_TEST_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let _ = take_created();
        let kernel = Arc::new(Mutex::new(Kernel {
            routes: routes.into_iter().map(|r| (r[0].clone(), r)).collect(),
            calls: Vec::new(),
            fail: fail
                .map(|(action, remote, fault)| (action.to_string(), remote.to_string(), fault)),
            fired: false,
            fail_rollback: false,
            extra_snapshot: None,
            query_error: false,
            post_delete_query_error: false,
            lie_delete: None,
            race_before_delete: None,
            operator_change_on_failure: None,
        }));
        let executor = kernel.clone();
        EXECUTOR.with(|slot| {
            assert!(slot.borrow().is_none());
            *slot.borrow_mut() = Some(Box::new(move |args| executor.lock().unwrap().run(args)));
        });
        Self {
            kernel,
            _guard: guard,
        }
    }
    fn mutations(&self) -> Vec<Vec<String>> {
        self.kernel
            .lock()
            .unwrap()
            .calls
            .iter()
            .filter(|args| {
                args.iter()
                    .any(|s| matches!(s.as_str(), "add" | "replace" | "del"))
            })
            .cloned()
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        EXECUTOR.with(|slot| *slot.borrow_mut() = None);
        let _ = take_created();
    }
}

fn candidate(ipv6: bool) -> LinuxCandidateRoute {
    LinuxCandidateRoute {
        remote: if ipv6 {
            "2001:db8::20"
        } else {
            "198.51.100.20"
        }
        .parse()
        .unwrap(),
        source: if ipv6 { "2001:db8::10" } else { "192.0.2.10" }
            .parse()
            .unwrap(),
        gateway: Some(if ipv6 { "2001:db8::1" } else { "192.0.2.1" }.into()),
        interface: "eth0".into(),
    }
}
fn previous(remote: IpAddr) -> Vec<String> {
    vec![
        remote.to_string(),
        "dev".into(),
        "old0".into(),
        "metric".into(),
        "7".into(),
    ]
}
fn plan(routes: Vec<LinuxCandidateRoute>) -> LinuxPreparedPathRoutes {
    LinuxPreparedPathRoutes {
        generation: 7,
        candidate_id: 41,
        routes,
        tunnel_interface: "qtest".into(),
    }
}
fn unknown(error: &anyhow::Error) -> bool {
    error.downcast_ref::<RouteCommitStateUnknown>().is_some()
}

/// Run the real commit path. A retirement uses the opposite family's new carrier.
fn failed(action: &str, ipv6: bool, fault: Fault, expect_unknown: bool) {
    let current = candidate(ipv6);
    let target = current.remote;
    let old = previous(target);
    let mut initial = Vec::new();
    if action != "add" {
        initial.push(old.clone());
    }
    let fixture = Fixture::new(initial, Some((action, target, fault)));
    if action != "add" {
        seed_owned(target);
    }
    let prepared = plan(vec![if action == "del" {
        candidate(!ipv6)
    } else {
        current.clone()
    }]);
    let error = prepared
        .commit(if action == "del" {
            std::slice::from_ref(&target)
        } else {
            &[]
        })
        .unwrap_err();
    assert_eq!(
        unknown(&error),
        expect_unknown,
        "{action} IPv6={ipv6}: {error}"
    );
    assert!(fixture.kernel.lock().unwrap().fired);
    // An uncertain add cannot claim a prefix which a concurrent operator could have installed.
    assert_eq!(
        created_by_us_owned(&carrier_route_undo(target)),
        action != "add"
    );
    assert_eq!(
        fixture
            .mutations()
            .iter()
            .filter(|args| args.iter().any(|s| s == &target.to_string()))
            .count(),
        1,
        "do not blindly rewrite/delete the failed step"
    );
    let state = fixture.kernel.lock().unwrap();
    match fault {
        Fault::Reject | Fault::UnchangedIo | Fault::Unreadable | Fault::Ambiguous => {
            assert_eq!(
                state.routes.get(&target.to_string()),
                if action == "add" { None } else { Some(&old) }
            );
        }
        Fault::Foreign => assert_eq!(state.routes[&target.to_string()][2], "operator0"),
        Fault::ApplyThenFail | Fault::ApplyThenIo(_) => {
            if action == "del" {
                assert!(!state.routes.contains_key(&target.to_string()));
            } else {
                assert_eq!(
                    state.routes[&target.to_string()],
                    candidate_route_command("add", &current)[if ipv6 { 3 } else { 2 }..]
                );
            }
        }
    }
    if action == "del" {
        let added = candidate(!ipv6).remote;
        assert!(
            !state.routes.contains_key(&added.to_string()),
            "new family must roll back"
        );
        assert!(!created_by_us_owned(&carrier_route_undo(added)));
    }
}

#[test]
fn failed_add_that_changed_kernel_state_is_not_reversible() {
    for ipv6 in [false, true] {
        failed("add", ipv6, Fault::ApplyThenFail, true);
    }
}
#[test]
fn failed_replace_that_changed_kernel_state_is_not_reversible() {
    for ipv6 in [false, true] {
        failed("replace", ipv6, Fault::ApplyThenFail, true);
    }
}
#[test]
fn failed_retirement_that_deleted_the_route_is_not_reversible() {
    for ipv6 in [false, true] {
        failed("del", ipv6, Fault::ApplyThenFail, true);
    }
}
#[test]
fn lost_command_result_after_mutation_requires_unknown_outcome() {
    for kind in [
        io::ErrorKind::TimedOut,
        io::ErrorKind::InvalidData,
        io::ErrorKind::BrokenPipe,
    ] {
        for action in ["add", "replace", "del"] {
            for ipv6 in [false, true] {
                failed(action, ipv6, Fault::ApplyThenIo(kind), true);
            }
        }
    }
}
#[test]
fn rejected_command_with_unchanged_route_remains_reversible() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            failed(action, ipv6, Fault::Reject, false);
        }
    }
}
#[test]
fn io_error_with_confirmed_unchanged_route_remains_reversible() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            failed(action, ipv6, Fault::UnchangedIo, false);
        }
    }
}
#[test]
fn unavailable_verification_cannot_prove_reversible_rejection() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            failed(action, ipv6, Fault::Unreadable, true);
        }
    }
}
#[test]
fn multiple_routes_cannot_hide_a_changed_snapshot_behind_the_first_line() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            failed(action, ipv6, Fault::Ambiguous, true);
        }
    }
}
#[test]
fn concurrent_operator_route_is_not_claimed_or_blindly_undone() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            failed(action, ipv6, Fault::Foreign, true);
        }
    }
}
#[test]
fn earlier_success_is_rolled_back_when_later_mutation_outcome_is_unknown() {
    let first = candidate(false);
    let second = candidate(true);
    let fixture = Fixture::new(
        Vec::new(),
        Some(("add", second.remote, Fault::ApplyThenFail)),
    );
    let error = plan(vec![first.clone(), second.clone()])
        .commit(&[])
        .unwrap_err();
    assert!(unknown(&error), "{error}");
    let state = fixture.kernel.lock().unwrap();
    assert!(!state.routes.contains_key(&first.remote.to_string()));
    assert!(state.routes.contains_key(&second.remote.to_string()));
    assert!(!created_by_us_owned(&carrier_route_undo(first.remote)));
}
#[test]
fn earlier_rollback_failure_remains_unknown_even_when_current_step_is_unchanged() {
    let first = candidate(false);
    let second = candidate(true);
    let fixture = Fixture::new(Vec::new(), Some(("add", second.remote, Fault::Reject)));
    fixture.kernel.lock().unwrap().fail_rollback = true;
    let error = plan(vec![first.clone(), second.clone()])
        .commit(&[])
        .unwrap_err();
    assert!(unknown(&error), "{error}");
    assert!(created_by_us_owned(&carrier_route_undo(first.remote)));
    let state = fixture.kernel.lock().unwrap();
    assert!(state.routes.contains_key(&first.remote.to_string()));
    assert!(!state.routes.contains_key(&second.remote.to_string()));
}
#[test]
fn successful_add_replace_and_retirement_keep_existing_contract() {
    for action in ["add", "replace", "del"] {
        for ipv6 in [false, true] {
            let route = candidate(ipv6);
            let fixture = Fixture::new(
                if action == "add" {
                    Vec::new()
                } else {
                    vec![previous(route.remote)]
                },
                None,
            );
            if action != "add" {
                seed_owned(route.remote);
            }
            let desired = if action == "del" {
                candidate(!ipv6)
            } else {
                route.clone()
            };
            plan(vec![desired.clone()])
                .commit(if action == "del" {
                    std::slice::from_ref(&route.remote)
                } else {
                    &[]
                })
                .unwrap();
            assert!(created_by_us_owned(&carrier_route_undo(desired.remote)));
            assert!(fixture.mutations().contains(&candidate_route_command(
                if action == "replace" {
                    "replace"
                } else {
                    "add"
                },
                &desired
            )));
            if action == "del" {
                assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
                assert!(!fixture
                    .kernel
                    .lock()
                    .unwrap()
                    .routes
                    .contains_key(&route.remote.to_string()));
            }
        }
    }
}
#[test]
fn operator_owned_match_and_conflict_do_not_mutate_or_claim_routes() {
    for matches in [false, true] {
        for ipv6 in [false, true] {
            let route = candidate(ipv6);
            let snapshot = if matches {
                candidate_route_command("add", &route)[if ipv6 { 3 } else { 2 }..].to_vec()
            } else {
                previous(route.remote)
            };
            let fixture = Fixture::new(vec![snapshot], None);
            let result = plan(vec![route.clone()]).commit(&[]);
            assert_eq!(result.is_ok(), matches);
            assert!(fixture.mutations().is_empty());
            assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
        }
    }
}

#[test]
fn ambiguous_initial_snapshot_rejects_before_any_mutation() {
    for ipv6 in [false, true] {
        let route = candidate(ipv6);
        let fixture = Fixture::new(vec![previous(route.remote)], None);
        seed_owned(route.remote);
        fixture.kernel.lock().unwrap().extra_snapshot =
            Some(format!("{} dev operator0", route.remote));
        let error = plan(vec![route.clone()]).commit(&[]).unwrap_err();
        assert!(!unknown(&error));
        assert!(error.to_string().contains("ambiguous"));
        assert!(fixture.mutations().is_empty());
        assert_eq!(
            fixture.kernel.lock().unwrap().routes[&route.remote.to_string()],
            previous(route.remote)
        );
        assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
    }
}

#[test]
fn earlier_retirement_is_restored_when_later_retirement_loses_its_result() {
    let first = candidate(false).remote;
    let second = candidate(true).remote;
    let mut new = candidate(false);
    new.remote = "203.0.113.20".parse().unwrap();
    let fixture = Fixture::new(
        vec![previous(first), previous(second)],
        Some(("del", second, Fault::ApplyThenFail)),
    );
    seed_owned(first);
    seed_owned(second);
    let error = plan(vec![new.clone()])
        .commit(&[first, second])
        .unwrap_err();
    assert!(unknown(&error), "{error}");
    let state = fixture.kernel.lock().unwrap();
    assert_eq!(state.routes[&first.to_string()], previous(first));
    assert!(!state.routes.contains_key(&second.to_string()));
    assert!(!state.routes.contains_key(&new.remote.to_string()));
    assert!(created_by_us_owned(&carrier_route_undo(first)));
    assert!(created_by_us_owned(&carrier_route_undo(second)));
    assert!(!created_by_us_owned(&carrier_route_undo(new.remote)));
}

// Seed pre-existing ownership with selectors, just like successful production installation.
fn seed_owned(remote: IpAddr) {
    let snapshot = exact_route_tokens(remote).unwrap().unwrap();
    let mut undo = carrier_route_undo(remote);
    undo.extend(snapshot.into_iter().skip(1));
    note_created_owned(undo);
}

fn install_owned(ipv6: bool) -> (Fixture, LinuxCandidateRoute) {
    let route = candidate(ipv6);
    let fixture = Fixture::new(Vec::new(), None);
    pin_carrier_route(
        route.remote,
        &PhysicalPath {
            gateway: route.gateway.clone(),
            device: route.interface.clone(),
        },
    )
    .unwrap();
    (fixture, route)
}

#[test]
fn ownership_cleanup_removes_unchanged_route_for_both_families() {
    for ipv6 in [false, true] {
        let (fixture, route) = install_owned(ipv6);
        cleanup_routes("qtest", "", &[]).unwrap();
        assert!(!fixture
            .kernel
            .lock()
            .unwrap()
            .routes
            .contains_key(&route.remote.to_string()));
        assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
        let calls = fixture.mutations();
        let deletion = calls
            .iter()
            .find(|args| args.iter().any(|s| s == "del"))
            .unwrap();
        assert!(deletion.windows(2).any(|p| p == ["dev", "eth0"]));
        assert!(deletion
            .windows(2)
            .any(|p| p[0] == "via" && p[1] == route.gateway.as_deref().unwrap()));
    }
}

#[test]
fn ownership_cleanup_preserves_operator_replacement() {
    for ipv6 in [false, true] {
        for field in ["dev", "via"] {
            let (fixture, route) = install_owned(ipv6);
            let key = route.remote.to_string();
            let replacement = {
                let mut state = fixture.kernel.lock().unwrap();
                let current = state.routes.get_mut(&key).unwrap();
                let index = current.iter().position(|s| s == field).unwrap() + 1;
                current[index] = if field == "dev" {
                    "operator0".into()
                } else if ipv6 {
                    "2001:db8::99".into()
                } else {
                    "192.0.2.99".into()
                };
                current.clone()
            };
            cleanup_routes("qtest", "", &[]).unwrap();
            assert_eq!(fixture.kernel.lock().unwrap().routes[&key], replacement);
            assert_eq!(fixture.mutations().len(), 1, "only original add");
            assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
        }
    }
}

#[test]
fn ownership_delete_selectors_preserve_replacement_between_check_and_delete() {
    for ipv6 in [false, true] {
        let (fixture, route) = install_owned(ipv6);
        let replacement = vec![route.remote.to_string(), "dev".into(), "operator0".into()];
        fixture.kernel.lock().unwrap().race_before_delete =
            Some((route.remote.to_string(), replacement.clone()));
        cleanup_routes("qtest", "", &[]).unwrap();
        assert_eq!(
            fixture.kernel.lock().unwrap().routes[&route.remote.to_string()],
            replacement
        );
        assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
    }
}

#[test]
fn ownership_cleanup_verifies_success_and_absent_error_before_forgetting() {
    for status in [true, false] {
        let (fixture, route) = install_owned(false);
        fixture.kernel.lock().unwrap().lie_delete = Some(status);
        assert!(cleanup_routes("qtest", "", &[]).is_err());
        assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
        assert!(fixture
            .kernel
            .lock()
            .unwrap()
            .routes
            .contains_key(&route.remote.to_string()));
        fixture.kernel.lock().unwrap().lie_delete = None;
        cleanup_routes("qtest", "", &[]).unwrap();
        assert!(!fixture
            .kernel
            .lock()
            .unwrap()
            .routes
            .contains_key(&route.remote.to_string()));
        assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
    }
}

#[test]
fn ownership_cleanup_accepts_confirmed_absence_after_lost_delete_result() {
    for ipv6 in [false, true] {
        let (fixture, route) = install_owned(ipv6);
        fixture.kernel.lock().unwrap().fail = Some((
            "del".into(),
            route.remote.to_string(),
            Fault::ApplyThenIo(io::ErrorKind::BrokenPipe),
        ));
        cleanup_routes("qtest", "", &[]).unwrap();
        assert!(!fixture
            .kernel
            .lock()
            .unwrap()
            .routes
            .contains_key(&route.remote.to_string()));
        assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
    }
}

#[test]
fn ownership_cleanup_retains_unreadable_snapshot_without_deleting() {
    let (fixture, route) = install_owned(false);
    fixture.kernel.lock().unwrap().query_error = true;
    assert!(cleanup_routes("qtest", "", &[]).is_err());
    assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
    assert_eq!(fixture.mutations().len(), 1);
    fixture.kernel.lock().unwrap().query_error = false;
    cleanup_routes("qtest", "", &[]).unwrap();
}

#[test]
fn ownership_cleanup_retries_after_unreadable_delete_verification() {
    let (fixture, route) = install_owned(true);
    fixture.kernel.lock().unwrap().post_delete_query_error = true;
    assert!(cleanup_routes("qtest", "", &[]).is_err());
    assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
    assert!(!fixture
        .kernel
        .lock()
        .unwrap()
        .routes
        .contains_key(&route.remote.to_string()));
    fixture.kernel.lock().unwrap().query_error = false;
    let calls = fixture.mutations().len();
    cleanup_routes("qtest", "", &[]).unwrap();
    assert_eq!(
        fixture.mutations().len(),
        calls,
        "confirmed absence needs no repeated delete"
    );
    assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
}

#[test]
fn ownership_cleanup_retains_ambiguous_snapshot() {
    let (fixture, route) = install_owned(false);
    fixture.kernel.lock().unwrap().extra_snapshot = Some(format!("{} dev other0", route.remote));
    assert!(cleanup_routes("qtest", "", &[]).is_err());
    assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
    assert_eq!(fixture.mutations().len(), 1);
}

#[test]
fn ownership_roaming_replaces_the_cleanup_selector_with_the_new_path() {
    let (fixture, mut route) = install_owned(false);
    route.interface = "wwan0".into();
    route.gateway = Some("192.0.2.99".into());
    plan(vec![route.clone()]).commit(&[]).unwrap();
    cleanup_routes("qtest", "", &[]).unwrap();
    assert!(!fixture
        .kernel
        .lock()
        .unwrap()
        .routes
        .contains_key(&route.remote.to_string()));
    let calls = fixture.mutations();
    let last = calls.last().unwrap();
    assert!(last.windows(2).any(|p| p == ["dev", "wwan0"]));
    assert!(last.windows(2).any(|p| p == ["via", "192.0.2.99"]));
}

#[test]
fn ownership_stale_record_does_not_authorize_replacing_operator_route() {
    let (fixture, route) = install_owned(false);
    let replacement = vec![route.remote.to_string(), "dev".into(), "operator0".into()];
    fixture
        .kernel
        .lock()
        .unwrap()
        .routes
        .insert(route.remote.to_string(), replacement.clone());
    let result = plan(vec![route.clone()]).commit(&[]);
    assert!(result.is_err());
    assert_eq!(
        fixture.kernel.lock().unwrap().routes[&route.remote.to_string()],
        replacement
    );
    assert_eq!(fixture.mutations().len(), 1);
    assert!(!created_by_us_owned(&carrier_route_undo(route.remote)));
}

#[test]
fn ownership_stale_record_does_not_authorize_retiring_operator_route() {
    let (fixture, old) = install_owned(false);
    let replacement = vec![old.remote.to_string(), "dev".into(), "operator0".into()];
    fixture
        .kernel
        .lock()
        .unwrap()
        .routes
        .insert(old.remote.to_string(), replacement.clone());
    plan(vec![candidate(true)]).commit(&[old.remote]).unwrap();
    assert_eq!(
        fixture.kernel.lock().unwrap().routes[&old.remote.to_string()],
        replacement
    );
    assert!(!created_by_us_owned(&carrier_route_undo(old.remote)));
}

#[test]
fn ownership_cleanup_handles_on_link_route_without_inventing_gateway() {
    for ipv6 in [false, true] {
        let route = candidate(ipv6);
        let fixture = Fixture::new(Vec::new(), None);
        pin_carrier_route(
            route.remote,
            &PhysicalPath {
                gateway: None,
                device: "eth0".into(),
            },
        )
        .unwrap();
        cleanup_routes("qtest", "", &[]).unwrap();
        assert!(!fixture
            .kernel
            .lock()
            .unwrap()
            .routes
            .contains_key(&route.remote.to_string()));
        assert!(!fixture
            .mutations()
            .last()
            .unwrap()
            .iter()
            .any(|s| s == "via"));
    }
}

#[test]
fn ownership_rollback_does_not_overwrite_operator_replacement() {
    let (fixture, mut first) = install_owned(false);
    let second = candidate(true);
    let replacement = vec![first.remote.to_string(), "dev".into(), "operator0".into()];
    {
        let mut state = fixture.kernel.lock().unwrap();
        state.fail = Some(("add".into(), second.remote.to_string(), Fault::Reject));
        state.operator_change_on_failure = Some((first.remote.to_string(), replacement.clone()));
    }
    first.interface = "wwan0".into();
    let error = plan(vec![first.clone(), second]).commit(&[]).unwrap_err();
    assert!(fixture.kernel.lock().unwrap().fired);
    assert_eq!(
        fixture.kernel.lock().unwrap().routes[&first.remote.to_string()],
        replacement
    );
    assert!(unknown(&error));
    assert!(!created_by_us_owned(&carrier_route_undo(first.remote)));
}

#[test]
fn ownership_retirement_restore_does_not_overwrite_operator_replacement() {
    let (fixture, first) = install_owned(false);
    let second = candidate(true);
    pin_carrier_route(
        second.remote,
        &PhysicalPath {
            gateway: second.gateway.clone(),
            device: second.interface.clone(),
        },
    )
    .unwrap();
    let mut next = candidate(false);
    next.remote = "203.0.113.20".parse().unwrap();
    let replacement = vec![first.remote.to_string(), "dev".into(), "operator0".into()];
    {
        let mut state = fixture.kernel.lock().unwrap();
        state.fail = Some(("del".into(), second.remote.to_string(), Fault::Reject));
        state.operator_change_on_failure = Some((first.remote.to_string(), replacement.clone()));
    }
    let error = plan(vec![next])
        .commit(&[first.remote, second.remote])
        .unwrap_err();
    assert!(fixture.kernel.lock().unwrap().fired);
    assert_eq!(
        fixture.kernel.lock().unwrap().routes[&first.remote.to_string()],
        replacement
    );
    assert!(unknown(&error));
}

#[test]
fn ownership_cleanup_preserves_changed_preferred_source_of_candidate() {
    for ipv6 in [false, true] {
        let route = candidate(ipv6);
        let fixture = Fixture::new(Vec::new(), None);
        plan(vec![route.clone()]).commit(&[]).unwrap();
        let replacement = {
            let mut state = fixture.kernel.lock().unwrap();
            let current = state.routes.get_mut(&route.remote.to_string()).unwrap();
            let index = current.iter().position(|s| s == "src").unwrap() + 1;
            current[index] = if ipv6 {
                "2001:db8::77".into()
            } else {
                "192.0.2.77".into()
            };
            current.clone()
        };
        cleanup_routes("qtest", "", &[]).unwrap();
        assert_eq!(
            fixture.kernel.lock().unwrap().routes[&route.remote.to_string()],
            replacement
        );
        assert_eq!(fixture.mutations().len(), 1);
    }
}

#[test]
fn ownership_cleanup_retains_malformed_snapshot_for_retry() {
    let (fixture, route) = install_owned(false);
    let original = fixture.kernel.lock().unwrap().routes[&route.remote.to_string()].clone();
    fixture.kernel.lock().unwrap().routes.insert(
        route.remote.to_string(),
        vec!["garbled".into(), "dev".into(), "eth0".into()],
    );
    assert!(cleanup_routes("qtest", "", &[]).is_err());
    assert!(created_by_us_owned(&carrier_route_undo(route.remote)));
    assert_eq!(fixture.mutations().len(), 1);
    fixture
        .kernel
        .lock()
        .unwrap()
        .routes
        .insert(route.remote.to_string(), original);
    cleanup_routes("qtest", "", &[]).unwrap();
}
