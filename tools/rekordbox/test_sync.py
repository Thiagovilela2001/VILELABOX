import contextlib
import importlib.util
import io
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('bridge', Path(__file__).with_name('sync.py'))
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class ImportProtectionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.db = Path(self.temp.name) / 'mixxxdb.sqlite'
        with contextlib.closing(sqlite3.connect(self.db)) as conn:
            conn.execute('CREATE TABLE test (value TEXT)')
            conn.execute("INSERT INTO test VALUES ('original')")
            conn.commit()
        self.stopped = patch.object(bridge, 'assert_mixxx_stopped')
        self.stopped.start()
        self.addCleanup(self.stopped.stop)

    def test_backup_precedes_write_and_rollback(self):
        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaises(ValueError):
                with bridge.protected_import_connection(str(self.db)) as conn:
                    conn.execute("UPDATE test SET value='changed'")
                    raise ValueError('invalid XML data')
        backups = list(self.db.parent.glob('backups-rekordbox/*.bak'))
        self.assertEqual(len(backups), 1)
        for path in [self.db, backups[0]]:
            with contextlib.closing(sqlite3.connect(path)) as conn:
                self.assertEqual(conn.execute('SELECT value FROM test').fetchone()[0], 'original')

    def test_writer_lock_refuses_import_without_backup(self):
        with contextlib.closing(sqlite3.connect(self.db)) as writer:
            writer.execute('BEGIN IMMEDIATE')
            with self.assertRaises(sqlite3.OperationalError):
                with bridge.protected_import_connection(str(self.db)):
                    self.fail('must not write')
        self.assertFalse((self.db.parent / 'backups-rekordbox').exists())

    def test_wal_backup_includes_committed_data(self):
        with contextlib.closing(sqlite3.connect(self.db)) as writer:
            writer.execute('PRAGMA journal_mode=WAL')
            writer.execute("UPDATE test SET value='wal'")
            writer.commit()
            with contextlib.redirect_stdout(io.StringIO()):
                with bridge.protected_import_connection(str(self.db)) as conn:
                    conn.execute("UPDATE test SET value='imported'")
            backup = next(self.db.parent.glob('backups-rekordbox/*.bak'))
            with contextlib.closing(sqlite3.connect(backup)) as conn:
                self.assertEqual(conn.execute('SELECT value FROM test').fetchone()[0], 'wal')

    def test_running_process_refuses_import(self):
        bridge.assert_mixxx_stopped.side_effect = RuntimeError('running')
        with self.assertRaises(RuntimeError):
            with bridge.protected_import_connection(str(self.db)):
                self.fail('must not write')
        self.assertFalse((self.db.parent / 'backups-rekordbox').exists())


if __name__ == '__main__':
    unittest.main()
