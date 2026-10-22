#!/usr/bin/env python3
"""Calcula as métricas de anotação de frases a partir do CSV preenchido.

Uso:
    python3 tools/phrases/score_csv.py <caminho/frases.csv>

Critérios de aprovação (retorna 0):
    - Pelo menos 20 faixas anotadas
    - Pelo menos 80.0% dos limites de frase anotados como 'sim'
Caso contrário, retorna 1.
"""
import csv
import sys
from collections import defaultdict
from pathlib import Path


def is_truthy(val: str) -> bool:
    return val.strip().lower() in ("sim", "s", "true", "1", "yes", "y")


def calculate_score(csv_path: str | Path):
    path = Path(csv_path)
    if not path.is_file():
        raise FileNotFoundError(f"Arquivo CSV não encontrado: {path}")

    track_stats = defaultdict(lambda: {
        "total_rows": 0,
        "limite_eval": 0,
        "limite_ok": 0,
        "tipo_eval": 0,
        "tipo_ok": 0,
        "unannotated": 0,
    })

    total_rows = 0
    total_unannotated = 0
    total_limite_eval = 0
    total_limite_ok = 0
    total_tipo_eval = 0
    total_tipo_ok = 0

    with path.open(mode="r", encoding="utf-8-sig", newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            total_rows += 1
            track_name = (row.get("faixa") or row.get("caminho") or "Desconhecida").strip()
            limite_raw = (row.get("limite_ok") or "").strip()
            tipo_correto = (row.get("tipo_correto") or "").strip()
            tipo_detectado = (row.get("tipo_detectado") or "").strip()

            stats = track_stats[track_name]
            stats["total_rows"] += 1

            is_annotated = False
            if limite_raw:
                is_annotated = True
                stats["limite_eval"] += 1
                total_limite_eval += 1
                if is_truthy(limite_raw):
                    stats["limite_ok"] += 1
                    total_limite_ok += 1

            if tipo_correto:
                is_annotated = True
                stats["tipo_eval"] += 1
                total_tipo_eval += 1
                if tipo_correto.upper() == tipo_detectado.upper():
                    stats["tipo_ok"] += 1
                    total_tipo_ok += 1

            if not is_annotated:
                stats["unannotated"] += 1
                total_unannotated += 1

    tracks_annotated = [t for t, s in track_stats.items() if (s["limite_eval"] > 0 or s["tipo_eval"] > 0)]
    num_tracks_annotated = len(tracks_annotated)

    limite_pct = (total_limite_ok / total_limite_eval * 100.0) if total_limite_eval > 0 else 0.0
    tipo_pct = (total_tipo_ok / total_tipo_eval * 100.0) if total_tipo_eval > 0 else 0.0

    passed = (num_tracks_annotated >= 20) and (limite_pct >= 80.0)

    return {
        "track_stats": dict(track_stats),
        "total_rows": total_rows,
        "total_unannotated": total_unannotated,
        "num_tracks_annotated": num_tracks_annotated,
        "total_limite_eval": total_limite_eval,
        "total_limite_ok": total_limite_ok,
        "limite_pct": limite_pct,
        "total_tipo_eval": total_tipo_eval,
        "total_tipo_ok": total_tipo_ok,
        "tipo_pct": tipo_pct,
        "passed": passed,
    }


def print_report(results: dict):
    print("=" * 70)
    print("RELATÓRIO DE ANOTAÇÃO DE FRASES MUSICAIS")
    print("=" * 70)

    print(f"{'Faixa':<35} | {'Limites OK':<14} | {'Tipos OK':<14}")
    print("-" * 70)
    for track, s in sorted(results["track_stats"].items()):
        lim_str = f"{s['limite_ok']}/{s['limite_eval']} ({s['limite_ok']/s['limite_eval']*100:.1f}%)" if s["limite_eval"] else "–"
        tip_str = f"{s['tipo_ok']}/{s['tipo_eval']} ({s['tipo_ok']/s['tipo_eval']*100:.1f}%)" if s["tipo_eval"] else "–"
        display_name = track if len(track) <= 35 else track[:32] + "..."
        print(f"{display_name:<35} | {lim_str:<14} | {tip_str:<14}")

    print("-" * 70)
    print(f"Total de linhas lidas: {results['total_rows']}")
    print(f"Linhas sem anotação (ignoradas): {results['total_unannotated']}")
    print(f"Faixas anotadas: {results['num_tracks_annotated']} (meta: >= 20)")
    lim_total = (
        f"{results['total_limite_ok']}/{results['total_limite_eval']} "
        f"({results['limite_pct']:.1f}%)"
        if results['total_limite_eval'] else "0/0 (0.0%)"
    )
    print(f"Acurácia de limites (limite_ok = sim): {lim_total} (meta: >= 80.0%)")
    tip_total = (
        f"{results['total_tipo_ok']}/{results['total_tipo_eval']} "
        f"({results['tipo_pct']:.1f}%)"
        if results['total_tipo_eval'] else "0/0 (0.0%)"
    )
    print(f"Acurácia de rótulo (tipo_correto == tipo_detectado): {tip_total}")
    print("=" * 70)
    if results["passed"]:
        print("RESULTADO: APROVADO (meta de 20 faixas e >= 80% atingida)")
    else:
        reasons = []
        if results["num_tracks_annotated"] < 20:
            reasons.append(f"faixas insuficientes ({results['num_tracks_annotated']}/20)")
        if results["limite_pct"] < 80.0:
            reasons.append(f"acurácia de limites abaixo da meta ({results['limite_pct']:.1f}% < 80.0%)")
        print(f"RESULTADO: REPROVADO ({', '.join(reasons)})")
    print("=" * 70)


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help"):
        print(__doc__)
        sys.exit(2)

    csv_path = sys.argv[1]
    try:
        results = calculate_score(csv_path)
    except Exception as e:
        print(f"Erro ao processar CSV: {e}", file=sys.stderr)
        sys.exit(2)

    print_report(results)
    sys.exit(0 if results["passed"] else 1)


if __name__ == "__main__":
    main()
