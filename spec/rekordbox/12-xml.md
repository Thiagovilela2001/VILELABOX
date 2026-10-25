# 12 — Intercâmbio XML

Revisão v2, feita sobre a instalação 7.2.8 para Windows. Toda afirmação marcada "sem confirmar" é inferência; as demais foram observadas diretamente. A v2 incorpora a observação de uma exportação real da coleção ("Export Collection in xml format", 2026-10-24, interface em português, 3088 Registros, 10 Playlists), feita pelo usuário; afirmações confirmadas por ela estão marcadas "observado na exportação". O arquivo exportado fica fora do repositório, por conter dados pessoais.

## Escopo e fora do escopo

**Escopo.** O Documento de intercâmbio: o arquivo XML de coleção que o Núcleo lê como biblioteca importada e escreve como exportação da coleção, com raiz `DJ_PLAYLISTS`. Cobre estrutura, cada elemento e atributo, regras de leitura e de escrita e casos de teste para o leitor e o escritor do VILELABOX.

**Fora do escopo.** As listas internas do perfil (`masterPlaylists6.xml`, `automixPlaylist6.xml`) não são especificadas aqui; a biblioteca de outro programa de música importada por XML também fica fora. A cópia automática do XML para serviço de nuvem é classificada como fora do escopo (nuvem). O catálogo principal fica em 10-master-db.

## Glossário

Reaproveita os termos de 00-mapa (Núcleo, Dados de análise). Neste documento:

- **Documento de intercâmbio** é um arquivo XML com raiz `DJ_PLAYLISTS`;
- **Coleção** é o conjunto de faixas do documento, no elemento `COLLECTION`;
- **Registro de faixa** é um elemento `TRACK` filho de `COLLECTION`;
- **Referência de faixa** é um elemento `TRACK` filho de um `NODE` de playlist, que aponta para um Registro de faixa;
- **Trecho de grade** é um elemento `TEMPO`: um ponto da grade rítmica a partir do qual vale um andamento;
- **Grade variável** é um Registro de faixa com mais de um Trecho de grade;
- **Marca** é um elemento `POSITION_MARK`: ponto de disparo, memória, loop, fade ou carga;
- **Disparo rápido** é a Marca com `Num` de 0 em diante (botões A, B, C…);
- **Marca de memória** é a Marca com `Num` igual a -1;
- **Nó** é um elemento `NODE` da árvore de playlists; **Pasta** é o Nó de tipo 0 e **Playlist** o Nó de tipo 1;
- **Biblioteca importada** é a árvore que o Núcleo mostra a partir de um Documento de intercâmbio escolhido pelo usuário.

## Estrutura do documento

Hierarquia:

| elemento | pai | quantidade | conteúdo |
| --- | --- | --- | --- |
| `DJ_PLAYLISTS` | raiz | 1 | `PRODUCT`, `COLLECTION`, `PLAYLISTS`, nessa ordem |
| `PRODUCT` | `DJ_PLAYLISTS` | 1 | vazio |
| `COLLECTION` | `DJ_PLAYLISTS` | 1 | Registros de faixa |
| `TRACK` (Registro) | `COLLECTION` | 0 ou mais | `TEMPO` e depois `POSITION_MARK` |
| `TEMPO` | Registro de faixa | 0 ou mais | vazio |
| `POSITION_MARK` | Registro de faixa | 0 ou mais | vazio |
| `PLAYLISTS` | `DJ_PLAYLISTS` | 1 | um `NODE` raiz |
| `NODE` | `PLAYLISTS` ou `NODE` de Pasta | 1 raiz; 0 ou mais filhos | Pasta: `NODE`; Playlist: Referências de faixa |
| `TRACK` (Referência) | `NODE` de Playlist | 0 ou mais | vazio |

Na escrita, `TEMPO` vem antes de `POSITION_MARK` em todos os Registros e a ordem é `PRODUCT`, `COLLECTION`, `PLAYLISTS` (observado na exportação). Que a leitura exija essa ordem fica sem confirmar.

Codificação e cabeçalho:

- o documento começa com a declaração `<?xml version="1.0" encoding="UTF-8"?>`, seguida de uma linha em branco; todo texto é UTF-8;
- na escrita, os caracteres especiais em valores de atributo são substituídos por referências de entidade; `&amp;`, `&apos;`, `&quot;` e `&lt;` foram observados na exportação; `&gt;` é esperado pelo XML, sem confirmar;
- recuo de dois espaços por nível; atributos longos quebram a linha e continuam alinhados sob o primeiro atributo (observado na exportação). Recuo e quebras de linha não têm significado na leitura.

