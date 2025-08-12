use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionTLSConfig {
    None,
    ClientCACertificate(ClientCertificate),
    NoSSLValidation,
    MutualTLS(MutualTLS),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientCertificate {
    pub(crate) cert_file: PathBuf,
}

impl ClientCertificate {
    pub fn new(path: impl AsRef<Path>) -> Self {
        ClientCertificate {
            cert_file: path.as_ref().to_path_buf(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutualTLS {
    pub(crate) validation: bool,
    pub(crate) cert_file: Option<PathBuf>,
    pub(crate) client_cert: PathBuf,
    pub(crate) client_key: PathBuf,
}

impl MutualTLS {
    pub fn new(
        cert_file: Option<impl AsRef<Path>>,
        client_cert: impl AsRef<Path>,
        client_key: impl AsRef<Path>,
    ) -> Self {
        MutualTLS {
            validation: true,
            cert_file: cert_file.map(|p| p.as_ref().to_path_buf()),
            client_cert: client_cert.as_ref().to_path_buf(),
            client_key: client_key.as_ref().to_path_buf(),
        }
    }
    pub fn with_no_validation(&self) -> Self {
        Self {
            validation: false,
            ..self.clone()
        }
    }
}
