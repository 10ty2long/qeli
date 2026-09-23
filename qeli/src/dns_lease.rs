//! Generation-owned per-link DNS markers. No saved name grants recovery authority.
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{self, Read},
    path::{Path, PathBuf},
};

const MAX_MARKER: u64 = 2048;
const PREFIX: &str = "dns-link-v1-";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub boot: String,
    pub device: u64,
    pub inode: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Link {
    pub scope: Scope,
    pub index: u32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8,
    token: String,
    link: Link,
}

impl Link {
    fn validate(&self) -> anyhow::Result<()> {
        let boot = self.scope.boot.as_bytes();
        let valid_boot = boot.len() == 36
            && boot.iter().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    *b == b'-'
                } else {
                    b.is_ascii_digit() || (b'a'..=b'f').contains(b)
                }
            });
        if !valid_boot
            || self.scope.inode == 0
            || self.index == 0
            || self.index > i32::MAX as u32
            || self.name.is_empty()
            || [".", ".."].contains(&self.name.as_str())
            || self.name.len() > 15
            || self
                .name
                .bytes()
                .any(|b| b == 0 || b == b'/' || b == b':' || b.is_ascii_whitespace())
        {
            anyhow::bail!("invalid DNS link identity");
        }
        Ok(())
    }
    fn path(&self, dir: &Path) -> PathBuf {
        dir.join(format!(
            "{PREFIX}{}-{}-{}-{}.state",
            self.scope.boot, self.scope.device, self.scope.inode, self.index
        ))
    }
}

fn options() -> OpenOptions {
    let options = OpenOptions::new();
    #[cfg(unix)]
    let options = {
        let mut options = options;
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC);
        options
    };
    options
}

fn regular(file: &File) -> anyhow::Result<()> {
    let meta = file.metadata()?;
    let valid = meta.is_file();
    #[cfg(unix)]
    let valid = {
        use std::os::unix::fs::MetadataExt;
        valid && meta.nlink() == 1
    };
    if !valid {
        anyhow::bail!("DNS ownership state must be a regular single-link file");
    }
    Ok(())
}

// The stable sidecar is never removed: unlinking a locked inode would let another owner
// lock a replacement. NONBLOCK also prevents a FIFO open from hanging before validation.
fn try_lock(path: &Path) -> anyhow::Result<Option<File>> {
    let lock = options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))?;
    regular(&lock)?;
    match lock.try_lock() {
        Ok(()) => Ok(Some(lock)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(error) => Err(anyhow::anyhow!("cannot lock DNS ownership state: {error}")),
    }
}

fn read_record(path: &Path) -> anyhow::Result<Option<Record>> {
    let file = match options().read(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    regular(&file)?;
    if file.metadata()?.len() > MAX_MARKER {
        anyhow::bail!("DNS ownership marker exceeds {MAX_MARKER} bytes");
    }
    let mut data = Vec::new();
    file.take(MAX_MARKER + 1).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_MARKER {
        anyhow::bail!("DNS ownership marker exceeds {MAX_MARKER} bytes");
    }
    let record: Record = serde_json::from_slice(&data)?;
    record.link.validate()?;
    if record.version != 1
        || record.token.len() != 32
        || !record
            .token
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || record.link.path(path.parent().unwrap_or(Path::new("."))) != path
    {
        anyhow::bail!("invalid DNS ownership marker version, token or path");
    }
    Ok(Some(record))
}

fn require_record(path: &Path, expected: &Record) -> anyhow::Result<()> {
    if read_record(path)?.as_ref() != Some(expected) {
        anyhow::bail!(
            "DNS ownership marker changed or disappeared; refusing cleanup at {}",
            path.display()
        );
    }
    Ok(())
}

fn retire(path: &Path, record: &Record) -> anyhow::Result<()> {
    require_record(path, record)?;
    std::fs::remove_file(path).map_err(|error| {
        anyhow::anyhow!(
            "cannot retire DNS ownership marker {}: {error}",
            path.display()
        )
    })
}

pub(crate) struct Lease {
    path: PathBuf,
    record: Record,
    _lock: File,
    active: bool,
}

impl Lease {
    pub(crate) fn acquire(dir: &Path, link: Link) -> anyhow::Result<Self> {
        link.validate()?;
        // Old versions used a lossy name. Refuse ambiguity rather than adopting a marker
        // which has neither namespace nor generation identity.
        let legacy_name: String = link
            .name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || ['-', '_', '.'].contains(c))
            .take(32)
            .collect();
        let legacy = dir.join(format!("dns-resolvectl-{legacy_name}"));
        match std::fs::symlink_metadata(&legacy) {
            Ok(_) => anyhow::bail!(
                "legacy DNS marker {} needs administrator recovery before DNS takeover",
                legacy.display()
            ),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let path = link.path(dir);
        let lock = try_lock(&path)?.ok_or_else(|| {
            anyhow::anyhow!("DNS link already has an active owner: {}", path.display())
        })?;
        if read_record(&path)?.is_some() {
            anyhow::bail!(
                "unrecovered DNS ownership marker {}; refusing to overwrite it",
                path.display()
            );
        }
        let record = Record {
            version: 1,
            token: format!("{:032x}", rand::random::<u128>()),
            link,
        };
        crate::util::write_atomic_private(&path, &serde_json::to_vec(&record)?)?;
        Ok(Self {
            path,
            record,
            _lock: lock,
            active: true,
        })
    }

    pub(crate) fn link(&self) -> &Link {
        &self.record.link
    }

    pub(crate) fn cleanup(
        &mut self,
        revert: impl FnOnce(&Link) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        if !self.active {
            return Ok(());
        }
        require_record(&self.path, &self.record)?;
        revert(&self.record.link).map_err(|error| {
            anyhow::anyhow!(
                "DNS cleanup failed: {error}; marker kept at {}",
                self.path.display()
            )
        })?;
        retire(&self.path, &self.record)?;
        self.active = false;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Recovery {
    Busy,
    Foreign,
    Live,
    Retired,
    Absent,
}

pub(crate) fn is_marker(name: &str) -> bool {
    name.starts_with(PREFIX) && name.ends_with(".state")
}

/// Startup has no original TUN descriptor. It may retire a marker for an absent index,
/// but never invokes resolvectl on a live link based only on a saved index/name.
pub(crate) fn recover(
    path: &Path,
    scope: &Scope,
    exists: impl FnOnce(u32) -> anyhow::Result<bool>,
) -> anyhow::Result<Recovery> {
    let Some(_lock) = try_lock(path)? else {
        return Ok(Recovery::Busy);
    };
    let Some(record) = read_record(path)? else {
        return Ok(Recovery::Absent);
    };
    if &record.link.scope != scope {
        return Ok(Recovery::Foreign);
    }
    if exists(record.link.index)? {
        return Ok(Recovery::Live);
    }
    retire(path, &record)?;
    Ok(Recovery::Retired)
}

/// A current descriptor observation grants authority only in the captured namespace.
/// A removed original needs no per-link revert: do not target any replacement at that index.
pub(crate) fn command_target(
    link: &Link,
    scope: &Scope,
    attached: Option<u32>,
) -> anyhow::Result<Option<u32>> {
    if &link.scope != scope {
        anyhow::bail!("DNS cleanup namespace changed; refusing resolver mutation");
    }
    match attached {
        None => Ok(None),
        Some(index) if index == link.index => Ok(Some(index)),
        Some(_) => anyhow::bail!("DNS link identity changed; refusing resolver mutation"),
    }
}

#[cfg(test)]
#[path = "dns_lease/tests.rs"]
mod tests;