`DJ_PLAYLISTS`:

| atributo | tipo | obrigatório | significado |
| --- | --- | --- | --- |
| `Version` | texto | sim | versão do formato; o programa escreve `1.0.0` |

`PRODUCT`:

| atributo | tipo | significado |
| --- | --- | --- |
| `Name` | texto | nome do produto que gerou o arquivo; outros programas o exibem; o programa escreve `rekordbox` (observado na exportação) |
| `Version` | texto | versão do produto; para esta instalação, `7.2.8` (observado na exportação) |
| `Company` | texto | empresa; o programa escreve `AlphaTheta` |

`COLLECTION`:

| atributo | tipo | significado |
| --- | --- | --- |
| `Entries` | inteiro com sinal de 32 bits | quantidade de Registros de faixa |

## Faixa

Registro de faixa (`TRACK` em `COLLECTION`). Somente `Location` é declarada essencial pela documentação pública; `TrackID` é necessário sempre que alguma Playlist usa `KeyType` 0. Todos os valores são texto no XML; a coluna "tipo" diz como interpretá-lo.

| atributo | tipo | unidade e formato | faixa válida | significado |
| --- | --- | --- | --- | --- |
| `TrackID` | inteiro (texto) | — | único no documento | identificador do Registro, usado pelas Referências com `KeyType` 0 |
| `Name` | texto | — | — | título |
| `Artist` | texto | — | — | artista |
| `Composer` | texto | — | — | compositor ou produtor |
| `Album` | texto | — | — | álbum |
| `Grouping` | texto | — | — | agrupamento |
| `Genre` | texto | — | — | gênero |
| `Kind` | texto | ver tabela de tipos | — | tipo do arquivo de áudio |
| `Size` | inteiro de 64 bits | bytes | ≥ 0 | tamanho do arquivo |
| `TotalTime` | inteiro | segundos, sem casas decimais | ≥ 0 | duração |
| `DiscNumber` | inteiro de 32 bits | — | ≥ 0 | número do disco |
| `TrackNumber` | inteiro de 32 bits | — | ≥ 0 | número da faixa no álbum |
| `Year` | inteiro de 32 bits | ano | — | ano |
| `AverageBpm` | decimal | batidas por minuto, duas casas (`128.00`) | > 0; `0.00` quando sem grade (observado na exportação) | andamento médio |
| `DateModified` | data | `aaaa-mm-dd` | — | data de modificação; não observada no binário, sem confirmar que o programa a escreva |
| `DateAdded` | data | `aaaa-mm-dd` | — | data de inclusão na coleção |
| `BitRate` | inteiro de 32 bits | kbit/s | ≥ 0 | taxa de bits |
| `SampleRate` | decimal | Hz | ex.: 44100, 48000 | taxa de amostragem |
| `Comments` | texto | — | — | comentário |
| `PlayCount` | inteiro de 32 bits | — | ≥ 0 | contagem de reprodução |
| `LastPlayed` | data | `aaaa-mm-dd` | — | última reprodução; não observada no binário, sem confirmar que o programa a escreva |
| `Rating` | inteiro de 32 bits | escala 0–255 | ver tabela | classificação por estrelas |
| `Location` | URL de arquivo | ver abaixo | — | localização do arquivo, incluindo o nome |
| `Remixer` | texto | — | — | remixador |
| `Tonality` | texto | nome do tom: tônica e `m` para menor (`Am`, `F#m`, `Abm`, `C`) | vazio quando sem tom | tom; a exportação usou nomes, não notação alfanumérica, e trouxe tanto bemóis quanto sustenidos (`Abm` e `G#m`) — se a grafia depende de preferência ou da origem do valor fica sem confirmar |
| `Label` | texto | — | — | gravadora |
| `Mix` | texto | — | — | nome da mixagem |
| `Colour` | texto | `0xRRGGBB` | 8 valores, ver tabela | cor de agrupamento da faixa |

`Name`, `Kind`, `Size`, `Year` e `Mix` são escritos em todos os Registros, mesmo vazios (`Mix=""`, `Year="0"`). `DateModified` e `LastPlayed` não são escritos (observado na exportação).

