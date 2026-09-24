//! Real child processes model rules in private files; no host firewall/sysctl writes.
use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::MutexGuard,
    time::{Duration, Instant},
};

static SERIAL: Mutex<()> = Mutex::new(());
struct Fixture {
    dir: PathBuf,
    profile: String,
    ids: Mutex<Vec<DnsInputId>>,
    _serial: MutexGuard<'static, ()>,
}
const TOOL: &str = r#"
printf '%s\n' "$*" >> "$0.calls"
[ "$1" = --wait ] && shift 2
if [ "$1" = --version ]; then
  [ -e "$0.probe-delay" ] && sleep "$(cat "$0.probe-delay")"
  printf fixture; exit 0
fi
[ "$1" = -t ] && shift 2
op=$1; chain=$2; shift 2
proto=udp
while [ "$#" -gt 0 ]; do
  if [ "$1" = -p ]; then proto=$2; shift; fi
  shift
done
case "$op" in
  -S) [ -e "$0.inventory-delay" ] && sleep "$(cat "$0.inventory-delay")"; printf '%s\n' "-P $chain ACCEPT"; exit 0;;
  -C) [ -e "$0.$chain.$proto" ] && exit 0; exit 1;;
  -D)
    [ -e "$0.deny-$proto" ] && exit 4
    rm -f "$0.$chain.$proto"
    [ -e "$0.$proto-delay" ] && sleep "$(cat "$0.$proto-delay")"
    exit 0;;
  *) exit 17;;
esac
"#;
impl Fixture {
    fn new() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let profile = format!("budget-{}", rand::random::<u64>());
        let dir = std::env::temp_dir().join(format!("qeli-nat-{}-{profile}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        for bin in ["iptables", "ip6tables"] {
            let path = dir.join(bin);
            std::fs::write(&path, format!("#!/bin/sh\n{TOOL}")).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self {
            dir,
            profile,
            ids: Mutex::new(Vec::new()),
            _serial: serial,
        }
    }
    fn run<T>(&self, run: impl FnOnce() -> T) -> T {
        cleanup_budget::with_paths(
            ["iptables", "ip6tables"]
                .map(|bin| Some(self.dir.join(bin).to_str().unwrap().to_owned())),
            run,
        )
    }
    fn flag(&self, name: &str, value: &str) {
        std::fs::write(self.dir.join(name), value).unwrap();
    }
    fn calls(&self, bin: &str) -> String {
        std::fs::read_to_string(self.dir.join(format!("{bin}.calls"))).unwrap_or_default()
    }
    fn retain(&self, ipv6: bool, proto: &str) {
        let _guard = firewall_program_lock().lock().unwrap();
        owned_rules()
            .lock()
            .unwrap()
            .retain(
                &self.profile,
                crate::nat_owned_rules::Rule {
                    ipv6,
                    table: "filter".into(),
                    chain: "FORWARD".into(),
                    args: vec![
                        "-p".into(),
                        proto.into(),
                        "-m".into(),
                        "comment".into(),
                        "--comment".into(),
                        tag(&self.profile),
                        "-j".into(),
                        "ACCEPT".into(),
                    ],
                },
            )
            .unwrap();
        self.flag(
            &format!(
                "{}.FORWARD.{}",
                if ipv6 { "ip6tables" } else { "iptables" },
                proto
            ),
            "present",
        );
    }
    fn retained(&self) -> Vec<bool> {
        let _guard = firewall_program_lock().lock().unwrap();
        let mut families = Vec::new();
        let _ = owned_rules()
            .lock()
            .unwrap()
            .cleanup(Some(&self.profile), |rule| {
                families.push(rule.ipv6);
                anyhow::bail!("inspection only")
            });
        families
    }
    fn retire_dns(&self) -> DnsInputRules {
        let _guard = firewall_program_lock().lock().unwrap();
        let owned =
            DnsInputRules::new(&self.profile, "vpn0", "192.0.2.0/24", "192.0.2.1", 53).unwrap();
        let mut registry = dns_input_registry().lock().unwrap();
        let id = registry
            .begin(owned.clone(), |_| panic!("unexpected previous owner"))
            .unwrap();
        self.ids.lock().unwrap().push(id);
        assert!(registry
            .finish(id, |_| anyhow::bail!("defer fixture cleanup"))
            .is_err());
        self.flag("iptables.INPUT.udp", "present");
        self.flag("iptables.INPUT.tcp", "present");
        owned
    }
    fn cleanup(&self) {
        self.run(|| cleanup_until(&self.profile, budget(3000)))
            .unwrap();
        assert!(self.retained().is_empty());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _guard = firewall_program_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _ = owned_rules()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .cleanup(Some(&self.profile), |_| Ok(()));
        let mut registry = dns_input_registry()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for id in self.ids.get_mut().unwrap() {
            let _ = registry.finish(*id, |_| Ok(()));
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
fn budget(ms: u64) -> Budget {
    Budget {
        until: Instant::now() + Duration::from_millis(ms),
    }
}
fn deadline(result: anyhow::Result<()>) {
    let error = result.unwrap_err();
    assert!(format!("{error:#}").contains("deadline"), "{error:#}");
}

#[test]
fn expired_and_busy_cleanup_admission_start_no_commands() {
    let f = Fixture::new();
    f.run(|| {
        deadline(cleanup_until(&f.profile, budget(0)));
        let held = firewall_program_lock().lock().unwrap();
        deadline(cleanup_until(&f.profile, budget(30)));
        deadline(cleanup_all_until(budget(30), || {
            panic!("recovery without admission")
        }));
        deadline(finish_owned_cleanup_until(budget(30), || {
            panic!("release without admission")
        }));
        drop(held);
    });
    assert!(f.calls("iptables").is_empty() && f.calls("ip6tables").is_empty());
}

#[test]
fn registry_waits_consume_cleanup_deadline_and_retain_rules() {
    let f = Fixture::new();
    f.retain(false, "udp");
    f.run(|| {
        let held = owned_rules().lock().unwrap();
        deadline(retry_owned_rules(Some(&f.profile), budget(30)));
        drop(held);
        let held = dns_input_registry().lock().unwrap();
        deadline(retry_dns_input(Some(&f.profile), budget(30)));
        drop(held);
        let held = ipv6_sysctl_leases().lock().unwrap();
        deadline(release_ipv6_sysctls_until(&f.profile, budget(30)));
        drop(held);
    });
    assert_eq!(f.retained(), [false]);
    assert!(f.calls("iptables").is_empty());
    f.cleanup();
}

#[test]
fn cleanup_queue_time_consumes_first_sweep_budget() {
    let f = Fixture::new();
    f.flag("iptables.inventory-delay", "0.45");
    let held = firewall_program_lock().lock().unwrap();
    let (sent, received) = std::sync::mpsc::channel();
    let result = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let b = budget(650);
            sent.send(()).unwrap();
            f.run(|| cleanup_until(&f.profile, b))
        });
        received.recv_timeout(Duration::from_secs(2)).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        drop(held);
        worker.join().unwrap()
    });
    deadline(result);
    assert_eq!(f.calls("iptables").lines().count(), 1);
    assert!(f.calls("ip6tables").is_empty());
}

