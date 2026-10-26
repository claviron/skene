# Skene architecture

Skene composes one persistent Symfony application on Claviron's runner shell.
The PHP kernel loop comes from the `claviron/symfony` Composer adapter. Skene
owns the native launch, application selection and binding of PHP to the shell.

## Ownership

- `options.rs` reads one `skene.yaml`: shared `RunnerOptions`, the existing PHP
  application schema, and an optional native system log. Unknown top-level
  keys fail. The PHP engine validates its own sections and environment recipes.
- `runner.rs` registers definitions, application-owned services and ingress.
  It never starts a second runtime. Worker launches inherit the master's
  serialized document rather than reopening a default configuration file.
- `application.rs` binds the named `PhpApplicationService` during provider boot.
  Its prepared adapter handles requests without resolving the registry again.
- `main.rs` provides the optional CLI. A custom Rust binary can use `Runner`
  directly and register its own services or routes.

Claviron owns PHP startup, thread pools, application generations, admission,
OPcache warmup, signals, supervision, Link management and graceful drain. A
master with workers serves no PHP. Each serving worker has its own PHP engine
state, application pool and Orbit lane. Standalone mode uses one serving process.

## Application contract

The application owns `application.php`, Composer dependencies, its Symfony
kernel, routes, services and cache generation. Its entry calls
`Claviron\Symfony\ApplicationRunner::run($kernel)`. That adapter boots the kernel
once per PHP context, receives buffered requests, responds, invokes Symfony
termination, and shuts the kernel down when Claviron ends native receive.
Cancellation and execution deadlines remain the PHP engine's contract.

The native description reports runtime `php`, runner `Skene`, request mode
`application` and framework `Symfony`. PHP configuration cannot enable a
script/SAPI pool or multiple application definitions. Skene does not infer
application mode from HTTP traffic or reconstruct a kernel for each request.

## Ingress and lifecycle

The existing runner shell supplies Orbit and optional TCP/Unix socket ingress,
request/response guards, trusted forwarding facts and optional static/Rust
routes. Every door reaches the same prepared PHP adapter. Orbit advertises the
configured application name in the independently configured fleet. It uses
Claviron's common transport and hosted-application management protocol.

The server's normal lifecycle registers and boots providers before ingress
runnables start. Shutdown stops admission and drains work through Claviron;
Skene adds no timeout, polling loop, kill policy or independent supervisor.
`--check-config` uses Claviron's isolated configuration-check path and starts
no listener, application runnable or worker.

## Configuration and updates

Paths are relative to the application process's working directory, including
paths in a configuration file located elsewhere. CLI `--root` selects that
directory before configuration loading. The library leaves cwd to its caller.
Inline PHP environment and env-file declarations use the existing engine API.
`--production` selects the native server environment; Symfony's `APP_ENV` and
`APP_DEBUG` are explicit application environment values.

Framework cache generation precedes worker startup. `php.engine.warmup` primes
each serving process's OPcache in the background without executing source or
gating traffic. Native INI controls freshness. Replacing the YAML requires a
new launch; PHP application restart reuses the prepared definition. Skene does
not implement file watching or a source deployment protocol.

## Verification

Rust tests exercise configuration contracts and worker-envelope serialization
without booting a runtime. The Symfony example is the live acceptance boundary
for buffered HTTP, persistent state, process isolation and graceful shutdown.
