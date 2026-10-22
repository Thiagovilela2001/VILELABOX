#!/usr/bin/env python3
"""Script de equivalência entre tools/rekordbox/sync.py e vilelabox-rekordbox (Rust).

Normaliza a estrutura XML (atributos ordenados e espaços normalizados; filhos na ordem do documento)
e valida a equivalência estrita de conteúdo.

Uso:
    python3 rust/vilelabox-rekordbox/tests/compare_with_python.py [xml_python] [xml_rust]
"""

import sys
import xml.etree.ElementTree as ET
from pathlib import Path


def canonicalize_element(elem: ET.Element) -> ET.Element:
    """Cria cópia canônica do elemento com atributos ordenados e filhos ordenados."""
    # Cria novo elemento com tag
    canonical = ET.Element(elem.tag)
    # Ordena atributos
    sorted_attrs = sorted((k, v.strip()) for k, v in elem.attrib.items())
    for k, v in sorted_attrs:
        canonical.set(k, v)

    # Texto
    if elem.text and elem.text.strip():
        canonical.text = elem.text.strip()

    # Filhos na ordem do documento: a ordem das faixas nas playlists faz parte
    # do resultado e não pode ser escondida por uma ordenação.
    child_canonicals = [canonicalize_element(c) for c in elem]

    for c in child_canonicals:
        canonical.append(c)

    return canonical


def elements_are_equal(e1: ET.Element, e2: ET.Element) -> tuple[bool, str]:
    if e1.tag != e2.tag:
        return False, f"Tag divergente: {e1.tag} != {e2.tag}"

    if sorted(e1.attrib.items()) != sorted(e2.attrib.items()):
        return False, f"Atributos divergentes em <{e1.tag}>:\n  E1: {sorted(e1.attrib.items())}\n  E2: {sorted(e2.attrib.items())}"

    if len(e1) != len(e2):
        return False, f"Quantidade de filhos divergente em <{e1.tag}>: {len(e1)} != {len(e2)}"

    for idx, (c1, c2) in enumerate(zip(e1, e2)):
        eq, reason = elements_are_equal(c1, c2)
        if not eq:
            return False, f"Filho [{idx}] de <{e1.tag}> divergente -> {reason}"

    return True, "OK"


def compare_xml_files(path_py: Path, path_rs: Path) -> bool:
    if not path_py.is_file():
        print(f"Arquivo XML Python não encontrado: {path_py}", file=sys.stderr)
        return False
    if not path_rs.is_file():
        print(f"Arquivo XML Rust não encontrado: {path_rs}", file=sys.stderr)
        return False

    tree_py = ET.parse(path_py)
    tree_rs = ET.parse(path_rs)

    norm_py = canonicalize_element(tree_py.getroot())
    norm_rs = canonicalize_element(tree_rs.getroot())

    equal, reason = elements_are_equal(norm_py, norm_rs)
    if equal:
        print("[+] Equivalência estrita confirmada! O XML do Rust e o do Python são idênticos.")
        return True
    else:
        print(f"[-] Divergência encontrada entre os XMLs:\n    {reason}", file=sys.stderr)
        return False


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(2)

    path_py = Path(sys.argv[1])
    path_rs = Path(sys.argv[2])

    equal = compare_xml_files(path_py, path_rs)
    sys.exit(0 if equal else 1)


if __name__ == "__main__":
    main()
