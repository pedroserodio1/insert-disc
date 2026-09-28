# ADR-0010: Windows primeiro; núcleo portátil

**Status:** aceito

## Contexto
O autor usa Windows e não tem Linux instalado. O suporte a Linux é desejado, mas ainda não confirmado.

## Decisão
- Fase 1: Windows. Linux (incluindo gamescope) numa fase posterior, a confirmar.
- O núcleo (catálogo, máquina de estados, lançador-política, formato do disco) não usa APIs de SO.
- Tudo o que é específico de plataforma fica atrás de `DriveBackend` e `SystemIntegration`.

> Nota: a arquitetura acrescentou uma terceira interface, `GamepadSource`, para a entrada de controle, que depende do spike W1 ([ARCHITECTURE](../ARCHITECTURE.md)). Ela segue a mesma regra.

## Consequências
- Arquitetura em [ARCHITECTURE](../ARCHITECTURE.md); matriz em [PLATFORMS](../PLATFORMS.md).
- O `FakeIsoDrive` também serve para testar o núcleo em qualquer SO.
