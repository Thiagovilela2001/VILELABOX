import re
import shutil
import sqlite3
import sys
from pathlib import Path

source = Path(__file__).resolve().parent.parent
profile = Path(sys.argv[1]).resolve()
profile.mkdir(parents=True, exist_ok=True)
(profile / 'skins').mkdir(exist_ok=True)
skin = profile / 'skins' / 'RbxFlowBars'
if not skin.exists():
    skin.symlink_to(source / 'skins' / 'RbxFlowBars', target_is_directory=True)
if not (profile / 'mixxx.cfg').exists():
    config = (source / 'mixxx.cfg').read_text()
    config = re.sub(r'^ResizableSkin\s.*$', 'ResizableSkin RbxFlowBars', config, flags=re.M)
    config = re.sub(r'^ResizableSkin=.*\n?', '', config, flags=re.M)
    (profile / 'mixxx.cfg').write_text(config)
    database = source / 'mixxxdb.sqlite'
    if database.exists():
        with sqlite3.connect(f'{database.as_uri()}?mode=ro', uri=True) as original:
            with sqlite3.connect(profile / 'mixxxdb.sqlite') as copied:
                original.backup(copied)
    for name in ('soundconfig.xml', 'effects.xml', 'samplers.xml'):
        if (source / name).exists():
            shutil.copy2(source / name, profile / name)
    for name in ('controllers', 'effects', 'analysis'):
        if (source / name).is_dir():
            shutil.copytree(source / name, profile / name, dirs_exist_ok=True)
print(f'Perfil de teste: {profile}')
