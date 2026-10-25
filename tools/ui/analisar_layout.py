"""Deriva a geometria de layout a partir de uma medicao feita por tools/ui/medir_ui.ps1.

Entrada: diretorio com meta.txt, uia.tsv e cliente.raw (RGB cru da area de cliente).
Saida:   JSON com ambiente, faixas horizontais, paineis verticais e marcos semanticos.

Somente biblioteca padrao: a leitura dos pixels usa o .raw produzido pelo PowerShell,
sem PIL nem numpy, para o repo nao ganhar dependencia de imagem.

Uso:
    python tools/ui/analisar_layout.py third_party/rekordbox/ui-workbench/medicoes \\
        --saida spec/rekordbox/evidence/layout-rekordbox-7.2.8.json
"""

from __future__ import annotations

import argparse
import array
import json
import statistics
from pathlib import Path

TOLERANCIA_COR = 6   # diferenca maxima por canal para considerar a mesma cor de fundo
COMPRIMENTO_MIN = 8  # descarta micro-trechos (bordas de 1-2 px de texto/icone)


def ler_meta(diretorio: Path) -> dict[str, str]:
    meta: dict[str, str] = {}
    for linha in (diretorio / "meta.txt").read_text(encoding="utf-8-sig").splitlines():
        if "=" in linha:
            chave, valor = linha.split("=", 1)
            meta[chave.strip()] = valor.strip()
    return meta


class Cliente:
    """Pixels da area de cliente: RGB cru, 3 bytes por pixel, com stride."""

    def __init__(self, diretorio: Path, meta: dict[str, str]) -> None:
        self.largura = int(meta["cliente_l"])
        self.altura = int(meta["cliente_a"])
        self.stride = int(meta["cliente_stride"])
        dados = array.array("B")
        dados.frombytes((diretorio / "cliente.raw").read_bytes())
        self.dados = dados
        if len(self.dados) < self.stride * self.altura:
            raise ValueError("cliente.raw menor que o esperado para a area de cliente")

    def pixel(self, x: int, y: int) -> tuple[int, int, int]:
        base = y * self.stride + x * 3
        return (self.dados[base], self.dados[base + 1], self.dados[base + 2])

    def coluna_mediana(self, y0: int, y1: int) -> list[tuple[int, int, int]]:
        """Cor mediana de cada coluna na faixa vertical [y0, y1)."""
        saida = []
        ys = range(y0, y1)
        for x in range(self.largura):
            rs, gs, bs = [], [], []
            for y in ys:
                base = y * self.stride + x * 3
                rs.append(self.dados[base])
                gs.append(self.dados[base + 1])
                bs.append(self.dados[base + 2])
            saida.append((int(statistics.median(rs)), int(statistics.median(gs)), int(statistics.median(bs))))
        return saida

    def linha_mediana(self, x0: int, x1: int, y0: int, y1: int) -> list[tuple[int, int, int]]:
        """Cor mediana de cada linha da faixa vertical [y0, y1) na faixa horizontal [x0, x1)."""
        saida = []
        for y in range(y0, y1):
            rs, gs, bs = [], [], []
            base_linha = y * self.stride
            for x in range(x0, x1):
                base = base_linha + x * 3
                rs.append(self.dados[base])
                gs.append(self.dados[base + 1])
                bs.append(self.dados[base + 2])
            saida.append((int(statistics.median(rs)), int(statistics.median(gs)), int(statistics.median(bs))))
        return saida


def segmentar(cores: list[tuple[int, int, int]], deslocamento: int = 0,
              tolerancia: int = TOLERANCIA_COR, minimo: int = COMPRIMENTO_MIN) -> list[dict]:
    """Agrupa posicoes consecutivas de cor semelhante em trechos."""
    trechos: list[dict] = []
    inicio = 0
    for i in range(1, len(cores)):
        if max(abs(cores[i][c] - cores[i - 1][c]) for c in range(3)) > tolerancia:
            if i - inicio >= minimo:
                trechos.append({
                    "inicio": deslocamento + inicio,
                    "fim": deslocamento + i - 1,
                    "tamanho": i - inicio,
                    "cor": cores[inicio + 2],
                })
            inicio = i
    if len(cores) - inicio >= minimo:
        trechos.append({
            "inicio": deslocamento + inicio,
            "fim": deslocamento + len(cores) - 1,
            "tamanho": len(cores) - inicio,
            "cor": cores[-2],
        })
    return trechos