Ordem de escrita dos atributos, idêntica em todos os Registros da exportação: `TrackID`, `Name`, `Artist`, `Composer`, `Album`, `Grouping`, `Genre`, `Kind`, `Size`, `TotalTime`, `DiscNumber`, `TrackNumber`, `Year`, `AverageBpm`, `DateAdded`, `BitRate`, `SampleRate`, `Comments`, `PlayCount`, `Rating`, `Location`, `Remixer`, `Tonality`, `Label`, `Mix`, `Colour`. Atributos de texto vazios são escritos como `""`, não omitidos; `Colour` é a exceção e é omitido quando a faixa não tem cor (nenhum Registro da exportação tinha cor).

`Rating`:

| estrelas | valor |
| --- | --- |
| 0 | 0 |
| 1 | 51 |
| 2 | 102 |
| 3 | 153 |
| 4 | 204 |
| 5 | 255 |

Valores intermediários: arredondar para a estrela mais próxima, sem confirmar.

`Kind` é **traduzido para o idioma da interface**. Com interface em português a exportação escreveu `Ficheiro MP3`, `Ficheiro WAV`, `Ficheiro FLAC` e `Formato Desconhecido` (este para um `.mpeg`). Os textos em inglês do binário são: `MP3 File`, `AAC File`, `MP4 File`, `M4A File`, `FLAC File`, `ALAC File`, `WAV File`, `AIFF File`, `Unknown Format`. Consequência: o leitor não deve deduzir o formato de `Kind`; usar a extensão de `Location`. O escritor do VILELABOX escreve os textos em inglês (`ADAPTAÇÃO`).

`Colour` (lista completa dos valores escritos pelo programa; os nomes vêm da documentação pública):

| valor | nome público |
| --- | --- |
| `0xFF007F` | Rose |
| `0xFF0000` | Red |
| `0xFFA500` | Orange |
| `0xFFFF00` | Lemon |
| `0x00FF00` | Green |
| `0x25FDE9` | Turquoise |
| `0x0000FF` | Blue |
| `0x660099` | Violet |

Faixa sem cor: o atributo é omitido (observado na exportação).

`Rating`: a exportação só trouxe 0 e 51, coerentes com a tabela acima.

### Localização

- Forma: prefixo `file://localhost/` seguido do caminho absoluto com `/` como separador. No Windows, a unidade vem logo após o prefixo e o `:` fica literal: `file://localhost/C:/Musica/faixa.mp3`. Em sistemas com raiz `/`, o caminho começa logo após o prefixo: `file://localhost/Users/dj/faixa.mp3`, sem confirmar a forma escrita pelo programa no macOS.
- Codificação: o caminho é convertido para bytes UTF-8 e cada byte fora do conjunto preservado é escrito como `%` seguido de dois dígitos hexadecimais. Observado na exportação: ficam literais as letras ASCII, os dígitos e `! # $ ( ) + , - . / : _`; são codificados o espaço (`%20`), `&` (`%26`), `'` (`%27`), `[` (`%5b`), `]` (`%5d`) e todo byte não ASCII. **`#` fica literal**, ao contrário do esperado para URL. Os caracteres `* ; = ? @ ~` não apareceram em nenhum caminho; o tratamento deles fica sem confirmar.
- Caixa dos dígitos: minúscula, sem exceção (`%c3%a9`, 1862 ocorrências, nenhuma maiúscula; observado na exportação). Na leitura, aceitar as duas caixas.
- Normalização Unicode: o caminho é escrito com os bytes do sistema de arquivos, sem normalizar; um nome em forma decomposta saiu como `E%cc%81` (observado na exportação). O leitor compara caminhos sem normalizar.
- Na leitura, aceitar também `file:///` (sem `localhost`), sem confirmar que o programa aceite.

## Grade

Trecho de grade (`TEMPO`). Um Registro pode ter vários; a Grade variável é a sequência deles em ordem crescente de `Inizio`, e cada um vale até o próximo.

| atributo | tipo | unidade e formato | faixa válida | significado |
| --- | --- | --- | --- | --- |
| `Inizio` | decimal | segundos, três casas (`0.025`) | ≥ 0; menor que a duração, sem confirmar | posição da batida em que o trecho começa |
| `Bpm` | decimal | batidas por minuto, duas casas (`128.00`) | > 0 | andamento a partir de `Inizio` |
| `Metro` | texto | `n/d` | ex.: `4/4`, `3/4`, `7/8` | compasso |
| `Battito` | inteiro | — | 1 a n do compasso (1–4 em `4/4`) | número da batida no compasso que cai em `Inizio`; 1 é o tempo forte |

