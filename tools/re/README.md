# Export headless do Ghidra

Estas ferramentas geram o inventário usado pelo lado sujo do processo clean-room.

## Pré-requisitos

- Ghidra 12.1.4 instalado em `C:\Tools\ghidra_12.1.4_PUBLIC` (ou informado com `-Ghidra`);
- JDK 21 configurado para o Ghidra;
- projeto `rekord_clone` existente na raiz do repositório;
- GUI do Ghidra fechada. A presença de `rekord_clone.lock*` aborta a execução com código 3.

## Uso

Na raiz do repositório, para gerar o inventário:

```powershell
powershell -File tools\re\export.ps1 -Mode inventory
```

Para descompilar uma lista de endereços, com um endereço por linha:

```powershell
powershell -File tools\re\export.ps1 -Mode decompile -List enderecos.txt
```

Os parâmetros opcionais `-Ghidra` e `-Out` alteram, respectivamente, a instalação do Ghidra e o diretório de saída. `-ProjectDir` aponta para outra pasta com `rekord_clone.gpr` (padrão: raiz do repositório). `inventory` e `decompile` abrem o projeto com `-readOnly` e sem nova análise.

O modo `rtti` (RecoverClassesFromRTTIScript) estourou a memória duas vezes neste binário numa máquina de 7,7 GB (2 GB e 5 GB de heap, 2026-10-23); o `vtables.tsv` substitui o uso dele. Se houver máquina com mais RAM, rode-o numa cópia do projeto, nunca no original (o script recusa a raiz do repo):

```powershell
robocopy rekord_clone.rep third_party\rekordbox\re\ghidra-rtti\rekord_clone.rep /E
Copy-Item rekord_clone.gpr third_party\rekordbox\re\ghidra-rtti\
powershell -File tools\re\export.ps1 -Mode rtti -ProjectDir third_party\rekordbox\re\ghidra-rtti -MaxMem 5G
powershell -File tools\re\export.ps1 -Mode inventory -ProjectDir third_party\rekordbox\re\ghidra-rtti
```

## Saída

Por padrão, os arquivos são escritos em `third_party/rekordbox/re/export/` como TSV UTF-8:

- `functions.tsv`: `address`, `name`, `namespace`, `size`, `callers`, `callees`;
- `classes.tsv`: `class`, `parent`, `functions` (a análise padrão quase não põe funções dentro das classes; use `vtables.tsv`);
- `vtables.tsv`: `class`, `vftable`, `slot`, `function` — métodos virtuais de cada classe, lidos das vftables rotuladas pela análise RTTI padrão;
- `strings.tsv`: `address`, `xrefs`, `value`;
- `imports.tsv`: `library`, `symbol`;
- `callgraph.tsv`: `caller`, `callee`;
- `decomp/<endereço>.c`: resultado de cada função solicitada ou uma única linha de falha;
- `decomp/_status.tsv`: `address`, `status`, onde o estado é `ok`, `timeout`, `erro` ou `nao-encontrada`.

Tabs, quebras de linha e retornos de carro em campos TSV são escapados. Uma falha ou timeout individual de descompilação não interrompe o lote.

**Saída é lado sujo: só analistas e auditores leem. O agent principal e implementadores Rust nunca abrem.**
