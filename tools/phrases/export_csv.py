#!/usr/bin/env python3
"""Exports detected phrases to a CSV for manual annotation (PLANO-3, fase 6).

    python3 tools/phrases/export_csv.py [perfil] > frases.csv

Reads <perfil>/analysis/phrases/*.phrases.json (default ~/.mixxx). One row per
phrase; fill in the last three columns while listening:
  limite_ok   : sim / não (the boundary is within one bar of the real one)
  tipo_correto: INTRO, UP, CHORUS, DOWN, VERSE or OUTRO
  observacao  : free text
"""
import csv
import json
import sys
from pathlib import Path

TYPES = ["?", "INTRO", "UP", "CHORUS", "DOWN", "VERSE", "OUTRO"]


def rows(profile: Path):
    for path in sorted((profile / "analysis" / "phrases").glob("*.phrases.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        track = data.get("location") or path.name
        for index, phrase in enumerate(data.get("phrases", [])):
            start, end, bars, kind, confidence = (phrase + [0] * 5)[:5]
            yield [Path(track).name, track, index + 1, f"{start:.2f}", f"{end:.2f}", bars,
                   TYPES[kind] if 0 <= int(kind) < len(TYPES) else "?", f"{confidence:.2f}", "", "", ""]


def main():
    profile = Path(sys.argv[1]).expanduser() if len(sys.argv) > 1 else Path.home() / ".mixxx"
    writer = csv.writer(sys.stdout)
    writer.writerow(["faixa", "caminho", "frase", "inicio_s", "fim_s", "compassos", "tipo_detectado",
                     "confianca", "limite_ok", "tipo_correto", "observacao"])
    count = 0
    for row in rows(profile):
        writer.writerow(row)
        count += 1
    if count == 0:
        print("Nenhuma frase encontrada: ligue 'Analisar frases musicais' nas preferências "
              "e carregue ou analise as faixas.", file=sys.stderr)


if __name__ == "__main__":
    main()
