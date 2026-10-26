# Skene

Run Symfony applications on Claviron. Skene connects the application to a
Claviron edge through Orbit and can also serve directly over TCP or a Unix socket.

Your application keeps its Symfony routes, controllers, services and request
lifecycle. Claviron supplies the PHP engine, application pools, worker processes
and graceful shutdown. In application mode, each PHP context boots its kernel once and handles
successive requests through the `claviron/symfony` adapter.

## Run an application

Skene is under development. Build the native executable from this repository
with a ZTS embed PHP toolchain available through `PHP` and `PHP_CONFIG`:

```sh
cargo build --release
```

Install the PHP adapter in your Symfony application:

```sh
composer require claviron/symfony:^0.1
cp vendor/claviron/symfony/stubs/application.php application.php
```

Create `skene.yaml` in the application root:

```yaml
name: symfony-app
fleet_name: symfony
orbit: true
workers: 4
php:
  application:
    script_path: application.php
    env:
      APP_ENV: prod
      APP_DEBUG: "0"
```

Run `skene --config skene.yaml` from that directory, or select it with
`--root /path/to/application`. The edge routes to application `symfony-app`
in fleet `symfony`. Multiple workers advertise separate lanes for the same
application. Omit `workers`, or set it to zero, for standalone execution.

`--check-config` validates the native configuration without starting listeners
or application workers. A working configuration check does not execute or
validate the Symfony kernel. Ctrl+C and service-manager termination use
Claviron's normal graceful shutdown.

The [Symfony example](examples/symfony) contains a minimal application and
buffered GET/POST routes. The native executable is separate from Composer.

## Configuration

`skene.yaml` combines Claviron's shared runner options with its single-application
PHP schema. `tcp` and `uds` select socket ingress; `orbit` selects Link ingress.
Request and response policies, pool sizes, retries and execution budgets use
Claviron's existing settings. Paths resolve from the application's working
directory, independently of the configuration file's location.

Symfony's environment is explicit under `php.application.env`. Native
`--production` selects the server environment; it does not rewrite that PHP
policy. Prepare Symfony's production container/router cache before startup.
Optional `php.engine.warmup` folders prime each worker's OPcache in the
background. PHP's own INI controls cache freshness.

When `ini_path` and `ini_scan_dir` are omitted, PHP uses its normal configuration
search, including `PHPRC`, `PHP_INI_SCAN_DIR` and its compiled default paths.

The `supervisor` and `cli` features are enabled by default. Optional `files`,
`axum` and `bench` forward to the existing Claviron capabilities.
Applications can use the `Runner` library to register their own native services
and Rust routes while retaining Symfony as the fallback.

The current PHP adapter handles buffered HTTP requests and responses. See
[architecture](ARCHITECTURE.md) for ownership, lifecycle and configuration boundaries.
