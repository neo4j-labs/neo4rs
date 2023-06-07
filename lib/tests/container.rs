use lenient_semver::Version;
use neo4j_testcontainers::Neo4j;
use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{clients::Cli, Container};

use std::sync::Arc;

pub struct Neo4jContainer {
    graph: Arc<Graph>,
    version: String,
    _container: Option<Container<'static, Neo4j>>,
}

impl Neo4jContainer {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        Self::from_config(ConfigBuilder::default()).await
    }

    pub async fn from_config(config: ConfigBuilder) -> Self {
        let _ = pretty_env_logger::try_init();

        let server = Self::server_from_env();

        let (connection, _container) = match server {
            TestServer::TestContainer => {
                let (connection, container) = Self::create_testcontainer();
                (connection, Some(container))
            }
            TestServer::External(uri) => {
                let connection = Self::create_test_endpoint(uri);
                (connection, None)
            }
        };

        let version = connection.version;
        let graph = Self::connect(config, connection.uri, &connection.auth).await;
        Self {
            graph,
            version,
            _container,
        }
    }

    pub fn graph(&self) -> Arc<Graph> {
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

    fn create_testcontainer() -> (TestConnection, Container<'static, Neo4j>) {
        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = docker.run(Neo4j::default());

        let uri = Neo4j::uri_ipv4(&container);
        let version = container.image().version().to_owned();
        let user = container.image().user().to_owned();
        let pass = container.image().pass().to_owned();
        let auth = TestAuth { user, pass };

        let connection = TestConnection { uri, version, auth };

        (connection, container)
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

    async fn connect(config: ConfigBuilder, uri: String, auth: &TestAuth) -> Arc<Graph> {
        let config = config
            .uri(uri)
            .user(&auth.user)
            .password(&auth.pass)
            .build()
            .unwrap();

        let graph = Graph::connect(config).await.unwrap();

        Arc::new(graph)
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
