<?php

declare(strict_types=1);

namespace Skene\Example;

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
    }

    private function configureRoutes(RoutingConfigurator $routes): void
    {
        $routes->import(__DIR__.'/Controller.php', 'attribute');
    }
}
