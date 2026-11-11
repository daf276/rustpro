"""Writes porting/queue.tsv: the TCG Advanced pool sorted by ygoprodeck weekly views, most viewed first.

  cargo run --release -p diff --bin pool > pool.tsv
  python tools/queue.py pool.tsv cardinfo.json

cardinfo.json is https://db.ygoprodeck.com/api/v7/cardinfo.php?misc=yes (not committed). Cards it lacks (new sets)
get their name from the cdb files the pool reads, no type and 0 views.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import ROOT, cdb

pool_path, cards_path = sys.argv[1:3]
cards = {card["id"]: card for card in json.loads(Path(cards_path).read_text(encoding="utf-8"))["data"]}
names = dict(cdb("select id, name from texts"))
rows = []
for line in Path(pool_path).read_text(encoding="utf-8").splitlines():
    code, ported = line.split("\t")
    card = cards.get(int(code), {"name": names.get(int(code), "")})
    views = (card.get("misc_info") or [{}])[0].get("viewsweek", 0)
    rows.append((-views, int(code), card.get("name", ""), card.get("type", ""), ported))
with open(ROOT / "porting/queue.tsv", "w", encoding="utf-8", newline="\n") as out:
    out.write("code\tname\ttype\tweekly_views\tported\n")
    for views, code, name, kind, ported in sorted(rows):
        out.write(f"{code}\t{name}\t{kind}\t{-views}\t{ported}\n")
