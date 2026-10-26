<?php

declare(strict_types=1);

use Claviron\Symfony\ApplicationRunner;
use Skene\Example\Kernel;

require __DIR__.'/vendor/autoload.php';

$environment = $_SERVER['APP_ENV'] ?? $_ENV['APP_ENV'] ?? 'prod';
$debug = filter_var($_SERVER['APP_DEBUG'] ?? $_ENV['APP_DEBUG'] ?? false, FILTER_VALIDATE_BOOL);

ApplicationRunner::run(new Kernel($environment, $debug));
