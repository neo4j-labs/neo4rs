#[cfg(feature = "unstable-bolt-protocol-impl-v2")]
pub(crate) trait Bookmark {
    fn get_bookmark(&self) -> Option<&str>;
}
