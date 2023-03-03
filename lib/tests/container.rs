use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{clients::Cli, core::WaitFor, Container, Image};

use std::{collections::HashMap, sync::Arc};

pub struct Neo4jContainer {
    graph: Arc<Graph>,
    _container: Container<'static, Neo4j>,
}

impl Neo4jContainer {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = docker.run(Neo4j::new(USER, PASS));

        let bolt_port = container.ports().map_to_host_port_ipv4(7687).unwrap();
        let uri = format!("127.0.0.1:{}", bolt_port);
        let graph = Arc::new(Graph::new(&uri, USER, PASS).await.unwrap());

        Self {
            graph,
            _container: container,
        }
    }

    #[allow(dead_code)]
    pub async fn from_config(config: ConfigBuilder) -> Self {
        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = docker.run(Neo4j::new(USER, PASS));

        let bolt_port = container.ports().map_to_host_port_ipv4(7687).unwrap();
        let uri = format!("127.0.0.1:{}", bolt_port);

        let config = config.uri(&uri).user(USER).password(PASS).build().unwrap();
        let graph = Graph::connect(config).await.unwrap();
        let graph = Arc::new(graph);

        Self {
            graph,
            _container: container,
        }
    }

    pub fn graph(&self) -> Arc<Graph> {
        self.graph.clone()
    }
}

const USER: &str = "neo4j";
const PASS: &str = "neo";

#[derive(Debug)]
struct Neo4j {
    env_vars: HashMap<String, String>,
}

impl Neo4j {
    fn new(user: &str, pass: &str) -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("NEO4J_AUTH".to_owned(), format!("{user}/{pass}"));

        Self { env_vars }
    }
}

impl Image for Neo4j {
    type Args = ();

    fn name(&self) -> String {
        "neo4j".to_owned()
    }

    fn tag(&self) -> String {
        "4.2".to_owned()
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
