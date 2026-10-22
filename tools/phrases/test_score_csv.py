#!/usr/bin/env python3
"""Testes unitários para tools/phrases/score_csv.py."""

import csv
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

# Adiciona o diretório tools/phrases ao path para importação direta
sys.path.insert(0, str(Path(__file__).resolve().parent))
import score_csv


def write_test_csv(file_path: Path, rows: list[list[str]]):
    with file_path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "faixa", "caminho", "frase", "inicio_s", "fim_s", "compassos",
            "tipo_detectado", "confianca", "limite_ok", "tipo_correto", "observacao"
        ])
        for r in rows:
            writer.writerow(r)


class TestScoreCsv(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.dir_path = Path(self.temp_dir.name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_tudo_certo_20_faixas_aprovado(self):
        """20 faixas com >= 80% de limites_ok aprovam."""
        rows = []
        for i in range(1, 21):
            track_name = f"Faixa_{i}.mp3"
            # 2 frases por faixa, ambas com limite_ok = sim, tipo_correto correto
            rows.append([track_name, f"/path/{track_name}", "1", "0.00", "15.00", "8", "INTRO", "0.90", "sim", "INTRO", "ok"])
            rows.append([track_name, f"/path/{track_name}", "2", "15.00", "30.00", "8", "UP", "0.85", "sim", "UP", "ok"])

        csv_file = self.dir_path / "aprovado.csv"
        write_test_csv(csv_file, rows)

        res = score_csv.calculate_score(csv_file)
        self.assertEqual(res["num_tracks_annotated"], 20)
        self.assertEqual(res["total_limite_eval"], 40)
        self.assertEqual(res["total_limite_ok"], 40)
        self.assertEqual(res["limite_pct"], 100.0)
        self.assertEqual(res["tipo_pct"], 100.0)
        self.assertTrue(res["passed"])

        # Testa saída via CLI
        cmd = [sys.executable, str(Path(score_csv.__file__)), str(csv_file)]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0)
        self.assertIn("RESULTADO: APROVADO", proc.stdout)

    def test_abaixo_da_meta_limites(self):
        """20 faixas, mas limites_ok < 80% é reprovado."""
        rows = []
        for i in range(1, 21):
            track_name = f"Faixa_{i}.mp3"
            # 1 limite sim e 3 não por faixa -> 25% acurácia de limites
            rows.append([track_name, f"/path/{track_name}", "1", "0.00", "15.00", "8", "INTRO", "0.90", "sim", "INTRO", ""])
            rows.append([track_name, f"/path/{track_name}", "2", "15.00", "30.00", "8", "UP", "0.85", "não", "UP", ""])
            rows.append([track_name, f"/path/{track_name}", "3", "30.00", "45.00", "8", "CHORUS", "0.85", "nao", "CHORUS", ""])
            rows.append([track_name, f"/path/{track_name}", "4", "45.00", "60.00", "8", "OUTRO", "0.85", "não", "OUTRO", ""])

        csv_file = self.dir_path / "abaixo_meta.csv"
        write_test_csv(csv_file, rows)

        res = score_csv.calculate_score(csv_file)
        self.assertEqual(res["num_tracks_annotated"], 20)
        self.assertEqual(res["limite_pct"], 25.0)
        self.assertFalse(res["passed"])

        cmd = [sys.executable, str(Path(score_csv.__file__)), str(csv_file)]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 1)
        self.assertIn("RESULTADO: REPROVADO", proc.stdout)
        self.assertIn("acurácia de limites abaixo da meta", proc.stdout)

    def test_menos_de_20_faixas_reprovado(self):
        """Menos de 20 faixas anotadas é reprovado mesmo com 100% de limites corretos."""
        rows = []
        for i in range(1, 10):  # Apenas 9 faixas
            track_name = f"Faixa_{i}.mp3"
            rows.append([track_name, f"/path/{track_name}", "1", "0.00", "15.00", "8", "INTRO", "0.90", "sim", "INTRO", ""])

        csv_file = self.dir_path / "poucas_faixas.csv"
        write_test_csv(csv_file, rows)

        res = score_csv.calculate_score(csv_file)
        self.assertEqual(res["num_tracks_annotated"], 9)
        self.assertEqual(res["limite_pct"], 100.0)
        self.assertFalse(res["passed"])

        cmd = [sys.executable, str(Path(score_csv.__file__)), str(csv_file)]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(proc.returncode, 1)
        self.assertIn("faixas insuficientes (9/20)", proc.stdout)

    def test_linhas_vazias_e_sem_anotacao(self):
        """Linhas sem preenchimento são ignoradas do cálculo e contadas à parte."""
        rows = [
            # Faixa 1: 1 anotada, 2 vazias
            ["Faixa_1.mp3", "/path/1", "1", "0.00", "15.00", "8", "INTRO", "0.90", "sim", "INTRO", ""],
            ["Faixa_1.mp3", "/path/1", "2", "15.00", "30.00", "8", "UP", "0.80", "", "", ""],
            ["Faixa_1.mp3", "/path/1", "3", "30.00", "45.00", "8", "CHORUS", "0.80", "", "", ""],
            # Faixa 2: totalmente não anotada
            ["Faixa_2.mp3", "/path/2", "1", "0.00", "15.00", "8", "INTRO", "0.90", "", "", ""],
            ["Faixa_2.mp3", "/path/2", "2", "15.00", "30.00", "8", "UP", "0.80", "", "", ""],
        ]

        csv_file = self.dir_path / "linhas_vazias.csv"
        write_test_csv(csv_file, rows)

        res = score_csv.calculate_score(csv_file)
        self.assertEqual(res["total_rows"], 5)
        self.assertEqual(res["total_unannotated"], 4)
        self.assertEqual(res["num_tracks_annotated"], 1)
        self.assertEqual(res["total_limite_eval"], 1)
        self.assertEqual(res["total_limite_ok"], 1)
        self.assertEqual(res["limite_pct"], 100.0)
        self.assertEqual(res["tipo_pct"], 100.0)
        self.assertFalse(res["passed"])  # Reprovado por ter só 1 faixa anotada


if __name__ == "__main__":
    unittest.main()
