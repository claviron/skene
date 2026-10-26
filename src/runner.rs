use std::sync::Arc;

use claviron::Server;
use claviron::runner::{Owner, launch};
use claviron::runtime::{AppState, Register, ServerName, registry};

use crate::Options;
use crate::application::{Application, Symfony};
use crate::options::Launch;

type Registration = Box<dyn FnOnce(&AppState) -> registry::Result<()> + Send>;

/// One Symfony application served through Claviron's existing runner shell.
pub struct Runner {
    name: String,
    options: Options,
    registrations: Vec<Registration>,
    #[cfg(feature = "axum")]
    interceptor: Option<Registration>,
}

impl Runner {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            options: Options::default(),
            registrations: Vec::new(),
            #[cfg(feature = "axum")]
            interceptor: None,
        }
    }

    pub fn with_options(
        mut self,
        options: Options,
    ) -> Self {
        self.options = options;
        self
    }

    pub fn with_config_path(
        mut self,
        path: impl AsRef<std::path::Path>,
    ) -> crate::Result<Self> {
        self.options = Options::from_file(path)?;
        Ok(self)
    }

    pub fn with_services<C: Register<AppState> + 'static>(self) -> Self {
        self.with_bootstrap(C::register)
    }

    pub fn with_bootstrap<F>(
        mut self,
        register: F,
    ) -> Self
    where
        F: FnOnce(&AppState) -> registry::Result<()> + Send + 'static,
    {
        self.registrations.push(Box::new(register));
        self
    }

    /// Rust routes intercept; unmatched requests still reach Symfony.
    #[cfg(feature = "axum")]
    pub fn with_interceptor<F>(
        mut self,
        router: F,
    ) -> Self
    where
        F: FnOnce(AppState) -> claviron::axum::Router + Send + 'static,
    {
        self.interceptor = Some(Box::new(move |state| {
            claviron::runner::register_router(state, router)
        }));
        self
    }

    pub async fn run(mut self) -> crate::Result<()> {
        self.options.validate()?;
        let name = self.options.runner.name.clone().unwrap_or(self.name);
        claviron::orbit::transport::segment::check_application_name(&name)?;
        let server_name: ServerName = name.parse()?;
        self.options
            .runner
            .frameworks
            .get_or_insert_with(|| vec!["Symfony".into()]);
        self.options.runner.icon.get_or_insert_with(|| "symfony".into());

        let config_dir = self
            .options
            .config_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_owned();
        let production = self.options.runner.production;
        let environment = self.options.runner.environment.clone();
        let check_config = self.options.check_config;
        let system_log = self.options.configuration.logging.clone();
        self.options.configuration.config_path = Some(self.options.config_path);
        let mut launch = Launch {
            runner: self.options.runner,
            own: self.options.configuration,
        };
        let prepared = launch.prepare(name)?;
        let definitions = launch.clone();
        let definition_name = prepared.name.clone();
        let application_name = prepared.name.clone();
        let registrations = self.registrations;
        #[cfg(feature = "axum")]
        let interceptor = self.interceptor;

        let server = match (production, environment) {
            (true, _) => Server::new().production(),
            (false, Some(environment)) => {
                Server::new().environment(environment)
            }
            (false, None) => Server::new(),
        };
        #[cfg(feature = "supervisor")]
        let server = server.workers(prepared.workers);

        let server = server
            .name(server_name)
            .fleet_name(prepared.fleet_name)
            .launch(prepared.envelope)
            .config_dir(config_dir)
            .system_log(system_log)
            .with_definitions(move |state| {
                let definitions = definitions.inherited(state)?;
                #[cfg(feature = "supervisor")]
                claviron::runner::supervisor::register_policy(
                    state,
                    definitions.runner.supervisor.clone().unwrap_or_default(),
                )?;
                definitions.own.register(
                    state,
                    &definition_name,
                    definitions.runner.frameworks.clone().unwrap_or_default(),
                    definitions.runner.icon.clone(),
                )
            })
            .register(move |state| {
                let launch = launch.inherited(state)?;
                let runner = &launch.runner;
                let service = Symfony::default();
                state.registry_ref().register(Arc::new(Application {
                    name: application_name,
                    service: service.clone(),
                }))?;
                claviron::runner::register_owner(state, Owner::new(service))?;
                let root = std::env::current_dir()
                    .map_err(|error| registry::Error::msg(error.to_string()))?;
                launch::register_routing(state, runner, &root)?;
                #[cfg(feature = "bench")]
                claviron::bench::runner::BenchService::register(state)?;
                for register in registrations {
                    register(state)?;
                }
                #[cfg(feature = "axum")]
                if let Some(interceptor) = interceptor {
                    interceptor(state)?;
                }
                claviron::runner::ensure_application(state)?;
                launch::register_trust(state, runner)?;
                launch::register_doors(
                    state,
                    runner,
                    "php",
                    Some(claviron::orbit::transport::TRANSPORT_INBOX_CAPACITY),
                )
            });

        if check_config {
            server.check_config().await?;
        } else {
            server.serve().await?;
        }
        Ok(())
    }
}
