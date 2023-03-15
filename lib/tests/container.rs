use lenient_semver::Version;
use neo4rs::{config, ConfigBuilder, Graph};
use testcontainers::{clients::Cli, core::WaitFor, Container, Image};

use std::{collections::HashMap, sync::Arc};

pub struct Neo4jContainer {
    graph: Arc<Graph>,
    version: String,
    _container: Container<'static, Neo4j>,
}

impl Neo4jContainer {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        Self::from_version(Self::version_from_env()).await
    }

    #[allow(dead_code)]
    pub async fn from_version(version: impl Into<String>) -> Self {
        Self::from_config_and_version(config(), version).await
    }

    #[allow(dead_code)]
    pub async fn from_config(config: ConfigBuilder) -> Self {
        Self::from_config_and_version(config, Self::version_from_env()).await
    }

    #[allow(dead_code)]
    pub async fn from_config_and_version(
        config: ConfigBuilder,
        version: impl Into<String>,
    ) -> Self {
        let _ = pretty_env_logger::try_init();

        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let version = version.into();
        let container = docker.run(Neo4j::new(USER, PASS, version.clone()));

        let bolt_port = container.ports().map_to_host_port_ipv4(7687).unwrap();
        let uri = format!("127.0.0.1:{}", bolt_port);

        let config = config.uri(&uri).user(USER).password(PASS).build().unwrap();
        let graph = Graph::connect(config).await.unwrap();
        let graph = Arc::new(graph);

        Self {
            graph,
            version,
            _container: container,
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

    fn version_from_env() -> String {
        const VERSION_VAR: &str = "NEO4J_VERSION_TAG";
        const DEFAULT_VERSION_TAG: &str = "4.2";

        std::env::var(VERSION_VAR).unwrap_or_else(|_| DEFAULT_VERSION_TAG.to_owned())
    }
}

const USER: &str = "neo4j";
const PASS: &str = "neo";

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
