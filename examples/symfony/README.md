# Symfony application

Install dependencies here with `composer install`. Build Skene from the
repository root, then run `../../target/release/skene --config skene.yaml`
from this directory. The example advertises `symfony-app` in the `symfony`
Orbit fleet. An edge can route to that name without a TCP listener.

For direct local HTTP, add `--tcp 127.0.0.1:8000`. `GET /` returns a greeting,
`GET /state` reports the process, persistent controller identity and request
count, and `POST /echo` returns the raw request body. These routes use no
database or session. The fixed framework secret is only for this example.

`--workers 2` runs two supervised application processes. Each owns its PHP
context and kernel; the master does not execute the application. Use an isolated
fleet name and listener when running live acceptance beside another service.
