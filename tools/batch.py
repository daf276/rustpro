"""Prints the next porting batches: unported cards of the queue (porting/queue.tsv) grouped by archetype, the archetypes with the most
weekly views first, each batch with the unported cards its Lua names (listed names, materials, searches) that are in
the pool. Cards outside any archetype rank among the archetypes by their own views. One batch per line: code<TAB>name;code<TAB>name;...

  python tools/batch.py [--queue=FILE] [--top=N] [--type=REGEX] [batches] [size] [skip codes...]

`--queue` plans from another queue file, e.g. porting/queue-ocg.tsv; the cards its Lua names may also come from
porting/queue.tsv. `skip` takes codes already assigned to running batches; `--top` picks only among the N most viewed cards and `--type` only
among cards whose queue type matches (the cards their Lua names still come from the whole pool).
"""
import csv
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import ROOT, cdb, lua_script, ported_codes

top = next((int(arg[6:]) for arg in sys.argv[1:] if arg.startswith("--top=")), None)
queue = next((arg[8:] for arg in sys.argv[1:] if arg.startswith("--queue=")), "porting/queue.tsv")
kind = next((arg[7:] for arg in sys.argv[1:] if arg.startswith("--type=")), "")
args = [arg for arg in sys.argv[1:] if not arg.startswith("--")]
batches, size = (int(arg) for arg in (args[:2] + ["4", "16"][len(args[:2]) :]))
skip = {int(code) for code in args[2:]}
ported = ported_codes() | skip
read = lambda path: list(csv.DictReader(open(ROOT / path, encoding="utf-8"), delimiter="\t"))
rows = read(queue)
pool = {int(row["code"]): row for row in read("porting/queue.tsv") + rows if row["ported"] != "no script"}
todo = [int(row["code"]) for row in rows[:top] if row["ported"] == "no" and int(row["code"]) not in ported and re.search(kind, row["type"])]
setcodes = dict(cdb("select id, setcode from datas"))


def named(code):
    script = lua_script(code)
    text = script.read_text(encoding="utf-8", errors="replace") if script.exists() else ""
    codes = {int(found) for found in re.findall(r"\b(\d{4,9})\b", text)} - {code}
    return [other for other in codes if other in pool and other not in ported]


groups = defaultdict(list)
for code in todo:
    groups[setcodes.get(code, 0) & 0xFFF].append(code)
loose = groups.pop(0, [])
views = lambda codes: sum(int(pool[code]["weekly_views"]) for code in codes)
order = sorted([*groups.values(), *([code] for code in loose)], key=lambda codes: -views(codes))
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
if batch and len(out) < batches:
    out.append(batch)
for batch in out:
    print(";".join(f"{code}\t{pool[code]['name']}" for code in batch))
