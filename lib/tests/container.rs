use lenient_semver::Version;
use neo4rs::{ConfigBuilder, Graph};
use testcontainers::{runners::AsyncRunner, ContainerAsync, ContainerRequest, ImageExt};
use testcontainers_modules::neo4j::{Neo4j, Neo4jImage};

use std::{error::Error, io::BufRead as _};

#[allow(dead_code)]
#[derive(Default)]
pub struct Neo4jContainerBuilder {
    enterprise: bool,
    config: ConfigBuilder,
    env: Vec<(String, String)>,
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

    pub fn with_driver_config(mut self, config: ConfigBuilder) -> Self {
        self.config = config;
        self
    }

    pub fn modify_driver_config(
        mut self,
        block: impl FnOnce(ConfigBuilder) -> ConfigBuilder,
    ) -> Self {
        self.config = block(self.config);
        self
    }

    pub fn add_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn with_server_config(self, key: &str, value: impl Into<String>) -> Self {
        let key = format!("NEO4J_{}", key.replace('_', "__").replace('.', "_"));
        self.add_env(key, value)
    }

    pub async fn start(self) -> Result<Neo4jContainer, Box<dyn Error + Send + Sync + 'static>> {
        Neo4jContainer::from_config_and_edition_and_env(self.config, self.enterprise, self.env)
            .await
    }
}

pub struct Neo4jContainer {
    graph: Graph,
    version: String,
    _container: Option<ContainerAsync<Neo4jImage>>,
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
        Self::from_config_and_edition_and_env::<_, String, String>(config, enterprise_edition, [])
            .await
    }

    pub async fn from_config_and_edition_and_env<I, K, V>(
        config: ConfigBuilder,
        enterprise_edition: bool,
        env_vars: I,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let _ = pretty_env_logger::try_init();

        let server = Self::server_from_env();
        let connection = Self::create_test_endpoint(matches!(server, TestServer::Aura(_)));

        let (uri, _container) = match server {
            TestServer::TestContainer => {
                let (uri, container) = Self::create_testcontainer(
                    &connection,
                    enterprise_edition,
                    env_vars.into_iter().map(|(k, v)| (k.into(), v.into())),
                )
                .await?;
                (uri, Some(container))
            }
            TestServer::External(uri) | TestServer::Aura(uri) => (uri, None),
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
        const CHECK_AURA_VAR: &str = "NEO4RS_TEST_ON_AURA";
        const AURA_URI_VAR: &str = "NEO4J_URI";

        use std::env::var;

        var(CHECK_AURA_VAR)
            .ok()
            .filter(|use_aura| use_aura == "1")
            .and_then(|_| var(AURA_URI_VAR).ok().map(TestServer::Aura))
            .or_else(|| var(TEST_URI_VAR).ok().map(TestServer::External))
            .unwrap_or(TestServer::TestContainer)
    }

    async fn create_testcontainer<I>(
        connection: &TestConnection,
        enterprise: bool,
        env_vars: I,
    ) -> Result<(String, ContainerAsync<Neo4jImage>), Box<dyn Error + Send + Sync + 'static>>
    where
        I: Iterator<Item = (String, String)>,
    {
        let container = Self::create_testcontainer_image(connection, enterprise, env_vars)?;
        let container = container.start().await?;

        let uri = format!("bolt://127.0.0.1:{}", container.image().bolt_port_ipv4()?);

        Ok((uri, container))
    }

    fn create_testcontainer_image<I>(
        connection: &TestConnection,
        enterprise: bool,
        env_vars: I,
    ) -> Result<ContainerRequest<Neo4jImage>, Box<dyn Error + Send + Sync + 'static>>
    where
        I: Iterator<Item = (String, String)>,
    {
        let image = Neo4j::new()
            .with_user(connection.auth.user.to_owned())
            .with_password(connection.auth.pass.to_owned());

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

            env_vars.fold(
                image
                    .with_version(version)
                    .with_env_var("NEO4J_ACCEPT_LICENSE_AGREEMENT", "yes"),
                |i, (k, v)| i.with_env_var(k, v),
            )
        } else {
            image.with_version(connection.version.to_owned()).into()
        };

        Ok(container)
    }

    fn create_test_endpoint(use_aura: bool) -> TestConnection {
        const USER_VAR: &str = "NEO4J_TEST_USER";
        const AURA_USER_VAR: &str = "NEO4J_USERNAME";
        const PASS_VAR: &str = "NEO4J_TEST_PASS";
        const AURA_PASS_VAR: &str = "NEO4J_PASSWORD";
        const VERSION_VAR: &str = "NEO4J_VERSION_TAG";

        const DEFAULT_USER: &str = "neo4j";
        const DEFAULT_PASS: &str = "neo";
        const DEFAULT_VERSION_TAG: &str = "5";

        use std::env::var;

        let user = var(if use_aura { AURA_USER_VAR } else { USER_VAR })
            .unwrap_or_else(|_| DEFAULT_USER.to_owned());
        let pass = var(if use_aura { AURA_PASS_VAR } else { PASS_VAR })
            .unwrap_or_else(|_| DEFAULT_PASS.to_owned());
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
    Aura(String),
    External(String),
}
