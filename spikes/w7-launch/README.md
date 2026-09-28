# Spike W7: lançamento de processos

Descartável. Roda casos de lançamento com `std::process::Command` (sem shell) contra um executável de teste (`echo_args`) que grava seu diretório de trabalho e cada argumento.

```bash
cargo build && cargo run --bin spike-w7-launch
```

## Resultado (2026-09-28, Windows 11, Rust 1.93.1)

| Caso | Resultado |
|---|---|
| Executável em pasta com espaços e acento, argumentos hostis (`; calc.exe`, `& whoami`, `%PATH%`, `^`, aspas, tab, vazio) | **Sucesso:** os 10 argumentos chegaram idênticos, cada um como um argumento |
| Diretório de trabalho | Herdado do launcher por padrão; `current_dir` explícito funciona (usar a pasta do executável como padrão, como a política do núcleo faz) |
| Executável inexistente | `ErrorKind::NotFound` → mapeia para `LaunchError::NotFound` |
| `.bat` com argumentos | `simples`, com espaço, `& echo INJETADO`, aspas e `%PATH%` chegaram como texto literal, **sem injeção**. Argumento com quebra de linha: **recusado** (`InvalidInput: batch file arguments are invalid`) |
| Não bloquear o app | `spawn` retornou em ~6 ms com o filho vivo |

## Conclusões

- **Sucesso** para executáveis comuns: sem shell, sem injeção, com espaços e caracteres especiais.
- O Rust atual escapa argumentos de `.bat`, mas a documentação dele ainda avisa que `cmd.exe`/`.bat` decodificam de forma não padrão. **A política do catálogo continua conservadora:** `.bat`/`.cmd` só sem argumentos ([Q13](../../docs/OPEN-QUESTIONS.md#q13-bat-e-cmd-como-executável)). Este teste não prova que todo caso é seguro.
- Achado novo: um `.bat` gravado em UTF-8 com caminho com acento **não funciona** (o `cmd.exe` lê o arquivo na página de código OEM). É problema do `.bat` do usuário, não do app, mas vale uma dica na UI se o caminho tiver acento.
- Se o jogo lançado morre ou fica em segundo plano, o app não é afetado: o processo filho é independente.

## Não verificado

- Abrir `steam://` por `explorer.exe` (não abri para não iniciar jogo).
- Elevação (UAC) com `requires_elevation`: exige interação humana.
- Launchers que abrem outro processo e saem (o app só vê o primeiro).
