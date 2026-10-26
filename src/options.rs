//! One launch document, with Claviron's PHP schema kept intact.

use std::path::{Path, PathBuf};

use claviron::logging::SystemLogConfig;
use claviron::runner::RunnerOptions;
use serde::{Deserialize, Serialize};

pub(crate) type Launch = claviron::runner::launch::Launch<Configuration>;

/// Native ingress options and the application's existing PHP configuration.
#[derive(Clone, Debug)]
pub struct Options {
    pub runner: RunnerOptions,
    pub configuration: Configuration,
    pub config_path: PathBuf,
    pub check_config: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            runner: RunnerOptions::default(),
            configuration: Configuration::default(),
            config_path: "skene.yaml".into(),
            check_config: false,
        }
    }
}

/// The Symfony runner's sections beside Claviron's shared runner options.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Configuration {
    /// Claviron's single-application PHP schema, including engine and pool.
    pub php: serde_yaml::Value,
    pub logging: Option<SystemLogConfig>,
    /// Source identity carried to re-executed workers.
    pub(crate) config_path: Option<PathBuf>,
}

impl Options {
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();
        Self::from_yaml(&std::fs::read_to_string(path)?, path)
    }

    /// Parse without loading environment files or booting a PHP engine.
    pub fn from_yaml(
        raw: &str,
        path: impl Into<PathBuf>,
    ) -> crate::Result<Self> {
        let launch: Launch = serde_yaml::from_str(raw)?;
        let options = Self {
            runner: launch.runner,
            configuration: launch.own,
            config_path: path.into(),
            check_config: false,
        };
        options.validate()?;
        Ok(options)
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.runner.validate()?;
        if self.config_path.as_os_str().is_empty() {
            return Err("config_path must name a file".into());
        }
        if self.runner.tcp.is_none()
            && self.runner.uds.is_none()
            && !self.runner.orbit
        {
            return Err("select orbit, tcp or uds ingress".into());
        }
        if let Some(logging) = &self.configuration.logging {
            logging.validate()?;
        }
        self.configuration.validate()?;
        Ok(())
    }

    #[cfg(feature = "cli")]
    pub fn from_args() -> crate::Result<Self> {
        use clap::Parser;

        #[cfg(feature = "supervisor")]
        if let claviron::supervisor::ProcessEntry::Worker(worker) =
            claviron::supervisor::ProcessEntry::detect()?
        {
            let launch: Launch =
                serde_json::from_slice(worker.launch().as_bytes())?;
            return Ok(Self {
                config_path: launch
                    .own
                    .config_path
                    .clone()
                    .unwrap_or_else(|| "skene.yaml".into()),
                runner: launch.runner,
                configuration: launch.own,
                check_config: false,
            });
        }

        #[derive(Parser)]
        #[command(version, about = "Run a persistent Symfony application")]
        struct Args {
            /// Application directory; paths inside the config are relative to it.
            #[arg(long)]
            root: Option<PathBuf>,
            #[arg(long, default_value = "skene.yaml")]
            config: PathBuf,
            /// Validate providers without starting applications or listeners.
            #[arg(long)]
            check_config: bool,
            #[command(flatten)]
            runner: RunnerOptions,
        }

        let args = Args::parse();
        if let Some(root) = args.root {
            std::env::set_current_dir(root)?;
        }
        // Read before validating ingress: CLI may provide the only door.
        let launch: Launch =
            serde_yaml::from_str(&std::fs::read_to_string(&args.config)?)?;
        let options = Self {
            runner: args.runner.over(launch.runner),
            configuration: launch.own,
            config_path: args.config,
            check_config: args.check_config,
        };
        options.validate()?;
        Ok(options)
    }
}

impl Configuration {
    pub(crate) fn validate(&self) -> crate::Result<()> {
        let php = self.php.as_mapping().ok_or("php must be a mapping")?;
        if php.contains_key("applications") {
            return Err(
                "Skene serves one application; use php.application".into()
            );
        }
        if !php.get("application").is_some_and(serde_yaml::Value::is_mapping) {
            return Err("php.application must be a mapping".into());
        }
        if php.get("sapi").is_some_and(|sapi| !sapi.is_null()) {
            return Err(
                "Skene uses application mode; php.sapi must be null or omitted"
                    .into(),
            );
        }
        Ok(())
    }

    pub(crate) fn register(
        &self,
        state: &claviron::runtime::AppState,
        name: &str,
        frameworks: Vec<String>,
        icon: Option<String>,
    ) -> claviron::runtime::registry::Result<()> {
        use claviron::runtime::registry;

        let yaml = serde_yaml::to_string(&self.php)
            .map_err(|error| registry::Error::msg(error.to_string()))?;
        let description = claviron::app::ApplicationDescription {
            frameworks,
            icon,
            ..claviron::app::ApplicationDescription::new("php", "application")
                .runner("Skene")
        };
        claviron::ephp::register_application_from_yaml(
            state,
            name,
            &yaml,
            description,
        )
    }
}

#[cfg(test)]
#[path = "../tests/unit/options.rs"]
mod tests;
