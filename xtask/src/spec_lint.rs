#[derive(Debug, PartialEq, Eq)]
pub enum Rule {
    NomeGhidra,
    Endereco,
    TipoDescompilado,
    BlocoC,
    SecaoEvidencia,
    TabelaNumericaLonga,
}

impl Rule {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NomeGhidra => "nome-ghidra",
            Self::Endereco => "endereco",
            Self::TipoDescompilado => "tipo-descompilado",
            Self::BlocoC => "bloco-c",
            Self::SecaoEvidencia => "secao-evidencia",
            Self::TabelaNumericaLonga => "tabela-numerica-longa",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Violation {
    pub line: usize,
    pub rule: Rule,
}

fn tokens(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| !(character.is_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
}

fn nome_ghidra(token: &str) -> bool {
    const PREFIXOS: [&str; 7] = [
        "FUN_", "DAT_", "LAB_", "PTR_", "thunk_", "switchD_", "caseD_",
    ];
    const VARIAVEIS: [&str; 10] = [
        "uVar", "iVar", "lVar", "bVar", "cVar", "pcVar", "pvVar", "local_", "param_", "in_",
    ];
    PREFIXOS.iter().any(|prefixo| token.starts_with(prefixo))
        || token.starts_with("extraout_")
            && token["extraout_".len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_hexdigit())
        || VARIAVEIS.iter().any(|prefixo| {
            token
                .strip_prefix(prefixo)
                .and_then(|restante| restante.chars().next())
                .is_some_and(|c| c.is_ascii_hexdigit())
        })
}

fn endereco(token: &str) -> bool {
    let hex = |texto: &str| texto.chars().all(|c| c.is_ascii_hexdigit());
    // Com 0x: 8+ dígitos. Sem prefixo (formato do export do Ghidra): 9–16
    // dígitos começando por número, para não pegar datas AAAAMMDD.
    token
        .strip_prefix("0x")
        .is_some_and(|restante| restante.len() >= 8 && hex(restante))
        || (9..=16).contains(&token.len())
            && token.starts_with(|c: char| c.is_ascii_digit())
            && hex(token)
}

fn tipo_descompilado(token: &str) -> bool {
    matches!(
        token,
        "undefined"
            | "undefined1"
            | "undefined2"
            | "undefined4"
            | "undefined8"
            | "__thiscall"
            | "__fastcall"
            | "__cdecl"
    ) || ["CONCAT", "SUB", "ZEXT", "SEXT"].iter().any(|prefixo| {
        token
            .strip_prefix(prefixo)
            .and_then(|restante| restante.chars().next())
            .is_some_and(|c| c.is_ascii_digit())
    })
}

fn cerca(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let marcador = if trimmed.starts_with("```") {
        "```"
    } else if trimmed.starts_with("~~~") {
        "~~~"
    } else {
        return None;
    };
    Some(trimmed[marcador.len()..].trim())
}

fn conta_numeros(line: &str) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut quantidade = 0;
    let mut indice = 0;
    while indice < chars.len() {
        if chars[indice].is_ascii_digit()
            && (indice == 0 || !(chars[indice - 1].is_alphanumeric() || chars[indice - 1] == '_'))
        {
            quantidade += 1;
            indice += 1;
            if chars[indice - 1] == '0'
                && indice < chars.len()
                && matches!(chars[indice], 'x' | 'X')
            {
                indice += 1;
                while indice < chars.len() && chars[indice].is_ascii_hexdigit() {
                    indice += 1;
                }
            } else {
                while indice < chars.len() && chars[indice].is_ascii_digit() {
                    indice += 1;
                }
                if indice + 1 < chars.len()
                    && chars[indice] == '.'
                    && chars[indice + 1].is_ascii_digit()
                {
                    indice += 1;
                    while indice < chars.len() && chars[indice].is_ascii_digit() {
                        indice += 1;
                    }
                }
                if indice + 1 < chars.len()
                    && matches!(chars[indice], 'e' | 'E')
                    && (chars[indice + 1].is_ascii_digit()
                        || indice + 2 < chars.len()
                            && matches!(chars[indice + 1], '+' | '-')
                            && chars[indice + 2].is_ascii_digit())
                {
                    indice += 1;
                    if indice < chars.len() && matches!(chars[indice], '+' | '-') {
                        indice += 1;
                    }
                    while indice < chars.len() && chars[indice].is_ascii_digit() {
                        indice += 1;
                    }
                }
            }
        } else {
            indice += 1;
        }
    }
    quantidade
}

fn aprovada(line: Option<&&str>) -> bool {
    line.is_some_and(|line| {
        let Some(data) = line
            .trim()
            .strip_prefix("<!-- tabela-aprovada: ")
            .and_then(|restante| restante.strip_suffix(" -->"))
        else {
            return false;
        };
        data.len() == 10
            && data.chars().enumerate().all(|(indice, c)| {
                matches!(indice, 4 | 7) && c == '-'
                    || !matches!(indice, 4 | 7) && c.is_ascii_digit()
            })
    })
}

pub fn lint_spec(text: &str) -> Vec<Violation> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    let mut bloco: Option<(usize, String, usize, bool)> = None;

