//! Live namespace evidence. Holding the descriptor prevents namespace identity reuse
//! while this route owner (or its unresolved orphan reservation) exists.
use std::fs::File;
use std::os::unix::fs::MetadataExt;

#[derive(Debug)]
pub(crate) struct Namespace(File);
impl Namespace {
    pub(crate) fn capture() -> anyhow::Result<Self> {
        Ok(Self(File::open("/proc/thread-self/ns/net")?))
    }
    pub(crate) fn verify(&self) -> anyhow::Result<()> {
        let expected = self.0.metadata()?;
        let actual = std::fs::metadata("/proc/thread-self/ns/net")?;
        if (expected.dev(), expected.ino()) != (actual.dev(), actual.ino()) {
            anyhow::bail!("network namespace changed; refusing network commands");
        }
        Ok(())
    }
}
