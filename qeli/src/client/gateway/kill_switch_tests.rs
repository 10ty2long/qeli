//! Public kill-switch admission must preserve an already protected profile.
use super::*;
#[cfg(all(target_os = "linux", feature = "client"))]
use crate::client::killswitch as ks;
#[cfg(all(test, not(all(target_os = "linux", feature = "client"))))]
use crate::client_killswitch as ks;

fn protect(tun: &str, addr: &str) -> anyhow::Result<()> {
    ks::engage(addr, 443, tun, false, false, true)
}
fn seed(kernel: &mut Kernel, ipv6: bool, name: &str) {
    kernel.rules.insert(
        (ipv6, "filter".into(), name.into()),
        vec![vec!["-j".into(), "DROP".into()]],
    );
    kernel.rules.insert(
        (ipv6, "filter".into(), "OUTPUT".into()),
        vec![vec!["-j".into(), name.into()]],
    );
}
fn allow_egress(kernel: &Kernel, chain: &str, out: &str, dest: &str, depth: usize) -> bool {
    assert!(depth < 8);
    for rule in kernel
        .rules
        .get(&(false, "filter".into(), chain.into()))
        .into_iter()
        .flatten()
    {
        let value = |flag: &str| {
            rule.windows(2)
                .find(|pair| pair[0] == flag)
                .map(|pair| pair[1].as_str())
        };
        if value("-o").is_some_and(|x| x != out)
            || value("-i").is_some()
            || value("-d").is_some_and(|x| x != dest)
            || value("-p").is_some_and(|x| x != "tcp")
        {
            continue;
        }
        match value("-j") {
            Some("ACCEPT") => return true,
            Some("DROP") => return false,
            Some(target) if target.starts_with("QELI_KS") => {
                return allow_egress(kernel, target, out, dest, depth + 1)
            }
            _ => panic!("unsupported model verdict: {rule:?}"),
        }
    }
    false
}
#[test]
fn regression_second_kill_switch_preserves_first_profile() {
    run(|k| {
        protect("ks_a", "203.0.113.7").unwrap();
        assert!(allow_egress(
            &k.borrow(),
            "OUTPUT",
            "wan0",
            "203.0.113.7",
            0
        ));
        let before = k.borrow().mutations();
        let result = protect("ks_b", "203.0.113.8");
        assert!(
            allow_egress(&k.borrow(), "OUTPUT", "wan0", "203.0.113.7", 0),
            "new terminal DROP blocks the first carrier"
        );
        assert!(
            result.is_err(),
            "incompatible host-wide policies must not both be accepted"
        );
        assert_eq!(k.borrow().mutations(), before);
    });
}
#[test]
fn regression_ipv6_conflict_is_checked_before_any_ipv4_mutation() {
    run(|k| {
        seed(&mut k.borrow_mut(), true, "QELI_KS_other");
        let before = k.borrow().rules.clone();
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
        assert_eq!(
            k.borrow()
                .rules
                .get(&(true, "filter".into(), "QELI_KS_other".into())),
            before.get(&(true, "filter".into(), "QELI_KS_other".into()))
        );
    });
}
#[test]
fn regression_unknown_inventory_does_not_authorize_installation() {
    run(|k| {
        k.borrow_mut().inventory_failure = Some(false);
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn regression_truncated_inventory_does_not_authorize_installation() {
    run(|k| {
        k.borrow_mut().inventory_override = Some((false, b"-P OUTPUT ACCEPT\n".to_vec()));
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn regression_legacy_chain_is_not_claimed_by_an_unrelated_profile() {
    run(|k| {
        seed(&mut k.borrow_mut(), false, "QELI_KS");
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
        assert!(k
            .borrow()
            .rules
            .contains_key(&(false, "filter".into(), "QELI_KS".into())));
    });
}
#[test]
fn regression_leak_override_does_not_bypass_ownership_conflict() {
    run(|k| {
        seed(&mut k.borrow_mut(), false, "QELI_KS_other");
        assert!(ks::engage("203.0.113.7", 443, "ks_a", true, true, false).is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn own_profile_can_rebuild_its_chain() {
    run(|k| {
        protect("ks_a", "203.0.113.7").unwrap();
        protect("ks_a", "203.0.113.7").unwrap();
        assert!(allow_egress(
            &k.borrow(),
            "OUTPUT",
            "wan0",
            "203.0.113.7",
            0
        ));
        ks::disengage("ks_a").unwrap();
        assert!(!k
            .borrow()
            .rules
            .keys()
            .any(|(_, _, name)| name == "QELI_KS_ks_a"));
    });
}
#[test]
fn independent_administrator_chain_does_not_conflict() {
    run(|k| {
        k.borrow_mut()
            .rules
            .insert((false, "filter".into(), "HOST_FIREWALL".into()), Vec::new());
        protect("ks_a", "203.0.113.7").unwrap();
        ks::disengage("ks_a").unwrap();
        assert!(k
            .borrow()
            .rules
            .contains_key(&(false, "filter".into(), "HOST_FIREWALL".into())));
    });
}

#[test]
fn non_utf8_inventory_refuses_without_mutation() {
    run(|k| {
        k.borrow_mut().inventory_override = Some((false, vec![0xff]));
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn malformed_or_duplicate_inventory_refuses_without_mutation() {
    let prefix = "-P INPUT ACCEPT\n-P OUTPUT ACCEPT\n-P FORWARD DROP\n";
    for suffix in [
        "-N\n",
        "-P OUTPUT ACCEPT\n",
        "-N INPUT\n",
        "-A undeclared -j DROP\n",
    ] {
        run(|k| {
            k.borrow_mut().inventory_override =
                Some((false, format!("{prefix}{suffix}").into_bytes()));
            assert!(protect("ks_a", "203.0.113.7").is_err());
            assert_eq!(k.borrow().mutations(), 0);
        });
    }
}
#[test]
fn ipv6_inspection_error_refuses_before_any_ipv4_mutation() {
    run(|k| {
        k.borrow_mut().inventory_failure = Some(true);
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn unhooked_foreign_chain_requires_explicit_recovery() {
    run(|k| {
        k.borrow_mut()
            .rules
            .insert((false, "filter".into(), "QELI_KS_stale".into()), Vec::new());
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_eq!(k.borrow().mutations(), 0);
    });
}
#[test]
fn another_profile_can_start_after_verified_cleanup() {
    run(|k| {
        protect("ks_a", "203.0.113.7").unwrap();
        ks::disengage("ks_a").unwrap();
        protect("ks_b", "203.0.113.8").unwrap();
        assert!(allow_egress(
            &k.borrow(),
            "OUTPUT",
            "wan0",
            "203.0.113.8",
            0
        ));
        ks::disengage("ks_b").unwrap();
    });
}
#[test]
fn concurrent_public_start_admits_only_one_policy() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let kernel = std::sync::Arc::new(Mutex::new(Kernel::default()));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = [("ks_a", "203.0.113.7"), ("ks_b", "203.0.113.8")]
        .into_iter()
        .map(|(tun, ip)| {
            let kernel = kernel.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                with_commands(
                    move |cmd| Action::Reply(kernel.lock().unwrap().command(cmd)),
                    || {
                        barrier.wait();
                        protect(tun, ip).is_ok()
                    },
                )
            })
        })
        .collect();
    let successful: usize = workers
        .into_iter()
        .map(|worker| usize::from(worker.join().unwrap()))
        .sum();
    assert_eq!(successful, 1);
    let chains: Vec<_> = kernel
        .lock()
        .unwrap()
        .rules
        .keys()
        .filter(|(v6, _, name)| !*v6 && name.starts_with("QELI_KS_"))
        .cloned()
        .collect();
    assert_eq!(chains.len(), 1);
}

// Force IPv6 installation to fail after IPv4 has been armed. The observation
// is supplied at the production command boundary; no host routes are changed.
fn failed_ipv6(k: &Rc<RefCell<Kernel>>, observation: io::Result<Output>) {
    let mut k = k.borrow_mut();
    k.fail_ipv6_drop = true;
    k.ipv6_observation = Some(observation);
}
fn assert_ipv4_removed(k: &Rc<RefCell<Kernel>>) {
    assert!(!k
        .borrow()
        .rules
        .contains_key(&(false, "filter".into(), "QELI_KS_ks_a".into())));
}
#[test]
fn regression_unknown_ipv6_refuses_and_rolls_back_ipv4() {
    for kind in [
        io::ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::TimedOut,
        io::ErrorKind::InvalidData,
    ] {
        run(|k| {
            failed_ipv6(&k, Err(kind.into()));
            assert!(
                protect("ks_a", "203.0.113.7").is_err(),
                "unknown IPv6 evidence: {kind:?}"
            );
            assert_ipv4_removed(&k);
        });
    }
}
#[test]
fn regression_failed_ipv6_query_refuses_and_rolls_back_ipv4() {
    run(|k| {
        failed_ipv6(&k, Ok(output(2, "", "query failed")));
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_ipv4_removed(&k);
    });
}
#[test]
fn regression_observed_ipv6_refuses_and_rolls_back_ipv4() {
    run(|k| {
        failed_ipv6(
            &k,
            Ok(output(
                0,
                "2: eth0\n    inet6 2001:db8::1/64 scope global\n",
                "",
            )),
        );
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_ipv4_removed(&k);
    });
}
#[test]
fn regression_unrecognized_ipv6_output_requires_protection() {
    run(|k| {
        let mut reply = output(0, "", "");
        reply.stdout = vec![0xff];
        failed_ipv6(&k, Ok(reply));
        assert!(protect("ks_a", "203.0.113.7").is_err());
        assert_ipv4_removed(&k);
    });
}
#[test]
fn regression_ipv6_refusal_reports_failed_ipv4_rollback() {
    run(|k| {
        failed_ipv6(&k, Err(io::ErrorKind::PermissionDenied.into()));
        k.borrow_mut().lie_delete = true;
        let error = protect("ks_a", "203.0.113.7").unwrap_err().to_string();
        assert!(error.contains("rollback"), "{error}");
        assert!(k
            .borrow()
            .rules
            .contains_key(&(false, "filter".into(), "QELI_KS_ks_a".into())));
    });
}
#[test]
fn verified_empty_ipv6_query_allows_ipv4_only_protection() {
    run(|k| {
        failed_ipv6(&k, Ok(output(0, "", "")));
        protect("ks_a", "203.0.113.7").unwrap();
        assert!(k
            .borrow()
            .rules
            .contains_key(&(false, "filter".into(), "QELI_KS_ks_a".into())));
    });
}
#[test]
fn explicit_ipv6_leak_override_allows_unknown_evidence() {
    run(|k| {
        failed_ipv6(&k, Err(io::ErrorKind::PermissionDenied.into()));
        ks::engage("203.0.113.7", 443, "ks_a", false, true, true).unwrap();
        assert!(k
            .borrow()
            .rules
            .contains_key(&(false, "filter".into(), "QELI_KS_ks_a".into())));
    });
}
