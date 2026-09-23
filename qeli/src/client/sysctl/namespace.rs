//! A journal group belongs to one network namespace and one PID coordinate system.
use super::host;

pub(super) struct Context {
    pub network: String,
    pub pid: String,
    pub time: Option<String>,
}

pub(super) fn valid_identity(value: &str) -> bool {
    let Some((device, inode)) = value.split_once(':') else {
        return false;
    };
    match (device.parse::<u64>(), inode.parse::<u64>()) {
        (Ok(device), Ok(inode)) => inode != 0 && value == format!("{device}:{inode}"),
        _ => false,
    }
}

fn require_local_procfs(status: &str, process: u32) -> anyhow::Result<()> {
    // NStgid is ordered from the PID namespace of this procfs mount inward.
    // A single value equal to getpid proves that numeric /proc/<pid> lookups and
    // kill(pid, 0) use the same coordinate system. An inherited ancestor mount
    // can otherwise inspect or signal-probe an unrelated process with the same PID.
    let mut fields = status
        .lines()
        .filter_map(|line| line.strip_prefix("NStgid:"));
    let values: Vec<&str> = fields
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    if fields.next().is_some()
        || values.len() != 1
        || values[0].parse::<u32>().ok() != Some(process)
        || process == 0
    {
        anyhow::bail!("host sysctl procfs PID namespace mismatch or unavailable NStgid; mount procfs for the current PID namespace");
    }
    Ok(())
}

pub(super) fn current() -> anyhow::Result<Context> {
    let status = host::read("/proc/self/status").map_err(|error| {
        anyhow::anyhow!("cannot verify host sysctl procfs PID namespace: {error}")
    })?;
    require_local_procfs(&status, std::process::id())?;
    // setns affects the calling thread, so /proc/self/ns/net is insufficient.
    let network = host::namespace_identity("/proc/thread-self/ns/net")?;
    let pid = host::namespace_identity("/proc/thread-self/ns/pid")?;
    // Linux with no CONFIG_TIME_NS does not expose ns/time. Any other error is
    // uncertainty. A present/absent or changed identity never matches a saved group.
    let time = match host::namespace_identity("/proc/thread-self/ns/time") {
        Ok(identity) => Some(identity),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if !valid_identity(&network)
        || !valid_identity(&pid)
        || time
            .as_ref()
            .is_some_and(|identity| !valid_identity(identity))
    {
        anyhow::bail!("invalid current host sysctl namespace identity");
    }
    Ok(Context { network, pid, time })
}
