"""Prints the next porting batches: unported cards of porting/queue.tsv grouped by archetype, the archetypes with the most
weekly views first, each batch with the unported cards its Lua names (listed names, materials, searches) that are in
the pool. Cards outside any archetype are batched by views. One batch per line: code<TAB>name;code<TAB>name;...

  python tools/batch.py [batches] [size] [skip codes...]

`skip` takes codes already assigned to running batches.
"""
import csv
import re
import sqlite3
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
batches, size = (int(arg) for arg in (sys.argv[1:3] + ["4", "16"][len(sys.argv[1:3]) :]))
skip = {int(code) for code in sys.argv[3:]}
ported = {int(path.stem[1:]) for path in (ROOT / "crates/scripts/src/official").glob("c*.rs")} | skip
rows = list(csv.DictReader(open(ROOT / "porting/queue.tsv", encoding="utf-8"), delimiter="\t"))
pool = {int(row["code"]): row for row in rows if row["ported"] != "no script"}
todo = [int(row["code"]) for row in rows if row["ported"] == "no" and int(row["code"]) not in ported]
setcodes = dict(sqlite3.connect((ROOT / "vendor/BabelCDB/cards.cdb").as_uri() + "?mode=ro", uri=True).execute("select id, setcode from datas"))


def named(code):
    script = ROOT / f"vendor/CardScripts/official/c{code}.lua"
    text = script.read_text(encoding="utf-8", errors="replace") if script.exists() else ""
    codes = {int(found) for found in re.findall(r"\b(\d{4,9})\b", text)} - {code}
    return [other for other in codes if other in pool and other not in ported]


groups = defaultdict(list)
for code in todo:
    groups[setcodes.get(code, 0) & 0xFFF].append(code)
loose = groups.pop(0, [])
views = lambda codes: sum(int(pool[code]["weekly_views"]) for code in codes)
order = sorted(groups.values(), key=lambda codes: -views(codes)) + [[code] for code in loose]
taken, batch, out = set(), [], []
for group in order:
    for code in group:
        for card in [code, *named(code)]:
            if card not in taken:
                taken.add(card)
                batch.append(card)
        if len(batch) >= size:
            out.append(batch)
            batch = []
            if len(out) == batches:
                break
    if len(out) == batches:
        break
for batch in out:
    print(";".join(f"{code}\t{pool[code]['name']}" for code in batch))