#[test]
fn exact_families_share_deadline_and_late_deletion_retains_retry() {
    let f = Fixture::new();
    f.retain(false, "udp");
    f.retain(true, "udp");
    f.flag("iptables.udp-delay", "0.25");
    f.flag("ip6tables.udp-delay", "0.45");
    deadline(f.run(|| cleanup_until(&f.profile, budget(650))));
    assert!(f.calls("iptables").contains("-D FORWARD"));
    assert!(f.calls("ip6tables").contains("-D FORWARD"));
    assert!(
        !f.dir.join("ip6tables.FORWARD.udp").exists(),
        "second mutation did not apply"
    );
    assert_eq!(f.retained(), [true], "unverified absence must remain owned");
    assert_eq!(
        f.calls("ip6tables")
            .lines()
            .filter(|line| line.contains("-C FORWARD"))
            .count(),
        1,
        "late acknowledgement started verification"
    );
    f.cleanup();
}

#[test]
fn retired_dns_transports_share_deadline_and_retain_generation() {
    let f = Fixture::new();
    let owned = f.retire_dns();
    f.flag("iptables.udp-delay", "0.25");
    f.flag("iptables.tcp-delay", "0.45");
    deadline(f.run(|| cleanup_until(&f.profile, budget(650))));
    assert!(
        !f.dir.join("iptables.INPUT.udp").exists() && !f.dir.join("iptables.INPUT.tcp").exists()
    );
    assert!(f.calls("ip6tables").is_empty());
    let mut pending = 0;
    assert!(dns_input_registry()
        .lock()
        .unwrap()
        .begin(owned, |_| {
            pending += 1;
            anyhow::bail!("pending generation")
        })
        .is_err());
    assert_eq!(pending, 1);
    f.cleanup();
    dns_input_registry()
        .lock()
        .unwrap()
        .retry(Some(&f.profile), |_| {
            panic!("cleanup lost pending generation")
        })
        .unwrap();
}

#[test]
fn final_cleanup_shares_deadline_and_preserves_unfinished_sysctl_work() {
    let f = Fixture::new();
    f.retain(false, "udp");
    f.retire_dns();
    f.flag("iptables.udp-delay", "0.2");
    f.flag("iptables.tcp-delay", "0.4");
    let mut released = false;
    deadline(f.run(|| {
        finish_owned_cleanup_until(budget(650), || {
            released = true;
            Ok(())
        })
    }));
    assert!(
        !released,
        "expired firewall work started another sysctl release"
    );
    assert!(
        f.retained().is_empty(),
        "verified NAT absence should be retired"
    );
    f.run(|| {
        finish_owned_cleanup_until(budget(3000), || {
            released = true;
            Ok(())
        })
    })
    .unwrap();
    assert!(released);
}

#[test]
fn startup_sweeps_share_deadline_and_late_sysctl_callbacks_cannot_succeed() {
    let f = Fixture::new();
    f.flag("iptables.inventory-delay", "0.06");
    f.flag("ip6tables.inventory-delay", "0.1");
    deadline(f.run(|| cleanup_all_until(budget(600), || Ok(()))));
    assert_eq!(f.calls("iptables").lines().count(), 5);
    assert!(!f.calls("ip6tables").is_empty());
    assert!(f.calls("ip6tables").lines().count() < 5);
    let before = (f.calls("iptables"), f.calls("ip6tables"));
    let late = || {
        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    };
    deadline(f.run(|| cleanup_all_until(budget(30), late)));
    deadline(f.run(|| finish_owned_cleanup_until(budget(30), late)));
    assert_eq!(before, (f.calls("iptables"), f.calls("ip6tables")));
}

#[test]
fn fallback_probe_uses_remaining_cleanup_budget() {
    let f = Fixture::new();
    f.flag("iptables.probe-delay", "0.4");
    let path = f.dir.join("iptables");
    let error = budget(100).probe(path.to_str().unwrap()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert_eq!(f.calls("iptables").trim(), "--version");
}
