"""What the tools share; `python -I` leaves the script's directory off sys.path, so each tool adds it to import this."""
import sqlite3
from contextlib import closing
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OFFICIAL = ROOT / "crates/scripts/src/official"


def cdb(query):
    rows = {}
    for path in sorted((ROOT / "vendor/BabelCDB").glob("*.cdb")):
        if path.name == "cards.cdb" or path.name.startswith("release-"):
            with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as database:
                rows.update((row[0], row) for row in database.execute(query))
    return list(rows.values())


def ported_codes():
    # A batch's stub files (an empty initial_effect) are not ported yet.
    return {int(path.stem[1:]) for path in OFFICIAL.glob("c*.rs") if "CardId) {}" not in path.read_text(encoding="utf-8")}


def lua_script(code):
    return ROOT / f"vendor/CardScripts/official/c{code}.lua"