    for (indice, line) in lines.iter().enumerate() {
        let numero = indice + 1;
        let mut adicionar = |rule| violations.push(Violation { line: numero, rule });
        let line_tokens: Vec<&str> = tokens(line).collect();
        if line_tokens.iter().any(|token| nome_ghidra(token)) {
            adicionar(Rule::NomeGhidra);
        }
        if line_tokens.iter().any(|token| endereco(token)) {
            adicionar(Rule::Endereco);
        }
        if line_tokens.iter().any(|token| tipo_descompilado(token)) {
            adicionar(Rule::TipoDescompilado);
        }
        let titulo = line.trim_start();
        let quantidade_hashes = titulo.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&quantidade_hashes)
            && titulo[quantidade_hashes..].starts_with(char::is_whitespace)
        {
            let conteudo = titulo[quantidade_hashes..].trim_start().to_lowercase();
            if conteudo.contains("evidência") || conteudo.contains("evidencia") {
                adicionar(Rule::SecaoEvidencia);
            }
        }

        if let Some((abertura, marcador, total, bloco_aprovado)) = bloco.as_mut() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(marcador.as_str()) {
                if *total > 16 && !*bloco_aprovado {
                    violations.push(Violation {
                        line: *abertura,
                        rule: Rule::TabelaNumericaLonga,
                    });
                }
                bloco = None;
            } else {
                *total += conta_numeros(line);
            }
            continue;
        }

        if let Some(linguagem) = cerca(line) {
            let marcador = if line.trim_start().starts_with("```") {
                "```"
            } else {
                "~~~"
            };
            let linguagem = linguagem.split_whitespace().next().unwrap_or("");
            if matches!(
                linguagem.to_ascii_lowercase().as_str(),
                "c" | "cpp" | "c++" | "h"
            ) {
                adicionar(Rule::BlocoC);
            }
            bloco = Some((
                numero,
                marcador.to_owned(),
                conta_numeros(line),
                aprovada(indice.checked_sub(1).and_then(|i| lines.get(i))),
            ));
        } else if conta_numeros(line) > 16
            && !aprovada(indice.checked_sub(1).and_then(|i| lines.get(i)))
        {
            adicionar(Rule::TabelaNumericaLonga);
        }
    }

    if let Some((abertura, _, total, bloco_aprovado)) = bloco {
        if total > 16 && !bloco_aprovado {
            violations.push(Violation {
                line: abertura,
                rule: Rule::TabelaNumericaLonga,
            });
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(text: &str) -> Vec<(usize, Rule)> {
        lint_spec(text)
            .into_iter()
            .map(|v| (v.line, v.rule))
            .collect()
    }

    #[test]
    fn texto_limpo_passa() {
        let t = "# 11 — ANLZ\n\nA tag `PQTZ` guarda o grid.\n\n| Campo | Bytes |\n| --- | --- |\n| bpm | 2 |\n";
        assert!(lint_spec(t).is_empty());
    }

    #[test]
    fn pega_nome_ghidra_em_codigo_inline() {
        assert_eq!(
            rules("linha\nchama `FUN_1400a3b0` antes\n"),
            vec![(2, Rule::NomeGhidra)]
        );
        assert_eq!(rules("| x | uVar3 |\n"), vec![(1, Rule::NomeGhidra)]);
        assert_eq!(rules("param_1 recebe\n"), vec![(1, Rule::NomeGhidra)]);
    }

    #[test]
    fn pega_endereco_longo_mas_nao_curto() {
        assert_eq!(rules("em 0x140001000\n"), vec![(1, Rule::Endereco)]);
        assert!(lint_spec("flag 0x40 e 0xFFFF\n").is_empty());
    }

    #[test]
    fn pega_endereco_sem_prefixo_mas_nao_data() {
        assert_eq!(rules("em 140001000\n"), vec![(1, Rule::Endereco)]);
        assert_eq!(rules("| 1400a3b0c |\n"), vec![(1, Rule::Endereco)]);
        assert!(lint_spec("versão 20261023 e 44100 Hz\n").is_empty());
    }

    #[test]
    fn pega_tipo_descompilado() {
        assert_eq!(
            rules("um undefined4 aqui\n"),
            vec![(1, Rule::TipoDescompilado)]
        );
        assert_eq!(rules("via CONCAT44\n"), vec![(1, Rule::TipoDescompilado)]);
    }

    #[test]
    fn pega_bloco_c_e_aceita_text() {
        assert_eq!(rules("a\n```cpp\nx\n```\n"), vec![(2, Rule::BlocoC)]);
        assert!(lint_spec("```text\nestado A -> B\n```\n").is_empty());
    }

    #[test]
    fn pega_secao_evidencia() {
        assert_eq!(
            rules("# X\n## Evidência\n"),
            vec![(2, Rule::SecaoEvidencia)]
        );
        assert_eq!(rules("### evidencia\n"), vec![(1, Rule::SecaoEvidencia)]);
    }

    #[test]
    fn tabela_numerica_longa_e_aprovacao() {
        let nums: Vec<String> = (0..17).map(|n| n.to_string()).collect();
        let bloco = format!("```text\n{}\n```\n", nums.join("\n"));
        assert_eq!(rules(&bloco), vec![(1, Rule::TabelaNumericaLonga)]);
        let aprovado = format!("<!-- tabela-aprovada: 2026-10-23 -->\n{bloco}");
        assert!(lint_spec(&aprovado).is_empty());
        let dezesseis: Vec<String> = (0..16).map(|n| n.to_string()).collect();
        assert!(lint_spec(&dezesseis.join(", ")).is_empty());
    }

    #[test]
    fn crlf_e_bom_mantem_numero_de_linha() {
        assert_eq!(
            rules("\u{feff}a\r\nb\r\nFUN_00401000\r\n"),
            vec![(3, Rule::NomeGhidra)]
        );
    }
}