Regras:

- grade constante: um único `TEMPO`; as batidas seguintes ficam em `Inizio + k × 60 / Bpm`;
- o programa escreve um `TEMPO` por trecho da grade, não por batida: a exportação teve 8320 `TEMPO` em 2602 Registros com grade (média de 3,2). Em 510 Registros há vários `TEMPO` com o mesmo `Bpm`, ou seja, um trecho novo não implica mudança de andamento (por exemplo, grade reancorada); só 1 Registro tinha andamentos distintos;
- `Battito` do primeiro `TEMPO` de um trecho pode ser qualquer batida do compasso (1 a 4 observados), não só 1;
- o leitor deve aceitar também um `TEMPO` por batida, que outros programas produzem;
- sem confirmar: a escrita da grade é controlada por uma preferência própria do programa, e, desligada, omite todos os `TEMPO`;
- `Metro` diferente de `4/4`: o programa lê o atributo; o comportamento da grade com outros compassos fica sem confirmar.

## Marcas

Marca (`POSITION_MARK`).

| atributo | tipo | unidade e formato | faixa válida | significado |
| --- | --- | --- | --- | --- |
| `Name` | texto | — | pode ser vazio | nome da Marca |
| `Type` | inteiro | ver tabela | 0–4 | tipo |
| `Start` | decimal | segundos, três casas | ≥ 0 | início |
| `End` | decimal | segundos, três casas | > `Start` | fim; presente só em loop (observado na exportação) |
| `Num` | inteiro | — | -1, ou 0 em diante | -1 é Marca de memória; 0, 1, 2… são os Disparos rápidos A, B, C… |
| `Red` | inteiro | 0–255 | — | componente vermelho da cor |
| `Green` | inteiro | 0–255 | — | componente verde da cor |
| `Blue` | inteiro | 0–255 | — | componente azul da cor |

`Type`:

| valor | significado |
| --- | --- |
| 0 | ponto (cue) |
| 1 | início de fade (fade-in) |
| 2 | fim de fade (fade-out) |
| 3 | ponto de carga |
| 4 | loop |

Regras:

- `Red`, `Green` e `Blue` vêm juntos. Sem os três atributos, a Marca usa a cor padrão do programa, sem confirmar;
- quantidade de Disparos rápidos: a exportação escreveu `Num` até 7 (A–H); valor acima de 7 na leitura fica sem confirmar;
- ordem dos atributos escritos: `Name`, `Type`, `Start`, `End`, `Num`, `Red`, `Green`, `Blue`; `End` só em loop (observado na exportação);
- Marcas de memória (`Num="-1"`) e loops de memória são escritos **sem** `Red`, `Green` e `Blue`; Disparos rápidos sempre com eles (observado na exportação, numa faixa com um ponto e um loop de memória criados para o teste);
- loop de memória: `Type="4"`, `Start` e `End` com três casas; o loop de 8 batidas a 130 BPM saiu como `Start="0.050" End="3.743"` (observado na exportação);
- ordem das Marcas no Registro: os Disparos rápidos primeiro, **fora da ordem de `Num`** (saiu 2, 0, 1, 7), depois as Marcas de memória, com o loop antes do ponto na mesma posição. O leitor não pode depender da ordem; a regra de ordenação do escritor fica sem confirmar;
- dois Disparos rápidos com o mesmo `Num` no mesmo Registro: o comportamento fica sem confirmar; o escritor do VILELABOX não deve produzir isso;
- `Start` e `End` são escritos com três casas decimais, isto é, resolução de 1 ms.

## Playlists e pastas

Nó (`NODE`).

| atributo | tipo | presença | significado |
| --- | --- | --- | --- |
| `Type` | inteiro | sempre | 0 = Pasta, 1 = Playlist |
| `Name` | texto | sempre | nome; o Nó raiz chama-se `ROOT` |
| `Count` | inteiro de 32 bits | Pasta | quantidade de `NODE` filhos |
| `Entries` | inteiro de 32 bits | Playlist | quantidade de Referências de faixa |
| `KeyType` | inteiro | Playlist | 0 = Referência por `TrackID`; 1 = Referência por `Location` |

Referência de faixa (`TRACK` em Playlist):

| atributo | tipo | significado |
| --- | --- | --- |
| `Key` | texto | com `KeyType` 0, o `TrackID` do Registro; com `KeyType` 1, a `Location` do Registro, codificada como na Coleção |

Regras:

