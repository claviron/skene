use std::sync::Arc;

use claviron::orbit::link::application::LINK_INBOX_CAPACITY;
use claviron::runner::{Composition, Owner};
use claviron::runtime::{AppState, Register, registry};

use crate::Options;
use crate::application::{Application, Symfony};
use crate::options::Launch;

type Registration = Box<dyn FnOnce(&AppState) -> registry::Result<()> + Send>;

/// One Symfony application served through Claviron's existing runner shell.
#[derive(Default)]
pub struct Runner {
    options: Options,
    registrations: Vec<Registration>,
    #[cfg(feature = "axum")]
    interceptor: Option<Registration>,
}

impl Runner {
    /// Create a runner whose application name is supplied by its options.
    pub fn new() -> Self {
        Self::default()
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
        let name = self.options.runner.name.take().ok_or("name is required")?;
        self.options
            .runner
            .frameworks
            .get_or_insert_with(|| vec!["Symfony".into()]);
        self.options.runner.icon.get_or_insert_with(|| "symfony".into());

        let config_path = self.options.config_path.clone();
        let check_config = self.options.check_config;
        let system_log = self.options.configuration.logging.clone();
        self.options.configuration.config_path = Some(self.options.config_path);
        let launch = Launch {
            runner: self.options.runner,
            own: self.options.configuration,
        };
        let definition_name = name.clone();
        let application_name = name.clone();
        let registrations = self.registrations;
        #[cfg(feature = "axum")]
        let interceptor = self.interceptor;

        let composition =
            Composition::new(launch, name, "php", move |state, _| {
                let service = Symfony::default();
                state.registry_ref().register(Arc::new(Application {
                    name: application_name,
                    service: service.clone(),
                }))?;
                Ok(Owner::new(service))
            })
            .default_in_flight(Some(LINK_INBOX_CAPACITY))
            .config_path(&config_path)
            .system_log(system_log)
            .definitions(move |state, launch| {
                launch.own.register(
                    state,
                    &definition_name,
                    launch.runner.frameworks.clone().unwrap_or_default(),
                    launch.runner.icon.clone(),
                )?;
                claviron_php::register(state)
            })
            .configure(move |state, _| {
                for register in registrations {
                    register(state)?;
                }
                #[cfg(feature = "axum")]
                if let Some(interceptor) = interceptor {
                    interceptor(state)?;
                }
                Ok(())
            });

        if check_config {
            composition.check_config().await?;
        } else {
            composition.serve().await?;
        }
        Ok(())
    }
}
