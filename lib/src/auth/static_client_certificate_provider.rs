use crate::auth::{ClientCertificate, ClientCertificateProvider};

pub struct StaticClientCertificateProvider {
    certificate: ClientCertificate
}

impl StaticClientCertificateProvider {
    pub fn new(cert_file: String) -> Self {
        Self {
            certificate: ClientCertificate {
                cert_file,
            }
        }
    }
}

impl ClientCertificateProvider for StaticClientCertificateProvider {
    fn get_certificate(&self) -> ClientCertificate {
        self.certificate.clone()
    }
}