- o Nó raiz é uma Pasta de nome `ROOT`, filha única de `PLAYLISTS`; o programa não mostra a raiz como Pasta, sem confirmar;
- a ordem dos `NODE` filhos é a ordem de exibição e a ordem das Referências é a ordem da Playlist, sem confirmar;
- a mesma faixa pode aparecer mais de uma vez na mesma Playlist, sem confirmar;
- profundidade de Pastas: sem limite conhecido, sem confirmar;
- o programa escreve `KeyType` 0 em todas as Playlists (observado na exportação);
- ordem dos atributos escritos: Pasta `Type`, `Name`, `Count`; Playlist `Name`, `Type`, `KeyType`, `Entries` (observado na exportação);
- Referências de faixa trazem só `Key`; na exportação nenhuma era órfã e não havia `TrackID` duplicado;
- Playlists de sistema também são exportadas (uma Playlist vazia criada pelo programa apareceu na exportação, com nome traduzido).

## Regras de importação

O programa mostra o Documento de intercâmbio como Biblioteca importada na árvore do navegador, sob os rótulos "rekordbox xml" e "Imported Library". O caminho do arquivo é escolhido nas preferências ("Select an XML file") e guardado nelas; a árvore pode ser recarregada pelo comando "Reload XML library". Durante a leitura aparecem as mensagens "Loading XML library" e "Preparing...".

O que é lido (observado como lido pelo programa):

- `COLLECTION`, `PLAYLISTS`, `PRODUCT` com `Version`;
- no Registro de faixa: `TrackID`, `Artist`, `Composer`, `Album`, `Genre`, `TotalTime`, `DiscNumber`, `TrackNumber`, `AverageBpm`, `DateAdded`, `BitRate`, `SampleRate`, `Comments`, `PlayCount`, `Rating`, `Location`, `Remixer`, `Tonality`, `Label`, `Colour`, além, sem confirmar, de `Name`, `Kind`, `Size`, `Year` e `Mix`;
- `TEMPO` com `Inizio`, `Bpm` (sem confirmar), `Metro` e `Battito`;
- `POSITION_MARK` com `Start`, `Green` e, sem confirmar, `Name`, `Type`, `End`, `Num`, `Red` e `Blue`;
- `NODE` com `KeyType`, `Entries` e `Count`, e as Referências de faixa.

O que é ignorado ou fica de fora:

- `Grouping` não aparece entre os atributos lidos do Registro, sem confirmar que seja ignorado;
- `DateModified` e `LastPlayed` não aparecem no binário, sem confirmar que sejam ignorados;
- atributos e elementos desconhecidos são ignorados, sem confirmar.

Casamento e conflitos:

- a Biblioteca importada é exibida à parte; as faixas só entram no catálogo principal quando o usuário as importa da árvore, sem confirmar;
- ao importar, a faixa é casada com o catálogo pelo caminho do arquivo, comparado sem diferenciar maiúsculas e minúsculas, sem confirmar;
- se a faixa já existe, quais campos do XML substituem os do catálogo (grade, Marcas, metadados) fica sem confirmar;
- faixa cujo arquivo não existe: é mostrada, mas não reproduzível, sem confirmar;
- Referência cujo `Key` não corresponde a nenhum Registro: é descartada, sem confirmar.

Erros: se o arquivo não pode ser lido ou interpretado, o programa mostra "The rekordbox xml file specified is unable to be read properly." e, sem confirmar, não monta a árvore.

## Regras de exportação

O programa exporta a coleção pelo comando "Export Collection in xml format".

- Escreve a declaração XML com `UTF-8`, `DJ_PLAYLISTS Version="1.0.0"`, `PRODUCT` com `Version`, `Company="AlphaTheta"` e, sem confirmar, `Name`, `COLLECTION` com `Entries`, os Registros com seus `TEMPO` e `POSITION_MARK`, e `PLAYLISTS` com a árvore.
- Arredondamentos: `AverageBpm` e `Bpm` com duas casas decimais; `Inizio`, `Start` e `End` com três casas decimais; `TotalTime` em segundos inteiros. Se o arredondamento é para o mais próximo ou por truncamento fica sem confirmar.
- `Colour` é escrito só com um dos 8 valores da tabela.
- Ordem dos atributos do Registro: a da seção Faixa. Ordem dos Registros na Coleção: não é crescente por `TrackID` (observado na exportação); provavelmente a ordem do catálogo, sem confirmar.
- `TrackID` do Registro é o identificador da faixa no catálogo, sem confirmar.

