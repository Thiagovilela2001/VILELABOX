//! Codificação de `Location` (12-xml, seção Localização).
//!
//! Observado na exportação real: prefixo `file://localhost/`, bytes UTF-8 fora
//! do conjunto literal viram `%xx` com dígitos minúsculos, `#` fica literal e o
//! caminho não é normalizado.

const PREFIXO: &str = "file://localhost/";

/// Caracteres ASCII escritos sem codificar, além de letras e dígitos.
/// `* ; = ? @ ~` não foram observados; ficam codificados (ADAPTAÇÃO).
fn literal(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$()+,-./:_".contains(&b)
}

/// Caminho com `/` (ex.: `C:/Musica/a.mp3` ou `/Users/dj/a.mp3`) → `Location`.
pub fn encode(caminho: &str) -> String {
    let mut out = String::from(PREFIXO);
    let resto = caminho.strip_prefix('/').unwrap_or(caminho);
    for &b in resto.as_bytes() {
        if literal(b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02x}"));
        }
    }
    out
}

/// `Location` → caminho com `/`. Aceita `file://localhost/` e `file:///`, e
/// dígitos hexadecimais nas duas caixas. `None` se a codificação for inválida.
pub fn decode(location: &str) -> Option<String> {
    let resto = location
        .strip_prefix(PREFIXO)
        .or_else(|| location.strip_prefix("file:///"))?;
    let mut bytes = Vec::with_capacity(resto.len());
    let mut it = resto.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let hex = [it.next()?, it.next()?];
            bytes.push(u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?);
        } else {
            bytes.push(b);
        }
    }
    let caminho = String::from_utf8(bytes).ok()?;
    // Unidade do Windows fica logo após o prefixo; caminhos com raiz `/` voltam com ela.
    let windows = caminho.as_bytes().get(1) == Some(&b':');
    Some(if windows {
        caminho
    } else {
        format!("/{caminho}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caso_da_spec_ida_e_volta() {
        let caminho = "C:/Músicas/Rock & Roll/faixa #1.mp3";
        let loc = encode(caminho);
        assert_eq!(
            loc,
            "file://localhost/C:/M%c3%basicas/Rock%20%26%20Roll/faixa%20#1.mp3"
        );
        assert_eq!(decode(&loc).as_deref(), Some(caminho));
    }

    #[test]
    fn aceita_maiusculas_file_triplo_e_raiz_unix() {
        assert_eq!(
            decode("file:///C:/M%C3%BAsica/a%5bb%5d.mp3").as_deref(),
            Some("C:/Música/a[b].mp3")
        );
        let loc = encode("/Users/dj/a'b.mp3");
        assert_eq!(loc, "file://localhost/Users/dj/a%27b.mp3");
        assert_eq!(decode(&loc).as_deref(), Some("/Users/dj/a'b.mp3"));
    }

    #[test]
    fn invalidos() {
        assert_eq!(decode("C:/sem/prefixo.mp3"), None);
        assert_eq!(decode("file://localhost/a%2"), None);
        assert_eq!(decode("file://localhost/%ff"), None);
    }

    #[test]
    fn nao_normaliza_unicode() {
        // "É" decomposto (E + acento combinante), como na exportação real
        let caminho = "C:/NE\u{301}CTAR.mpeg";
        assert_eq!(encode(caminho), "file://localhost/C:/NE%cc%81CTAR.mpeg");
    }
}