def ler_uia(diretorio: Path) -> list[dict]:
    linhas = (diretorio / "uia.tsv").read_text(encoding="utf-8-sig").splitlines()
    cabecalho = linhas[0].split("\t")
    nos = []
    for linha in linhas[1:]:
        partes = linha.split("\t")
        if len(partes) != len(cabecalho):
            continue
        no: dict[str, object] = dict(zip(cabecalho, partes))
        for chave in ("id", "depth", "parent", "x", "y", "largura", "altura"):
            no[chave] = int(str(no[chave]))
        nos.append(no)
    return nos


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("medicao", type=Path, help="diretorio produzido por medir_ui.ps1")
    ap.add_argument("--saida", type=Path, required=True, help="arquivo JSON de layout")
    args = ap.parse_args()

    meta = ler_meta(args.medicao)
    cliente = Cliente(args.medicao, meta)
    escala = float(meta["escala"])
    largura, altura = cliente.largura, cliente.altura

    # regioes: perfil de linhas usando uma coluna estreita que atravessa a janela
    perfil = cliente.linha_mediana(1200, 1210, 0, altura)
    faixas = segmentar(perfil, 0, tolerancia=TOLERANCIA_COR, minimo=5)

    # banda dos decks: os tres blocos horizontais aparecem logo abaixo da faixa de waveforms
    banda_decks = next((f for f in faixas if f["inicio"] >= 240 and f["fim"] <= 600), None)
    if banda_decks is None:
        banda_decks = {"inicio": 240, "fim": 578}
    y_decks = banda_decks["inicio"] + 5
    paineis = segmentar(cliente.coluna_mediana(y_decks, y_decks + 10), 0, minimo=15)
    paineis = [p for p in paineis if p["tamanho"] > 40]

    # navegador: colunas e faixas a partir dos marcos de acessibilidade
    nos = ler_uia(args.medicao)
    cx, cy = int(meta["cliente_x"]), int(meta["cliente_y"])

    def cliente_rel(no: dict) -> dict:
        return {
            "x": no["x"] - cx,
            "y": no["y"] - cy,
            "largura": no["largura"],
            "altura": no["altura"],
        }

    cabecalhos = [no for no in nos if no["tipo"] == "Header" and no["nome"].strip()]
    colunas = [{"titulo": no["nome"], **cliente_rel(no)} for no in cabecalhos]

    marcos = [
        {"nome": no["nome"], "tipo": no["tipo"], **cliente_rel(no)}
        for no in nos
        if no["nome"].strip() and no["nome"] != meta["processo"] and no["tipo"] in
        ("ComboBox", "Button", "Text", "Edit", "Header", "RadioButton", "ScrollBar")
    ]

    def pct(valor: float, total: int) -> float:
        return round(100.0 * valor / total, 2)

    regioes = []
    for faixa in faixas:
        regioes.append({
            "y": faixa["inicio"],
            "altura": faixa["tamanho"],
            "pct_altura": pct(faixa["tamanho"], altura),
            "cor_fundo": list(faixa["cor"]),
            "px_logicos": round(faixa["tamanho"] / escala, 1),
        })

    layout = {
        "fonte": {
            "medicao": str(args.medicao).replace("\\", "/"),
            "processo": meta["processo"],
            "tela": meta["tela"],
            "dpi": int(meta["dpi"]),
            "escala": escala,
            "cliente": {"largura": largura, "altura": altura,
                        "largura_logica": round(largura / escala, 1),
                        "altura_logica": round(altura / escala, 1)},
            "janela": {"x": int(meta["janela_x"]), "y": int(meta["janela_y"]),
                        "largura": int(meta["janela_l"]), "altura": int(meta["janela_a"])},
        },
        "regioes_horizontais": regioes,
        "banda_decks": {
            "y": banda_decks["inicio"],
            "altura": banda_decks["fim"] - banda_decks["inicio"] + 1,
            "paineis": [
                {
                    "x": p["inicio"],
                    "largura": p["tamanho"],
                    "pct_largura": pct(p["tamanho"], largura),
                    "cor_fundo": list(p["cor"]),
                    "px_logicos": round(p["tamanho"] / escala, 1),
                }
                for p in paineis
            ],
        },
        "colunas_navegador": colunas,
        "marcos": marcos,
    }

    args.saida.parent.mkdir(parents=True, exist_ok=True)
    args.saida.write_text(json.dumps(layout, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"cliente {largura}x{altura} @ escala {escala} (logico "
          f"{largura / escala:.0f}x{altura / escala:.0f})")
    print("faixas horizontais (cliente):")
    for r in regioes:
        print(f"  y {r['y']:>4}..{r['y'] + r['altura'] - 1:<4} h={r['altura']:<4} "
              f"{r['pct_altura']:>5}%  {r['px_logicos']:>6} px logicos  cor={r['cor_fundo']}")
    print(f"paineis da banda de decks (y {banda_decks['inicio']}):")
    for p in layout["banda_decks"]["paineis"]:
        print(f"  x {p['x']:>4}..{p['x'] + p['largura'] - 1:<4} w={p['largura']:<4} "
              f"{p['pct_largura']:>5}%  {p['px_logicos']:>6} px logicos")
    if colunas:
        print("colunas do navegador:")
        for c in colunas:
            print(f"  x {c['x']:>4} w={c['largura']:<4} {c['titulo']}")
    print(f"marcos semanticos: {len(marcos)} -> {args.saida}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
