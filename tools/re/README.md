# Export headless do Ghidra

Estas ferramentas geram inventários e trechos decompilados para o port em Rust.
Nesta fase, quem investiga e quem implementa podem lê-los junto com as
evidências do REA. A separação de papéis *clean-room* descrita nos prompts
`analista.md` e `auditor.md` fica para depois da disponibilização pública do app.

## Pré-requisitos

- Ghidra 12.2 instalado em `C:\Users\Thiago\Downloads\ghidra_12.2_PUBLIC` (ou informado com `-Ghidra`);
- JDK 21 ou mais novo configurado para o Ghidra (testado com JDK 25);
- projeto `rekord_clone` existente na raiz do repositório (ver "Criar o projeto" abaixo);
- GUI do Ghidra fechada. A presença de `rekord_clone.lock*` aborta a execução com código 3.

## Criar o projeto

A análise padrão estoura o heap no analyzer "Create Address Tables"
(`OutOfMemoryError` em `AddressTableAnalyzer`). O prescript
`DisableAddressTables.java` o desliga. Com 10 GB de heap, a análise levou 50 min
numa máquina de 16 GB e 28 núcleos (2026-11-13):

```powershell
$env:GHIDRA_HEADLESS_MAXMEM = "10G"
& C:\Users\Thiago\Downloads\ghidra_12.2_PUBLIC\support\analyzeHeadless.bat $PWD rekord_clone `
    -import "C:\Program Files\rekordbox\rekordbox 7.2.19\rekordbox.exe" `
    -loader PeLoader -loader-loadLibraries false `
    -scriptPath tools\re -preScript DisableAddressTables.java
```

Binários menores entram no mesmo projeto sem o prescript. O `NFSDaemon.dll`
(193 KB) foi importado com 4 GB de heap em 19 s (2026-11-14). `Demucs3.dll`
(419 KB, 62 s) e `Spleeter.dll` (372 KB, 27 s) entraram no mesmo comando,
com os dois caminhos depois de `-import`:

```powershell
$env:GHIDRA_HEADLESS_MAXMEM = "4G"
& C:\Users\Thiago\Downloads\ghidra_12.2_PUBLIC\support\analyzeHeadless.bat $PWD rekord_clone `
    -import "C:\Program Files\rekordbox\rekordbox 7.2.19\NFSDaemon.dll" `
    -loader PeLoader -loader-loadLibraries false
```

`RefsTo.java` lista quem referencia cada endereço de uma lista (string,
constante ou função), com a função que contém cada referência:

```powershell
& C:\Users\Thiago\Downloads\ghidra_12.2_PUBLIC\support\analyzeHeadless.bat $PWD rekord_clone `
    -process rekordbox.exe -noanalysis -readOnly -scriptPath tools\re `
    -postScript RefsTo.java saida.tsv enderecos.txt
```

`DisasmList.java` desmonta as funções de uma lista em `disasm/<endereço>.asm`,
com o valor (`f64`, `f32`, `i32`) de cada constante lida da memória. Ele
recebe os mesmos argumentos do `DecompileList.java`:

```powershell
& C:\Users\Thiago\Downloads\ghidra_12.2_PUBLIC\support\analyzeHeadless.bat $PWD rekord_clone `
    -process rekordbox.exe -noanalysis -readOnly -scriptPath tools\re `
    -postScript DisasmList.java saida enderecos.txt
```

O REA não abre um projeto Ghidra existente no Windows (`REA_GHIDRA_PROJECT_PATH`
retorna `unsupported_host`). Por isso, consulte este projeto com o `export.ps1`.

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

**Saída privada:** não versionar exports nem trechos decompilados. Na fase atual,
analistas e implementadores podem consultá-los para traduzir a lógica para Rust
e rastrear cada regra até a evidência.
