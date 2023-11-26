use lenient_semver::Version;
use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{clients::Cli, Container, RunnableImage};
use testcontainers_modules::neo4j::{Neo4j, Neo4jImage};

use std::{error::Error, io::BufRead as _};

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

        let connection = Self::create_test_endpoint();
        let server = Self::server_from_env();

        let (uri, _container) = match server {
            TestServer::TestContainer => {
                let (uri, container) = Self::create_testcontainer(&connection, enterprise_edition)?;
                (uri, Some(container))
            }
            TestServer::External(uri) => (uri, None),
        };

        let version = connection.version;
        let graph = Self::connect(config, uri, &connection.auth).await;
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
        connection: &TestConnection,
        enterprise: bool,
    ) -> Result<(String, Container<'static, Neo4jImage>), Box<dyn Error + Send + Sync + 'static>>
    {
        let image = Neo4j::new()
            .with_user(connection.auth.user.to_owned())
            .with_password(connection.auth.pass.to_owned());

        let docker = Cli::default();
        let docker = Box::leak(Box::new(docker));

        let container = if enterprise {
            const ACCEPTANCE_FILE_NAME: &str = "container-license-acceptance.txt";

            let version = format!("{}-enterprise", connection.version);
            let image_name = format!("neo4j:{}", version);

            let acceptance_file = std::env::current_dir()
                .ok()
                .map(|o| o.join(ACCEPTANCE_FILE_NAME));

            let has_license_acceptance = acceptance_file
                .as_deref()
                .and_then(|o| std::fs::File::open(o).ok())
                .into_iter()
                .flat_map(|o| std::io::BufReader::new(o).lines())
                .any(|o| o.map_or(false, |line| line.trim() == image_name));

            if !has_license_acceptance {
                return Err(format!(
                    concat!(
                        "You need to accept the Neo4j Enterprise Edition license by ",
                        "creating the file `{}` with the following content:\n\n\t{}",
                    ),
                    acceptance_file.map_or_else(
                        || ACCEPTANCE_FILE_NAME.to_owned(),
                        |o| { o.display().to_string() }
                    ),
                    image_name
                )
                .into());
            }
            let image: RunnableImage<Neo4jImage> = image.with_version(version).into();
            let image = image.with_env_var(("NEO4J_ACCEPT_LICENSE_AGREEMENT", "yes"));

            docker.run(image)
        } else {
            docker.run(image.with_version(connection.version.to_owned()))
        };

        let uri = format!("bolt://127.0.0.1:{}", container.image().bolt_port_ipv4());

        Ok((uri, container))
    }

    fn create_test_endpoint() -> TestConnection {
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

        TestConnection { auth, version }
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
    version: String,
    auth: TestAuth,
}

enum TestServer {
    TestContainer,
    External(String),
}
