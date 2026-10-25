"""Gera uma cópia SQLite comum (sem criptografia) do catálogo do rekordbox.

O VILELABOX lê só essa cópia (spec/rekordbox/10-master-db.md, contrato do
`vilelabox-catalog`): nenhuma chave entra no repositório. A abertura do banco
criptografado é feita pela biblioteca externa `pyrekordbox`, instalada pelo
usuário (`python -m pip install --user pyrekordbox`).

O original nunca é aberto: o script copia `master.db` (e `-wal`/`-shm`, se
existirem) para uma pasta temporária e trabalha na cópia.

Uso:
    python tools/catalog/exportar_catalogo.py [saida.db]

Padrão da saída: %LOCALAPPDATA%\\vilelabox\\catalogo.db (fora do repositório).
Depois: cargo run -p vilelabox-app -- <saida.db>
"""

import os
import shutil
import sys
import tempfile
from pathlib import Path


def main() -> int:
    try:
        from pyrekordbox import Rekordbox6Database
    except ImportError:
        print("pyrekordbox não instalado: python -m pip install --user pyrekordbox")
        return 1

    origem = Path(os.environ["APPDATA"]) / "Pioneer" / "rekordbox" / "master.db"
    padrao = Path(os.environ["LOCALAPPDATA"]) / "vilelabox" / "catalogo.db"
    saida = Path(sys.argv[1]) if len(sys.argv) > 1 else padrao
    saida.parent.mkdir(parents=True, exist_ok=True)
    if saida.exists():
        saida.unlink()

    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp:
        copia = Path(tmp) / "master.db"
        for sufixo in ("", "-wal", "-shm"):
            arq = origem.with_name(origem.name + sufixo)
            if arq.exists():
                shutil.copy2(arq, copia.with_name(copia.name + sufixo))
        db = Rekordbox6Database(path=copia)
        conexao = db.engine.raw_connection()
        cur = conexao.cursor()
        cur.execute("ATTACH DATABASE ? AS plaintext KEY ''", (str(saida),))
        cur.execute("SELECT sqlcipher_export('plaintext')")
        cur.execute("DETACH DATABASE plaintext")
        conexao.close()
        db.close()
        db.engine.dispose()  # libera a cópia temporária para a limpeza

    print(f"catálogo exportado: {saida}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
