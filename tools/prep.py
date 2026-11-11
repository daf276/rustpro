"""Prints a porting prep report for a batch: per card, the Lua API calls with no Rust function of the same snake_case
name (to port or to find under another name), and for calls few ported cards use, one ported card that uses it.

  python tools/prep.py <code>...
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import OFFICIAL, ROOT, lua_script

NAMESPACES = "Duel|Card|Effect|Group|aux|Auxiliary|Cost|Fusion|Synchro|Xyz|Link|Ritual|Pendulum|Gemini|Spirit|Equip|Union"
CALL = re.compile(rf"\b(?:{NAMESPACES})\.([A-Z]\w*)|:([A-Z]\w*)\s*\(")
RARE = 3
# Lua names whose Rust mirror is named differently (std's Clone takes `clone`).
ALIASES = {
    "clone": "clone_effect",
    "add_equip_procedure": "add_procedure",
    "is_s_setable": "is_ssetable",
    "s_set": "sset",
    "detach_from_self": "detach_from_self_with",
    "add_persistent_procedure": "add_procedure",
    "add_union_procedure": "add_procedure",
    "add_cannot_be_special_summoned": "add_must_be_special_summoned",
    "persistent_target_filter": "target_filter",
}
# Lua calls with no function of their own in the port.
NATIVE = {
    "create_group": "CardSet::default()",
    "keep_alive": "nothing, groups are values",
    "delete_group": "nothing, groups are values",
    "filter_bool_function": "a closure",
    "filter_bool_function_ex": "a closure",
    "not": "a closure negating the filter",
    "and": "a closure combining the filters",
    "true": "aux::TRUE (also the nil filter)",
    "infinite_mats": "the constant Xyz::INFINITE_MATS",
    "non_tuner_ex": "written inline (c.is_not_tuner(...) && the filter)",
    "from_cards": "CardSet::from_iter",
    "sub": "CardSet::difference",
    "add_card": "CardSet::insert",
    "remove_card": "CardSet::remove",
    "field_summon_proc_tg": "set_target_card(fun1) (see c89252153)",
    "filter_summon_code": "written inline (c.is_summon_code(duel, Some(sc), sumtype, tp, &[code]))",
    "target_bool_function": "a closure passed to set_target_card",
    "match": "CardSet::filter / keep_matching",
    "level": "CardId::set_level (Card.Level with a value)",
    "has_flag_effect": "get_flag_effect(...) > 0",
    "announce_level": "announce_number_range(tp, min, max) when the Lua passes a range, else announce_level",
    "add_proc_fun_rep": "FusionMix with SUB = false and the filter repeated",
}


def snake(name):
    for lua, rust in (("Sendto", "SendTo"), ("MZone", "Mzone"), ("SZone", "Szone")):
        name = name.replace(lua, rust)
    name = re.sub(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", "_", name).lower()
    return ALIASES.get(name, name)


sources = [*(ROOT / "crates/engine/src").glob("*.rs"), *(ROOT / "crates/scripts/src").glob("*.rs")]
defined = {name for path in sources for name in re.findall(r"\bfn (\w+)", path.read_text(encoding="utf-8"))}
cards = {path.stem: path.read_text(encoding="utf-8") for path in OFFICIAL.glob("c*.rs")}
users = defaultdict(list)
for stem, text in cards.items():
    for name in set(re.findall(r"\b([a-z_][a-z0-9_]*)\s*(?:::<[^>]*>)?\(", text)):
        users[name].append(stem)

for code in sys.argv[1:]:
    script = lua_script(code)
    if not script.exists():
        print(f"c{code}: no script")
        continue
    lua = re.sub(r"--.*", "", script.read_text(encoding="utf-8", errors="replace"))
    calls = sorted({snake(a or b) for a, b in CALL.findall(lua)} - {"register_effect"})
    native = [f"{name} is {NATIVE[name]}" for name in calls if name in NATIVE]
    missing = [name for name in calls if name not in defined and f"{name}_full" not in defined and name not in NATIVE]
    rare = [f"{name} ({users[name][0]})" for name in calls if name in defined and 0 < len(users[name]) <= RARE]
    unused = [name for name in calls if name in defined and not users[name]]
    print(f"c{code}:")
    if missing:
        print(f"  no Rust fn: {', '.join(missing)}")
    if unused:
        print(f"  defined, no ported card calls it yet: {', '.join(unused)}")
    if native:
        print(f"  native: {'; '.join(native)}")
    if rare:
        print(f"  rarely used, example: {', '.join(rare)}")
