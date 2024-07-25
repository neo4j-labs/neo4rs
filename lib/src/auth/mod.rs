use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ClientCertificate {
    pub(crate) cert_file: PathBuf,  // Path to the TLS certificate file.
}

impl ClientCertificate {
    pub fn new(path: impl AsRef<Path>) -> Self {
        ClientCertificate { cert_file: path.as_ref().to_path_buf() }
    }
}