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
        let remote = &args[2];
        if verb == "show" {
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
            self.routes.remove(remote);
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
        note_created_owned(carrier_route_undo(target));
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
                note_created_owned(carrier_route_undo(route.remote));
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
        note_created_owned(carrier_route_undo(route.remote));
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
    note_created_owned(carrier_route_undo(first));
    note_created_owned(carrier_route_undo(second));
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
