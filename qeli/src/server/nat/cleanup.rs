//! Verified cleanup of tagged firewall rules. Platform commands are injected so the
//! actual algorithm can be exercised without changing a host firewall.
use std::process::Output;

const CHAINS: [(&str, &str); 5] = [
    ("nat", "POSTROUTING"),
    ("nat", "PREROUTING"),
    ("filter", "INPUT"),
    ("filter", "FORWARD"),
    ("mangle", "FORWARD"),
];

#[derive(Default)]
struct Errors {
    messages: Vec<String>,
    omitted: usize,
}

impl Errors {
    fn record(&mut self, context: &str, result: anyhow::Result<()>) {
        if let Err(error) = result {
            if self.messages.len() < 8 {
                let detail: String = error.to_string().chars().take(2048).collect();
                self.messages.push(format!("{context}: {detail}"));
            } else {
                self.omitted += 1;
            }
        }
    }

    fn finish(self) -> anyhow::Result<()> {
        if self.messages.is_empty() {
            return Ok(());
        }
        let mut message = self.messages.join("; ");
        if self.omitted != 0 {
            message.push_str(&format!("; {} additional cleanup errors", self.omitted));
        }
        anyhow::bail!("firewall cleanup failed: {message}")
    }
}

/// Parse the shell-quoted shape emitted by `iptables -S` without invoking a shell. Comments
/// may contain whitespace or quotes because profile names are user-visible strings.
fn split_iptables_args(line: &str) -> Option<Vec<String>> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            started = true;
            continue;
        }
        match quote {
            Some(delimiter) if character == delimiter => {
                quote = None;
                started = true;
            }
            Some(_) if character == '\\' => escaped = true,
            Some(_) => {
                current.push(character);
                started = true;
            }
            None if character.is_whitespace() => {
                if started {
                    output.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            None if character == '"' || character == '\'' => {
                quote = Some(character);
                started = true;
            }
            None if character == '\\' => {
                escaped = true;
                started = true;
            }
            None => {
                current.push(character);
                started = true;
            }
        }
    }
    if escaped || quote.is_some() {
        return None;
    }
    if started {
        output.push(current);
    }
    Some(output)
}

/// The iptables comment on a rule (the token right after `--comment`, dequoted). `None`
/// when the rule carries no comment or malformed quoting.
pub(crate) fn rule_comment(line: &str) -> Option<String> {
    let toks = split_iptables_args(line)?;
    toks.windows(2)
        .find(|w| w[0] == "--comment")
        .map(|w| w[1].clone())
}

fn checked_output(
    args: &[&str],
    run: &mut impl FnMut(&[&str]) -> std::io::Result<Output>,
) -> anyhow::Result<Output> {
    let out = run(args).map_err(|error| anyhow::anyhow!("{}: {error}", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!(
            "{} failed with {}: {}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out)
}

fn owned_rules(
    output: &[u8],
    chain: &str,
    needle: &str,
    exact: bool,
) -> anyhow::Result<Vec<Vec<String>>> {
    let text = std::str::from_utf8(output)
        .map_err(|error| anyhow::anyhow!("invalid firewall listing UTF-8: {error}"))?;
    let mut rules = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let tokens = split_iptables_args(line)
            .ok_or_else(|| anyhow::anyhow!("malformed firewall listing quoting"))?;
        match tokens.first().map(String::as_str) {
            Some("-P" | "-N") if tokens.get(1).is_some_and(|name| name == chain) => {}
            Some("-A") if tokens.get(1).is_some_and(|name| name == chain) => {
                if tokens.windows(2).any(|pair| {
                    pair[0] == "--comment"
                        && if exact {
                            pair[1] == needle
                        } else {
                            pair[1].starts_with(needle)
                        }
                }) {
                    rules.push(tokens.into_iter().skip(2).collect());
                }
            }
            _ => anyhow::bail!("unexpected firewall listing for chain {chain}"),
        }
    }
    Ok(rules)
}

pub(crate) fn cleanup_matching_with(
    needle: &str,
    exact: bool,
    mut run: impl FnMut(&[&str]) -> std::io::Result<Output>,
) -> anyhow::Result<()> {
    let mut errors = Errors::default();
    for (table, chain) in CHAINS {
        let result = (|| -> anyhow::Result<()> {
            let output = checked_output(&["-t", table, "-S", chain], &mut run)?;
            let rules = owned_rules(&output.stdout, chain, needle, exact)?;
            if rules.is_empty() {
                return Ok(());
            }
            // One deletion per occurrence in this snapshot. A successful no-op cannot
            // make us loop forever; verification also detects additions during cleanup.
            let mut deletes = Errors::default();
            for rule in rules {
                let mut args = vec!["-t", table, "-D", chain];
                args.extend(rule.iter().map(String::as_str));
                deletes.record("delete", checked_output(&args, &mut run).map(|_| ()));
            }
            let verify = (|| -> anyhow::Result<()> {
                let output = checked_output(&["-t", table, "-S", chain], &mut run)?;
                let left = owned_rules(&output.stdout, chain, needle, exact)?.len();
                if left != 0 {
                    anyhow::bail!("{left} owned rules remain after deletion");
                }
                Ok(())
            })();
            deletes.record("verify", verify);
            deletes.finish()
        })();
        errors.record(&format!("{table}/{chain}"), result);
    }
    errors.finish()
}

#[cfg(test)]
#[path = "cleanup/tests.rs"]
mod tests;
