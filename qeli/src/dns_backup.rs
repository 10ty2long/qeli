//! Recovery of snapshots written by older Linux clients. New sessions use per-link DNS.
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Snapshot of `/etc/resolv.conf` before qeli touched it.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct DnsBackup {
    /// "symlink" | "file" | "absent" | "managed-no-original"
    pub(crate) kind: String,
    /// Link target for `kind == "symlink"`.
    pub(crate) target: Option<String>,
    /// File content for `kind == "file"`.
    pub(crate) content: Option<String>,
    /// Unix permission bits for `kind == "file"`.
    pub(crate) mode: Option<u32>,
}

/// Only retire the recovery record after every requested filesystem operation succeeds.
pub(crate) fn restore_and_remove(resolv: &Path, backup: &Path) -> anyhow::Result<()> {
    restore_resolv(resolv, backup).map_err(|error| {
        anyhow::anyhow!(
            "failed to restore {}: {error} (backup kept at {})",
            resolv.display(),
            backup.display()
        )
    })?;
    remove_if_present(backup).map_err(|error| {
        anyhow::anyhow!(
            "restored {}, but could not remove backup {}: {error}",
            resolv.display(),
            backup.display()
        )
    })
}

fn remove_if_present(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Rebuild the resolver from a validated snapshot, leaving retirement to the caller.
pub(crate) fn restore_resolv(resolv: &Path, backup: &Path) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(backup)?;
    let snap: DnsBackup = serde_json::from_str(&json)?;
    match snap.kind.as_str() {
        "symlink" => {
            let target = snap
                .target
                .filter(|target| !target.is_empty() && !target.contains('\0'))
                .ok_or_else(|| anyhow::anyhow!("symlink backup without a valid target"))?;
            #[cfg(unix)]
            {
                remove_if_present(resolv)?;
                std::os::unix::fs::symlink(&target, resolv)
                    .map_err(|error| anyhow::anyhow!("recreate symlink -> {target}: {error}"))?;
                Ok(())
            }
            #[cfg(not(unix))]
            {
                let _ = target;
                anyhow::bail!("Unix DNS symlink recovery is unsupported on this platform")
            }
        }
        "file" => {
            // An explicitly empty original is valid; a missing payload is not.
            let content = snap
                .content
                .ok_or_else(|| anyhow::anyhow!("file backup without content"))?;
            crate::util::write_atomic(resolv, content.as_bytes())?;
            #[cfg(unix)]
            if let Some(mode) = snap.mode {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(resolv, std::fs::Permissions::from_mode(mode))
                    .map_err(|error| anyhow::anyhow!("restore resolver permissions: {error}"))?;
            }
            Ok(())
        }
        "absent" => {
            // Check the unlink result itself: exists() follows dangling symlinks and can
            // hide inspection errors. A failed removal must keep the recovery record.
            remove_if_present(resolv)?;
            Ok(())
        }
        "managed-no-original" => {
            // Preserve the legacy recovery fallback; new sessions never create this record.
            let content =
                "# Restored by qeli (original unknown)\nnameserver 1.1.1.1\nnameserver 8.8.8.8\n";
            crate::util::write_atomic(resolv, content.as_bytes())
        }
        other => anyhow::bail!("unknown backup kind: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture {
        dir: PathBuf,
        resolv: PathBuf,
        backup: PathBuf,
    }
    impl Fixture {
        fn new(snapshot: serde_json::Value) -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "qeli-dns-backup-{}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&dir).unwrap();
            let resolv = dir.join("resolv.conf");
            let backup = dir.join("backup.json");
            std::fs::write(&backup, serde_json::to_vec(&snapshot).unwrap()).unwrap();
            Self {
                dir,
                resolv,
                backup,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn file_and_explicit_empty_content_restore_before_backup_retirement() {
        for original in ["nameserver 192.0.2.53\n", ""] {
            let f = Fixture::new(serde_json::json!({"kind":"file", "content":original}));
            std::fs::write(&f.resolv, "nameserver 10.0.0.1\n").unwrap();
            restore_and_remove(&f.resolv, &f.backup).unwrap();
            assert_eq!(std::fs::read_to_string(&f.resolv).unwrap(), original);
            assert!(!f.backup.exists());
        }
    }

    #[test]
    fn absent_snapshot_removes_file_and_accepts_an_already_absent_path() {
        for present in [true, false] {
            let f = Fixture::new(serde_json::json!({"kind":"absent"}));
            if present {
                std::fs::write(&f.resolv, "nameserver 10.0.0.1\n").unwrap();
            }
            restore_and_remove(&f.resolv, &f.backup).unwrap();
            assert!(!f.resolv.exists());
            assert!(!f.backup.exists());
        }
    }

    #[test]
    fn failed_resolver_removal_keeps_exact_backup_and_reports_failure() {
        let f = Fixture::new(serde_json::json!({"kind":"absent"}));
        // A directory reliably rejects remove_file on Windows and Unix, even as root.
        std::fs::create_dir(&f.resolv).unwrap();
        std::fs::write(f.resolv.join("owner-state"), b"untouched").unwrap();
        let before = std::fs::read(&f.backup).unwrap();
        let error = restore_and_remove(&f.resolv, &f.backup).unwrap_err();
        assert!(error.to_string().contains("backup kept"));
        assert_eq!(std::fs::read(&f.backup).unwrap(), before);
        assert_eq!(
            std::fs::read(f.resolv.join("owner-state")).unwrap(),
            b"untouched"
        );
    }

    #[test]
    fn invalid_snapshot_payload_does_not_modify_resolver_or_retire_backup() {
        for snapshot in [
            serde_json::json!({"kind":"file"}),
            serde_json::json!({"kind":"file", "content":null}),
            serde_json::json!({"kind":"symlink"}),
            serde_json::json!({"kind":"symlink", "target":""}),
            serde_json::json!({"kind":"symlink", "target":"bad\u{0}target"}),
            serde_json::json!({"kind":"unknown"}),
        ] {
            let f = Fixture::new(snapshot);
            std::fs::write(&f.resolv, b"original resolver").unwrap();
            let before = std::fs::read(&f.backup).unwrap();
            assert!(restore_and_remove(&f.resolv, &f.backup).is_err());
            assert_eq!(std::fs::read(&f.resolv).unwrap(), b"original resolver");
            assert_eq!(std::fs::read(&f.backup).unwrap(), before);
        }
    }

    #[test]
    fn failed_file_replacement_keeps_backup() {
        let f =
            Fixture::new(serde_json::json!({"kind":"file", "content":"nameserver 192.0.2.53\n"}));
        std::fs::create_dir(&f.resolv).unwrap();
        std::fs::write(f.resolv.join("owner-state"), b"untouched").unwrap();
        let before = std::fs::read(&f.backup).unwrap();
        assert!(restore_and_remove(&f.resolv, &f.backup).is_err());
        assert_eq!(std::fs::read(&f.backup).unwrap(), before);
        assert_eq!(
            std::fs::read(f.resolv.join("owner-state")).unwrap(),
            b"untouched"
        );
    }

    #[test]
    fn malformed_backup_remains_available_for_manual_recovery() {
        let f = Fixture::new(serde_json::json!({}));
        std::fs::write(&f.backup, b"{truncated").unwrap();
        std::fs::write(&f.resolv, b"original resolver").unwrap();
        assert!(restore_and_remove(&f.resolv, &f.backup).is_err());
        assert_eq!(std::fs::read(&f.backup).unwrap(), b"{truncated");
        assert_eq!(std::fs::read(&f.resolv).unwrap(), b"original resolver");
    }

    #[test]
    fn legacy_unknown_original_fallback_remains_compatible() {
        let f = Fixture::new(serde_json::json!({"kind":"managed-no-original"}));
        restore_and_remove(&f.resolv, &f.backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(&f.resolv).unwrap(),
            "# Restored by qeli (original unknown)\nnameserver 1.1.1.1\nnameserver 8.8.8.8\n"
        );
        assert!(!f.backup.exists());
    }

    #[cfg(not(unix))]
    #[test]
    fn unsupported_symlink_restore_leaves_both_files_untouched() {
        let f = Fixture::new(serde_json::json!({"kind":"symlink", "target":"stub-resolv.conf"}));
        std::fs::write(&f.resolv, b"original resolver").unwrap();
        assert!(restore_and_remove(&f.resolv, &f.backup).is_err());
        assert_eq!(std::fs::read(&f.resolv).unwrap(), b"original resolver");
        assert!(f.backup.exists());
    }

    #[cfg(unix)]
    #[test]
    fn absent_snapshot_removes_a_dangling_resolver_symlink() {
        let f = Fixture::new(serde_json::json!({"kind":"absent"}));
        std::os::unix::fs::symlink("missing-target", &f.resolv).unwrap();
        assert!(!f.resolv.exists());
        restore_and_remove(&f.resolv, &f.backup).unwrap();
        assert!(std::fs::symlink_metadata(&f.resolv).is_err());
        assert!(!f.backup.exists());
    }

    #[cfg(unix)]
    #[test]
    fn relative_symlink_target_is_restored() {
        let f = Fixture::new(serde_json::json!({"kind":"symlink", "target":"stub-resolv.conf"}));
        std::fs::write(&f.resolv, b"tunnel resolver").unwrap();
        restore_and_remove(&f.resolv, &f.backup).unwrap();
        assert_eq!(
            std::fs::read_link(&f.resolv).unwrap(),
            Path::new("stub-resolv.conf")
        );
        assert!(!f.backup.exists());
    }

    #[cfg(unix)]
    #[test]
    fn original_file_mode_is_restored_before_backup_retirement() {
        use std::os::unix::fs::PermissionsExt;
        let f =
            Fixture::new(serde_json::json!({"kind":"file", "content":"original", "mode":0o640}));
        std::fs::write(&f.resolv, b"tunnel resolver").unwrap();
        std::fs::set_permissions(&f.resolv, std::fs::Permissions::from_mode(0o600)).unwrap();
        restore_and_remove(&f.resolv, &f.backup).unwrap();
        assert_eq!(
            std::fs::metadata(&f.resolv).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert!(!f.backup.exists());
    }
}
