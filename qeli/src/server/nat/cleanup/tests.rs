use super::*;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

fn output(success: bool, stdout: impl Into<Vec<u8>>, stderr: &str) -> Output {
    Output {
        status: std::process::ExitStatus::from_raw(if success { 0 } else { 256 }),
        stdout: stdout.into(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

#[derive(Clone)]
struct Rule {
    table: &'static str,
    chain: &'static str,
    spec: Vec<String>,
}

impl Rule {
    fn new(table: &'static str, chain: &'static str, comment: &str) -> Self {
        Self {
            table,
            chain,
            spec: ["-m", "comment", "--comment", comment, "-j", "ACCEPT"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }
    fn line(&self) -> String {
        let spec = self
            .spec
            .iter()
            .map(|arg| format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ");
        format!("-A {} {spec}\n", self.chain)
    }
}

#[derive(Default)]
struct Firewall {
    rules: Vec<Rule>,
    calls: Vec<Vec<String>>,
    no_op: bool,
}

impl Firewall {
    fn run(&mut self, args: &[&str]) -> std::io::Result<Output> {
        // A broken algorithm fails the fixture instead of hanging the test process.
        assert!(
            self.calls.len() < 4096,
            "cleanup exceeded fixture command budget"
        );
        self.calls
            .push(args.iter().map(|arg| arg.to_string()).collect());
        match args[2] {
            "-S" => {
                let mut listing = format!("-P {} ACCEPT\n", args[3]);
                for rule in self
                    .rules
                    .iter()
                    .filter(|r| r.table == args[1] && r.chain == args[3])
                {
                    listing.push_str(&rule.line());
                }
                Ok(output(true, listing, ""))
            }
            "-D" => {
                let position = self
                    .rules
                    .iter()
                    .position(|r| r.table == args[1] && r.chain == args[3] && r.spec == args[4..])
                    .expect("only rules from the listing may be deleted");
                if !self.no_op {
                    self.rules.remove(position);
                }
                Ok(output(true, "", ""))
            }
            _ => panic!("cleanup must never install or flush rules"),
        }
    }
    fn cleanup(&mut self, needle: &str, exact: bool) -> anyhow::Result<()> {
        cleanup_matching_with(needle, exact, |args| self.run(args))
    }
}

#[test]
fn empty_cleanup_checks_every_chain_once() {
    let mut firewall = Firewall::default();
    firewall.cleanup("qeli-nat:edge", true).unwrap();
    assert_eq!(firewall.calls.len(), 5);
    assert!(firewall.calls.iter().all(|call| call[2] == "-S"));
}

#[test]
fn exact_cleanup_preserves_other_profiles_and_admin_rules_in_every_chain() {
    let mut firewall = Firewall::default();
    for (table, chain) in CHAINS {
        for tag in ["qeli-nat:edge", "qeli-nat:edge2", "admin qeli-nat:edge"] {
            firewall.rules.push(Rule::new(table, chain, tag));
        }
    }
    firewall.cleanup("qeli-nat:edge", true).unwrap();
    assert_eq!(firewall.rules.len(), 10);
    assert!(firewall.rules.iter().all(|r| r.spec[3] != "qeli-nat:edge"));
    let calls = firewall.calls.len();
    firewall.cleanup("qeli-nat:edge", true).unwrap();
    assert_eq!(
        firewall.calls.len() - calls,
        5,
        "idempotent repeated cleanup"
    );
}

#[test]
fn startup_prefix_cleanup_preserves_unowned_comments() {
    let mut firewall = Firewall::default();
    for tag in [
        "qeli-nat:edge",
        "qeli-nat:edge2",
        "admin qeli-nat:edge",
        "qeli-nat",
    ] {
        firewall.rules.push(Rule::new("filter", "INPUT", tag));
    }
    firewall.cleanup("qeli-nat:", false).unwrap();
    assert_eq!(firewall.rules.len(), 2);
    assert_eq!(firewall.rules[0].spec[3], "admin qeli-nat:edge");
    assert_eq!(firewall.rules[1].spec[3], "qeli-nat");
}

#[test]
fn quoted_comment_is_replayed_as_one_argument() {
    let tag = "qeli-nat:edge 'one' \"two\" \\ exit";
    let mut firewall = Firewall::default();
    firewall.rules.push(Rule::new("filter", "FORWARD", tag));
    firewall.cleanup(tag, true).unwrap();
    assert!(firewall.rules.is_empty());
    let deletion = firewall.calls.iter().find(|call| call[2] == "-D").unwrap();
    assert_eq!(deletion[7], tag);
}

#[test]
fn more_than_64_duplicates_are_all_removed_without_repeated_full_scans() {
    let mut firewall = Firewall {
        rules: vec![Rule::new("filter", "FORWARD", "qeli-nat:edge"); 129],
        ..Default::default()
    };
    firewall.cleanup("qeli-nat:edge", true).unwrap();
    assert!(firewall.rules.is_empty());
    assert_eq!(firewall.calls.iter().filter(|c| c[2] == "-D").count(), 129);
    assert_eq!(firewall.calls.iter().filter(|c| c[2] == "-S").count(), 6);
}

#[test]
fn successful_no_op_delete_returns_error_instead_of_looping() {
    let mut firewall = Firewall {
        no_op: true,
        ..Default::default()
    };
    firewall
        .rules
        .push(Rule::new("nat", "POSTROUTING", "qeli-nat:edge"));
    let error = firewall
        .cleanup("qeli-nat:edge", true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("nat/POSTROUTING"), "{error}");
    assert!(error.contains("1 owned rules remain"), "{error}");
    assert_eq!(firewall.calls.len(), 7);
}

#[test]
fn failed_delete_does_not_skip_other_rules_or_chains() {
    let mut firewall = Firewall::default();
    firewall
        .rules
        .push(Rule::new("nat", "POSTROUTING", "qeli-nat:broken"));
    firewall
        .rules
        .push(Rule::new("nat", "POSTROUTING", "qeli-nat:good"));
    firewall
        .rules
        .push(Rule::new("mangle", "FORWARD", "qeli-nat:good"));
    let error = cleanup_matching_with("qeli-nat:", false, |args| {
        if args[2] == "-D" && args.contains(&"qeli-nat:broken") {
            return Ok(output(false, "", "fixture permission failure"));
        }
        firewall.run(args)
    })
    .unwrap_err()
    .to_string();
    assert!(error.contains("fixture permission failure"), "{error}");
    assert!(error.contains("1 owned rules remain"), "{error}");
    assert_eq!(firewall.rules.len(), 1);
    // A later retry retains the rule's exact ownership and can succeed.
    firewall.cleanup("qeli-nat:", false).unwrap();
    assert!(firewall.rules.is_empty());
}

#[test]
fn failed_listing_is_reported_and_later_chains_are_still_cleaned() {
    for io_failure in [false, true] {
        let mut firewall = Firewall::default();
        firewall
            .rules
            .push(Rule::new("mangle", "FORWARD", "qeli-nat:edge"));
        let error = cleanup_matching_with("qeli-nat:edge", true, |args| {
            if args[1] == "nat" && args[3] == "POSTROUTING" {
                if io_failure {
                    return Err(std::io::Error::other("fixture spawn failure"));
                }
                return Ok(output(false, "", "fixture listing failure"));
            }
            firewall.run(args)
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("nat/POSTROUTING"), "{error}");
        assert!(
            error.contains(if io_failure {
                "fixture spawn failure"
            } else {
                "fixture listing failure"
            }),
            "{error}"
        );
        assert!(firewall.rules.is_empty());
    }
}

#[test]
fn malformed_listing_never_deletes_a_partial_snapshot() {
    let good = Rule::new("nat", "POSTROUTING", "qeli-nat:edge").line();
    for tail in [
        b"-A POSTROUTING --comment \"unfinished".as_slice(),
        b"\xff",
        b"-A INPUT --comment qeli-nat:edge",
        b"unexpected output",
    ] {
        let mut calls = Vec::new();
        let error = cleanup_matching_with("qeli-nat:edge", true, |args| {
            calls.push(args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            let listing = if args[1] == "nat" && args[3] == "POSTROUTING" {
                [good.as_bytes(), tail].concat()
            } else {
                Vec::new()
            };
            Ok(output(true, listing, ""))
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("nat/POSTROUTING"), "{error}");
        assert_eq!(calls.len(), 5);
        assert!(calls.iter().all(|call| call[2] == "-S"));
    }
}

#[test]
fn failed_verification_is_not_reported_as_success() {
    let mut firewall = Firewall::default();
    firewall
        .rules
        .push(Rule::new("nat", "POSTROUTING", "qeli-nat:edge"));
    let mut reads = 0;
    let error = cleanup_matching_with("qeli-nat:edge", true, |args| {
        if args[2] == "-S" && args[3] == "POSTROUTING" {
            reads += 1;
            if reads == 2 {
                return Ok(output(false, "", "fixture verification failure"));
            }
        }
        firewall.run(args)
    })
    .unwrap_err()
    .to_string();
    assert!(error.contains("verify"), "{error}");
    assert!(error.contains("fixture verification failure"), "{error}");
    assert_eq!(firewall.calls.last().unwrap()[1], "mangle");
}

#[test]
fn concurrent_owned_rule_addition_is_reported_without_chasing_it_forever() {
    let mut firewall = Firewall::default();
    firewall
        .rules
        .push(Rule::new("nat", "POSTROUTING", "qeli-nat:edge"));
    let error = cleanup_matching_with("qeli-nat:edge", true, |args| {
        let result = firewall.run(args);
        if args[2] == "-D" {
            firewall
                .rules
                .push(Rule::new("nat", "POSTROUTING", "qeli-nat:edge"));
        }
        result
    })
    .unwrap_err()
    .to_string();
    assert!(error.contains("1 owned rules remain"), "{error}");
    assert_eq!(firewall.calls.len(), 7);
}

#[test]
fn errors_keep_multiple_chain_contexts_and_bound_repeated_diagnostics() {
    let error = cleanup_matching_with("qeli-nat:", false, |_| {
        Ok(output(false, "", "fixture unavailable"))
    })
    .unwrap_err()
    .to_string();
    for (table, chain) in CHAINS {
        assert!(error.contains(&format!("{table}/{chain}")), "{error}");
    }
    let mut errors = Errors::default();
    for _ in 0..100 {
        errors.record("delete", Err(anyhow::anyhow!("{}", "e".repeat(4096))));
    }
    let error = errors.finish().unwrap_err().to_string();
    assert!(error.len() < 17000);
    assert!(error.contains("92 additional cleanup errors"));
}

#[test]
fn comment_parser_does_not_confuse_substrings_or_broken_quotes() {
    assert_eq!(
        rule_comment("-A INPUT --comment \"qeli-nat:edge with spaces\" -j ACCEPT").as_deref(),
        Some("qeli-nat:edge with spaces")
    );
    assert_eq!(rule_comment("-A INPUT --comment \"unfinished"), None);
    assert_eq!(rule_comment("-A INPUT --commentary qeli-nat:edge"), None);
}
