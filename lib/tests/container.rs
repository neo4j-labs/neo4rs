use lenient_semver::Version;
use neo4j_testcontainers::{Neo4j, Neo4jImage};
use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{clients::Cli, Container};

use std::error::Error;

#[allow(dead_code)]
#[derive(Default)]
pub struct Neo4jContainerBuilder {
    enterprise: bool,
    config: ConfigBuilder,
}

#[allow(dead_code)]
impl Neo4jContainerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enterprise_edition(mut self) -> Self {
        self.enterprise = true;
        self
    }

    pub fn with_config(mut self, config: ConfigBuilder) -> Self {
        self.config = config;
        self
    }

    pub fn modify_config(mut self, block: impl FnOnce(ConfigBuilder) -> ConfigBuilder) -> Self {
        self.config = block(self.config);
        self
    }

    pub async fn start(self) -> Result<Neo4jContainer, Box<dyn Error + Send + Sync + 'static>> {
        Neo4jContainer::from_config_and_edition(self.config, self.enterprise).await
    }
}

pub struct Neo4jContainer {
    graph: Graph,
    version: String,
    _container: Option<Container<'static, Neo4jImage>>,
}

impl Neo4jContainer {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        Self::from_config(ConfigBuilder::default()).await
    }

    pub async fn from_config(config: ConfigBuilder) -> Self {
        Self::from_config_and_edition(config, false).await.unwrap()
    }

    pub async fn from_config_and_edition(
        config: ConfigBuilder,
        enterprise_edition: bool,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let _ = pretty_env_logger::try_init();

        let server = Self::server_from_env();

        let (connection, _container) = match server {
            TestServer::TestContainer => {
                let (connection, container) = Self::create_testcontainer(enterprise_edition)?;
                (connection, Some(container))
            }
            TestServer::External(uri) => {
                let connection = Self::create_test_endpoint(uri);
                (connection, None)
            }
        };

        let version = connection.version;
        let graph = Self::connect(config, connection.uri, &connection.auth).await;
        Ok(Self {
            graph,
            version,
            _container,
        })
    }

    pub fn graph(&self) -> Graph {
        self.graph.clone()
    }

    #[allow(dead_code)]
    pub fn version(&self) -> Version<'static> {
        Version::parse(&self.version)
            .unwrap()
            .disassociate_metadata()
            .0
    }

    fn server_from_env() -> TestServer {
        const TEST_URI_VAR: &str = "NEO4J_TEST_URI";

        if let Ok(uri) = std::env::var(TEST_URI_VAR) {
            TestServer::External(uri)
        } else {
            TestServer::TestContainer
        }
    }

    fn create_testcontainer(
        enterprise: bool,
    ) -> Result<
        (TestConnection, Container<'static, Neo4jImage>),
        Box<dyn Error + Send + Sync + 'static>,
    > {
        let image = Neo4j::default();
        let image = if enterprise {
            image.with_enterprise_edition()?
        } else {
            image
        };

        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = docker.run(image);

        let uri = container.image().bolt_uri_ipv4();
        let version = container.image().version().to_owned();
        let user = container.image().user().expect("default user").to_owned();
        let pass = container
            .image()
            .password()
            .expect("default password")
            .to_owned();
        let auth = TestAuth { user, pass };

        let connection = TestConnection { uri, version, auth };

        Ok((connection, container))
    }

    fn create_test_endpoint(uri: String) -> TestConnection {
        const USER_VAR: &str = "NEO4J_TEST_USER";
        const PASS_VAR: &str = "NEO4J_TEST_PASS";
        const VERSION_VAR: &str = "NEO4J_VERSION_TAG";

        const DEFAULT_USER: &str = "neo4j";
        const DEFAULT_PASS: &str = "neo";
        const DEFAULT_VERSION_TAG: &str = "5";

        use std::env::var;

        let user = var(USER_VAR).unwrap_or_else(|_| DEFAULT_USER.to_owned());
        let pass = var(PASS_VAR).unwrap_or_else(|_| DEFAULT_PASS.to_owned());
        let auth = TestAuth { user, pass };
        let version = var(VERSION_VAR).unwrap_or_else(|_| DEFAULT_VERSION_TAG.to_owned());

        TestConnection { uri, auth, version }
    }

    async fn connect(config: ConfigBuilder, uri: String, auth: &TestAuth) -> Graph {
        let config = config
            .uri(uri)
            .user(&auth.user)
            .password(&auth.pass)
            .build()
            .unwrap();

        Graph::connect(config).await.unwrap()
    }
}

struct TestAuth {
    user: String,
    pass: String,
}

struct TestConnection {
    uri: String,
    version: String,
    auth: TestAuth,
}

enum TestServer {
    TestContainer,
    External(String),
}
