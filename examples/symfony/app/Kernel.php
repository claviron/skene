<?php

declare(strict_types=1);

namespace Skene\Example;

use Claviron\Bench\Symfony\BenchController;
use Symfony\Bundle\FrameworkBundle\FrameworkBundle;
use Symfony\Bundle\FrameworkBundle\Kernel\MicroKernelTrait;
use Symfony\Component\DependencyInjection\Loader\Configurator\ContainerConfigurator;
use Symfony\Component\HttpKernel\Kernel as BaseKernel;
use Symfony\Component\Routing\Loader\Configurator\RoutingConfigurator;

final class Kernel extends BaseKernel
{
    use MicroKernelTrait;

    public function getProjectDir(): string
    {
        return dirname(__DIR__);
    }

    public function registerBundles(): iterable
    {
        yield new FrameworkBundle();
    }

    private function configureContainer(ContainerConfigurator $container): void
    {
        $container->extension('framework', ['secret' => 'skene-example']);
        $container->services()->defaults()->autowire()->autoconfigure()
            ->set(Controller::class)->tag('controller.service_arguments');
        if (class_exists(BenchController::class)) {
            $container->services()->defaults()->autowire()->autoconfigure()
                ->set(BenchController::class)->tag('controller.service_arguments');
        }
    }

    private function configureRoutes(RoutingConfigurator $routes): void
    {
        $routes->import(__DIR__.'/Controller.php', 'attribute');
        if (class_exists(BenchController::class)) {
            $routes->import(__DIR__.'/../vendor/claviron/bench/src/Symfony/BenchController.php', 'attribute');
        }
    }
}
