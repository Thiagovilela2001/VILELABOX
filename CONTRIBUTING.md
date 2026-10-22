# Como contribuir com o VILELABOX

O VILELABOX é um fork experimental do Mixxx focado em recursos inspirados no rekordbox.
Contribuições da comunidade são bem-vindas, desde que sigam as diretrizes abaixo.

## Onde relatar problemas ou propor mudanças

Abra uma issue em <https://github.com/Thiagovilela2001/VILELABOX/issues>.
**Não abra chamados nem leve dúvidas do VILELABOX para os canais oficiais do Mixxx** (fóruns, Zulip ou repositório oficial).

Ao relatar um problema, inclua:
- O que aconteceu e o comportamento esperado.
- A skin utilizada e os passos detalhados para reproduzir.
- A saída de `tools/verify-mod.sh`, no caso de erros de compilação ou de testes.

## Diretrizes de código

1. **Mudanças pequenas e focadas:** cada alteração deve cobrir um conjunto reduzido de arquivos com propósito claro.
2. **Verificação obrigatória (`tools/verify-mod.sh`):** toda mudança deve passar em `tools/verify-mod.sh` antes de ser aceita.
3. **Regra de terceiros do rekordbox:** nunca copie arte, fontes, modelos, mapeamentos ou binários do instalador do rekordbox para o repositório. O hook local `pre-push` e o script `tools/check-third-party-assets.sh` barram arquivos idênticos aos da instalação de referência.
4. **Preservação de licença:** o código permanece sob GPL-2.0-or-later. Não altere cabeçalhos de licença do Mixxx nem modifique `lib/`, `src/controllers/` ou `res/controllers/` sem discussão prévia.

As diretrizes originais do Mixxx estão preservadas em [CONTRIBUTING.mixxx.md](CONTRIBUTING.mixxx.md).
