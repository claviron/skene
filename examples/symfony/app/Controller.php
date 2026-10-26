<?php

declare(strict_types=1);

namespace Skene\Example;

use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\HttpKernel\KernelInterface;
use Symfony\Component\Routing\Attribute\Route;

final class Controller
{
    private int $requests = 0;
    private readonly string $boot;

    public function __construct()
    {
        $this->boot = bin2hex(random_bytes(8));
    }

    #[Route('/', methods: ['GET'])]
    public function index(): Response
    {
        return new Response("Symfony on Skene\n", headers: ['Content-Type' => 'text/plain']);
    }

    #[Route('/state', methods: ['GET'])]
    public function state(KernelInterface $kernel): JsonResponse
    {
        return new JsonResponse([
            'pid' => getmypid(),
            'boot_id' => $this->boot,
            'request_count' => ++$this->requests,
            'environment' => $kernel->getEnvironment(),
            'debug' => $kernel->isDebug(),
        ]);
    }

    #[Route('/echo', methods: ['POST'])]
    public function echo(Request $request): Response
    {
        return new Response($request->getContent(), headers: ['Content-Type' => 'application/octet-stream']);
    }
}
