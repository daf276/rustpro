"""What the tools share; `python -I` leaves the script's directory off sys.path, so each tool adds it to import this."""
import re
import sqlite3
from contextlib import closing
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OFFICIAL = ROOT / "crates/scripts/src/official"
CONSTANT_SOURCES = ["constant.lua", "card_counter_constants.lua", "archetype_setcode_constants.lua"]
# The TCG-only Blazing Eternity cards, still under their pre-release codes upstream; goes away when BabelCDB migrates them.
PRERELEASE_CDB = "prerelease-betb-en.cdb"


def lua_constants():
    for source in CONSTANT_SOURCES:
        text = (ROOT / "vendor/CardScripts" / source).read_text(encoding="utf-8")
        for line in re.sub(r"--\[\[.*?\]\]", "", text, flags=re.S).splitlines():
            match = re.match(r"^(\w+)\s*=\s*(.+?)\s*$", line.split("--")[0])
            if match:
                yield match[1], strip_parentheses(match[2].replace("0X", "0x"))


def strip_parentheses(expression):
    depths = [expression[:end].count("(") - expression[:end].count(")") for end in range(1, len(expression))]
    return expression[1:-1] if expression.startswith("(") and all(depths) else expression


def cdb(query):
    rows = {}
    for path in sorted((ROOT / "vendor/BabelCDB").glob("*.cdb")):
        if path.name in ("cards.cdb", PRERELEASE_CDB) or path.name.startswith("release-"):
            with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as database:
                rows.update((row[0], row) for row in database.execute(query))
    return list(rows.values())


def ported_codes():
    # A batch's stub files (an empty initial_effect) are not ported yet.
    return {int(path.stem[1:]) for path in OFFICIAL.glob("c*.rs") if "CardId) {}" not in path.read_text(encoding="utf-8")}


def lua_script(code):
    official = ROOT / f"vendor/CardScripts/official/c{code}.lua"
    return official if official.exists() else ROOT / f"vendor/CardScripts/pre-release/c{code}.lua"
