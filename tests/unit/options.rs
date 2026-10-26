use super::{Launch, Options};

const APPLICATION: &str =
    "orbit: true\nphp:\n  application:\n    script_path: application.php\n";

#[test]
fn rejects_unknown_sections_and_accidental_script_execution() {
    for raw in [
        format!("{APPLICATION}request_mode: script\n"),
        format!("{APPLICATION}orbitt: true\n"),
        "orbit: true\nphp: {application: {}, sapi: {}}\n".into(),
        "orbit: true\nphp: {applications: {other: {script_path: other.php}}}\n"
            .into(),
        "orbit: true\nphp: {engine: {}}\n".into(),
    ] {
        assert!(Options::from_yaml(&raw, "skene.yaml").is_err());
    }
}

#[test]
fn requires_an_explicit_ingress() {
    let raw = "php:\n  application:\n    script_path: application.php\n";
    let error = Options::from_yaml(raw, "skene.yaml").unwrap_err();
    assert!(error.to_string().contains("select orbit, tcp or uds"));
}

#[test]
fn environment_recipes_survive_without_reading_or_expanding_them() {
    let raw = "orbit: true\nphp:\n  engine:\n    ini_path: ${PHP_INI}\n  application:\n    script_path: app.php\n    env_file: unavailable.env\n    env: {APP_ENV: prod, APP_DEBUG: '0'}\n";
    let options = Options::from_yaml(raw, "config/skene.yaml").unwrap();
    let php = &options.configuration.php;
    assert_eq!(php["engine"]["ini_path"].as_str(), Some("${PHP_INI}"));
    assert_eq!(
        php["application"]["env_file"].as_str(),
        Some("unavailable.env")
    );
    assert_eq!(php["application"]["env"]["APP_DEBUG"].as_str(), Some("0"));
}

#[test]
fn worker_envelope_keeps_application_policy_and_source() {
    let mut options =
        Options::from_yaml(APPLICATION, "config/selected.yaml").unwrap();
    options.runner.fleet_name = Some("example".into());
    #[cfg(feature = "supervisor")]
    {
        options.runner.workers = Some(2);
    }
    options.configuration.config_path = Some(options.config_path.clone());
    let mut launch =
        Launch { runner: options.runner, own: options.configuration };
    let prepared = launch.prepare("example-app".into()).unwrap();
    let worker: Launch = serde_json::from_slice(&prepared.envelope).unwrap();
    assert_eq!(worker.runner.name.as_deref(), Some("example-app"));
    assert_eq!(worker.runner.fleet_name.as_deref(), Some("example"));
    assert_eq!(
        worker.own.config_path.as_deref(),
        Some(std::path::Path::new("config/selected.yaml"))
    );
    assert_eq!(worker.own.php, launch.own.php);
    #[cfg(feature = "supervisor")]
    assert!(worker.runner.workers.is_none());
}

#[test]
fn invalid_external_fleet_name_is_a_fallible_launch_error() {
    let options = Options::from_yaml(APPLICATION, "skene.yaml").unwrap();
    let mut launch =
        Launch { runner: options.runner, own: options.configuration };
    launch.runner.fleet_name = Some("not a fleet".into());
    assert!(launch.prepare("example-app".into()).is_err());
}
