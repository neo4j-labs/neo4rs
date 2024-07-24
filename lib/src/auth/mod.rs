mod static_client_certificate_provider;

#[derive(Debug, Clone)]
pub struct ClientCertificate {
    pub(crate) cert_file: String,  // Path to the TLS certificate file.
}

pub trait ClientCertificateProvider {
    fn get_certificate(&self) -> ClientCertificate;
}

pub use static_client_certificate_provider::StaticClientCertificateProvider;