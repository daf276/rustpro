"""Writes porting/queue-usage.tsv: porting/queue.tsv reordered by how many TCG tournament decks of the last two years
play each card (main, extra or side), most played first; the cards no deck plays keep their view order after them. The
count replaces weekly_views, so batch.py ranks archetypes by it too.

  python tools/usage.py DIR

DIR holds the pages of https://ygoprodeck.com/api/decks/getDecks.php?limit=20&offset=N&format=tournament%20meta%20decks&tournament
(20 decks a page, newest first; not committed), fetched for N = 0, 20, ... until a page is older than two years. A deck
counts once however often its pages repeat it, and is dated by the tournament date in its description.
"""
import json
import re
import sys
from collections import Counter
from datetime import date, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import ROOT, cdb

MONTHS = "January|February|March|April|May|June|July|August|September|October|November|December".split("|")
pool_rs = (ROOT / "crates/carddata/src/pool.rs").read_text(encoding="utf-8")
prerelease = {int(official): int(code) for official, code in re.findall(r"\((\d+), (\d+)\)", pool_rs.split("PRERELEASE_CODES")[1].split("];")[0])}
aliases = {code: alias for code, alias in cdb("select id, alias from datas") if alias and abs(code - alias) < 10}
names = dict(cdb("select id, name from texts"))
lines = (ROOT / "porting/queue.tsv").read_text(encoding="utf-8").splitlines()
rows = [line.split("\t") for line in lines[1:]]
queue = {int(row[0]) for row in rows}
by_name = {row[1]: int(row[0]) for row in rows}


def deck_date(deck):
    found = re.search(rf"({'|'.join(MONTHS)}) (\d+)(?:st|nd|rd|th)?,? (\d{{4}})", deck["deck_description"] or "")
    return date(int(found[3]), MONTHS.index(found[1]) + 1, int(found[2])) if found else None


def original(code):
    code = prerelease.get(code, code)
    code = aliases.get(code, code)
    return code if code in queue else by_name.get(names.get(code), code)


decks = {}
for path in sorted(Path(sys.argv[1]).glob("*.json")):
    page = json.loads(path.read_text(encoding="utf-8"))
    decks.update((deck["deckNum"], deck) for deck in (page if isinstance(page, list) else []))
dates = {number: deck_date(deck) for number, deck in decks.items()}
since = max(filter(None, dates.values())) - timedelta(days=730)
usage, counted, unknown = Counter(), [], set()
for number, deck in decks.items():
    if (dates[number] or since) >= since:
        counted.append(number)
        for code in {original(int(code)) for part in ("main_deck", "extra_deck", "side_deck") for code in json.loads(deck.get(part) or "[]")}:
            if code in queue:
                usage[code] += 1
            else:
                unknown.add(code)
undated = sum(dates[number] is None for number in counted)
print(f"{len(counted)} decks since {since} ({undated} undated), {len(usage)} cards, {len(unknown)} codes outside the queue", file=sys.stderr)
ranked = sorted(enumerate(rows), key=lambda pair: (-usage[int(pair[1][0])], pair[0]))
with open(ROOT / "porting/queue-usage.tsv", "w", encoding="utf-8", newline="\n") as out:
    out.write(lines[0] + "\n")
    for _, (code, name, kind, _, ported) in ranked:
        out.write(f"{code}\t{name}\t{kind}\t{usage[int(code)]}\t{ported}\n")
