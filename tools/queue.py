"""Writes porting/queue.tsv: the TCG Advanced pool sorted by ygoprodeck weekly views, most viewed first.

  cargo run --release -p diff --bin pool > pool.tsv
  python tools/queue.py pool.tsv cardinfo.json

With --ocg it writes porting/queue-ocg.tsv instead, the OCG-only cards (OCG flag, no TCG flag) of a pool built with the
ocg feature:

  cargo run --release -p diff --bin pool --features ocg > pool-ocg.tsv
  python tools/queue.py --ocg pool-ocg.tsv cardinfo.json

cardinfo.json is https://db.ygoprodeck.com/api/v7/cardinfo.php?misc=yes (not committed). Cards it lacks by code (the
pre-release codes) are matched by name; cards it lacks altogether (new sets) get their name from the cdb files the pool
reads, no type and 0 views.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import ROOT, cdb

OT_OCG, OT_TCG = 0x1, 0x2
ocg = "--ocg" in sys.argv
pool_path, cards_path = (arg for arg in sys.argv[1:] if arg != "--ocg")
cards = {card["id"]: card for card in json.loads(Path(cards_path).read_text(encoding="utf-8"))["data"]}
by_name = {card["name"]: card for card in cards.values()}
names = dict(cdb("select id, name from texts"))
ots = dict(cdb("select id, ot from datas"))
rows = []
for line in Path(pool_path).read_text(encoding="utf-8").splitlines():
    code, ported = line.split("\t")
    if ocg and ots.get(int(code), 0) & (OT_OCG | OT_TCG) != OT_OCG:
        continue
    name = names.get(int(code), "")
    card = cards.get(int(code)) or by_name.get(name, {"name": name})
    views = (card.get("misc_info") or [{}])[0].get("viewsweek", 0)
    rows.append((-views, int(code), card.get("name", ""), card.get("type", ""), ported))
with open(ROOT / ("porting/queue-ocg.tsv" if ocg else "porting/queue.tsv"), "w", encoding="utf-8", newline="\n") as out:
    out.write("code\tname\ttype\tweekly_views\tported\n")
    for views, code, name, kind, ported in sorted(rows):
        out.write(f"{code}\t{name}\t{kind}\t{-views}\t{ported}\n")
