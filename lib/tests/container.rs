use lenient_semver::Version;
use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{clients::Cli, core::WaitFor, Container, Image};

use std::{collections::HashMap, sync::Arc};

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

        let (server, version) = Self::server_from_env();

        let (connection, _container) = match server {
            TestServer::TestContainer { auth } => {
                let (uri, container) = Self::create_testcontainer(&auth, &version).await;
                (TestConnection { uri, auth }, Some(container))
            }
            TestServer::External { connection } => (connection, None),
        };

        let graph = Self::connect(config, connection).await;
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

    fn server_from_env() -> (TestServer, String) {
        const USER_VAR: &str = "NEO4J_TEST_USER";
        const PASS_VAR: &str = "NEO4J_TEST_PASS";
        const TEST_URI_VAR: &str = "NEO4J_TEST_URI";
        const VERSION_VAR: &str = "NEO4J_VERSION_TAG";

        const DEFAULT_USER: &str = "neo4j";
        const DEFAULT_PASS: &str = "neo";
        const DEFAULT_VERSION_TAG: &str = "4.2";

        use std::env::var;

        let user = var(USER_VAR).unwrap_or_else(|_| DEFAULT_USER.to_owned());
        let pass = var(PASS_VAR).unwrap_or_else(|_| DEFAULT_PASS.to_owned());
        let auth = TestAuth { user, pass };

        let version = var(VERSION_VAR).unwrap_or_else(|_| DEFAULT_VERSION_TAG.to_owned());

        if let Ok(uri) = var(TEST_URI_VAR) {
            let config = TestConnection { uri, auth };
            (TestServer::External { connection: config }, version)
        } else {
            (TestServer::TestContainer { auth }, version)
        }
    }

    async fn create_testcontainer(
        auth: &TestAuth,
        version: &str,
    ) -> (String, Container<'static, Neo4j>) {
        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = docker.run(Neo4j::new(&auth.user, &auth.pass, version.to_owned()));

        let bolt_port = container.ports().map_to_host_port_ipv4(7687).unwrap();
        let uri = format!("bolt://127.0.0.1:{}", bolt_port);

        (uri, container)
    }

    async fn connect(config: ConfigBuilder, info: TestConnection) -> Arc<Graph> {
        let config = config
            .uri(&info.uri)
            .user(&info.auth.user)
            .password(&info.auth.pass)
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
    auth: TestAuth,
}

enum TestServer {
    TestContainer { auth: TestAuth },
    External { connection: TestConnection },
}

#[derive(Debug)]
struct Neo4j {
    version: String,
    env_vars: HashMap<String, String>,
}

impl Neo4j {
    fn new(user: &str, pass: &str, version: String) -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("NEO4J_AUTH".to_owned(), format!("{user}/{pass}"));
        env_vars.insert(
            "NEO4J_dbms_security_auth__minimum__password__length".to_owned(),
            "3".to_owned(),
        );

        Self { env_vars, version }
    }
}

impl Image for Neo4j {
    type Args = ();

    fn name(&self) -> String {
        "neo4j".to_owned()
    }

    fn tag(&self) -> String {
        self.version.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Bolt enabled on"),
            WaitFor::message_on_stdout("Started."),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