## Casos de teste

Origem de todos os casos: construídos a partir do formato público e da ponte já existente do VILELABOX, não produzidos pelo programa real. Cada um deve ser lido pelo leitor do VILELABOX com o resultado indicado; o escritor deve produzir XML equivalente (mesmos elementos, atributos e valores).

### Documento mínimo válido

Entrada:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<DJ_PLAYLISTS Version="1.0.0">
  <PRODUCT Name="rekordbox" Version="7.2.8" Company="AlphaTheta"/>
  <COLLECTION Entries="0"/>
  <PLAYLISTS>
    <NODE Type="0" Name="ROOT" Count="0"/>
  </PLAYLISTS>
</DJ_PLAYLISTS>
```

Saída esperada: Coleção vazia, árvore com só a raiz, nenhum erro.

### Faixa com grade variável

```xml
<TRACK TrackID="1" Name="Faixa" Location="file://localhost/C:/Musica/a.mp3"
       TotalTime="300" AverageBpm="125.00">
  <TEMPO Inizio="0.050" Bpm="120.00" Metro="4/4" Battito="1"/>
  <TEMPO Inizio="60.050" Bpm="130.00" Metro="4/4" Battito="1"/>
</TRACK>
```

Saída esperada: duas seções de grade; batidas a cada 0,5 s de 0,050 s até 60,050 s, e a cada 60/130 s depois disso; tempo forte em 0,050 s e em 60,050 s.

### Disparo rápido A com cor

```xml
<POSITION_MARK Name="Drop" Type="0" Start="32.000" Num="0"
               Red="255" Green="55" Blue="111"/>
```

Saída esperada: Disparo rápido A em 32,000 s, nome "Drop", cor RGB (255, 55, 111).

### Marca de memória

```xml
<POSITION_MARK Name="" Type="0" Start="16.500" Num="-1"/>
```

Saída esperada: Marca de memória em 16,500 s, sem nome, cor padrão.

### Loop

```xml
<POSITION_MARK Name="Loop" Type="4" Start="64.000" End="72.000" Num="-1"/>
```

Saída esperada: loop de memória de 64,000 s a 72,000 s (8 s). Com `Num="1"`, seria um loop no Disparo rápido B.

### Playlist aninhada

```xml
<NODE Type="0" Name="ROOT" Count="1">
  <NODE Type="0" Name="Festas" Count="1">
    <NODE Name="Abertura" Type="1" KeyType="0" Entries="2">
      <TRACK Key="1"/>
      <TRACK Key="2"/>
    </NODE>
  </NODE>
</NODE>
```

Saída esperada: Pasta "Festas" contendo a Playlist "Abertura" com os Registros 1 e 2, nessa ordem. A mesma Playlist com `KeyType="1"` usaria como `Key` a `Location` de cada Registro.

### Caracteres especiais no caminho

Caminho no disco: `C:\Músicas\Rock & Roll\faixa #1.mp3`.

```xml
<TRACK TrackID="3" Name="Rock &amp; Roll"
       Location="file://localhost/C:/M%c3%basicas/Rock%20%26%20Roll/faixa%20#1.mp3"/>
```

Saída esperada: o leitor recupera exatamente o caminho original; `ú` vira os bytes UTF-8 `%c3%ba` (o leitor aceita também `%C3%BA`); `#` fica literal, como na exportação real; o `&` do nome é escrito como `&amp;` no atributo `Name`. O leitor também aceita `%23`.

## Perguntas abertas

Respondidas na v2 pela exportação real: valores de `PRODUCT`; presença e posição de `Name`, `Kind`, `Size`, `Year` e `Mix`; ausência de `DateModified` e `LastPlayed`; caixa dos dígitos de `Location`; um `TEMPO` por trecho e não por batida; `Tonality` como nome de tom; escrita de Marcas de memória e loops. Duas exportações seguidas sem mudança na coleção saíram idênticas byte a byte: a escrita é determinística.

Ainda abertas:

- Como os caracteres `* ; = ? @ ~` são tratados em `Location`?
- A grafia de `Tonality` com bemol ou sustenido depende de preferência ou da origem do valor?
- Valores de `Colour` escritos: confirmar com uma faixa colorida.
- Na importação para o catálogo, quais campos do XML sobrescrevem os de uma faixa já existente?
- Como são tratadas Referências órfãs e Registros com `TrackID` duplicado na leitura?
