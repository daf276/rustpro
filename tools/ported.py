"""Lists the cards SUPPORT can use (ported, or Normal Monsters without a script): code, name, kind, level/rank/link,
attribute and race. Filters: kinds (normal, effect, tuner, fusion, synchro, xyz, link, ritual, pendulum; spell or trap,
with quickplay, continuous, equip, field, counter; monsters unless spell or trap is given), level, attribute, race,
archetype (`setcode=0x8` matches 0x8 and its sub-archetypes such as 0x1008, as `IsSetCard` does) and part of the name.

  python tools/ported.py [tuner] [synchro] [trap continuous] [level=3] [attribute=WIND] [race=DRAGON] [setcode=0x8] [name=Kuriboh]
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import cdb, lua_script, ported_codes

KINDS = {"spell": 0x2, "trap": 0x4, "quickplay": 0x10000, "continuous": 0x20000, "equip": 0x40000, "field": 0x80000,
         "counter": 0x100000, "normal": 0x10, "effect": 0x20, "fusion": 0x40, "ritual": 0x80, "tuner": 0x1000, "synchro": 0x2000,
         "token": 0x4000, "xyz": 0x800000, "pendulum": 0x1000000, "link": 0x4000000}
ATTRIBUTES = ["EARTH", "WATER", "FIRE", "WIND", "LIGHT", "DARK", "DIVINE"]
RACES = ["WARRIOR", "SPELLCASTER", "FAIRY", "FIEND", "ZOMBIE", "MACHINE", "AQUA", "PYRO", "ROCK", "WINGEDBEAST", "PLANT",
         "INSECT", "THUNDER", "DRAGON", "BEAST", "BEASTWARRIOR", "DINOSAUR", "FISH", "SEASERPENT", "REPTILE", "PSYCHIC",
         "DIVINE", "CREATORGOD", "WYRM", "CYBERSE", "ILLUSION"]


def names(value, table):
    return "|".join(name for bit, name in enumerate(table) if value >> bit & 1)


def archetype(setcodes, wanted):
    parts = (setcodes >> shift & 0xFFFF for shift in (0, 16, 32, 48))
    return any(part & 0xFFF == wanted & 0xFFF and part & wanted == wanted for part in parts)


sys.stdout.reconfigure(encoding="utf-8")
filters = dict(arg.split("=", 1) for arg in sys.argv[1:] if "=" in arg)
kinds = [KINDS[arg] for arg in sys.argv[1:] if "=" not in arg]
ported = ported_codes()
rows = cdb("select d.id, t.name, d.type, d.level, d.attribute, d.race, d.setcode from datas d join texts t on t.id = d.id")
scripted = lambda code: lua_script(code).exists()
for code, name, kind, level, attribute, race, setcode in sorted(rows, key=lambda row: (row[3] & 0xff, row[0])):
    usable = code in ported or kind & KINDS["normal"] and not scripted(code)
    monster_wanted = not any(bit & (KINDS["spell"] | KINDS["trap"]) for bit in kinds)
    if not usable or bool(kind & 1) != monster_wanted or kind & KINDS["token"] or any(not kind & bit for bit in kinds):
        continue
    if ("level" in filters and level & 0xff != int(filters["level"])
            or "attribute" in filters and filters["attribute"].upper() not in names(attribute, ATTRIBUTES)
            or "race" in filters and filters["race"].upper() not in names(race, RACES).split("|")
            or "setcode" in filters and not archetype(setcode, int(filters["setcode"], 16))
            or "name" in filters and filters["name"].lower() not in name.lower()):
        continue
    kind_names = "|".join(key for key, bit in KINDS.items() if kind & bit)
    print(f"{code}\t{name}\t{kind_names}\t{level & 0xff}\t{names(attribute, ATTRIBUTES)}\t{names(race, RACES)}")
