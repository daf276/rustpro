"""Checks that every ported card's metadata consts match its Lua script, and lists per table the cards whose const is
missing, extra or different. Values compare the way the core reads them: lists as sets, an absent list as empty or as
its FALLBACK, an absent bool as false. Exits non-zero on any mismatch; tables to check can be named (`listed_series`).

  python -I tools/check_metadata.py [TABLE ...]
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import OFFICIAL, cdb, lua_constants, lua_script, ported_codes

LISTS = ["listed_names", "fit_monster", "listed_series", "counter_place_list", "counter_list", "material_setcode", "material"]
BOOLS = ["dark_calling", "toss_coin", "roll_dice"]
NUMBERS = ["xyz_number", "assault_mode", "pendulum_level"]
FIELDS = LISTS + BOOLS + NUMBERS
# What Card.ListsCode and Card.ListsCounter read when the first field is nil.
FALLBACK = {"listed_names": "fit_monster", "counter_list": "counter_place_list"}
# Fields not compared when the Lua calls the procedure: a Fusion procedure with code materials replaces s.material, and
# the Rust one records them in the duel instead of a const.
SKIPPED = {"material": "Fusion.AddProc"}
# Ritual.AddProcCode(c,_type,lv,desc,...) and its Greater/Equal forms (c,lv,desc,...) set fit_monster to `...`.
RITUAL_CODE = re.compile(r"Ritual\.AddProc(Greater|Equal)?Code\(")


def evaluate(expression, lookup):
    expression = re.sub(r"\bas [ui]\d+\b", "", expression).replace("!", "~")
    python = re.sub(r"\b[A-Za-z_][\w:]*", lambda match: str(lookup(match[0])), expression)
    if not re.fullmatch(r"[\s\dxa-fA-F+\-*|&~()<>]+", python):
        raise ValueError(expression)
    return eval(python.strip(), {"__builtins__": {}})


def parse(field, text, lookup):
    if text in ("true", "false", "nil"):
        return text == "true"
    items = text.strip("&[]{} \n").split(",") if text.lstrip("&").startswith(("[", "{")) else [text]
    values = [evaluate(item, lookup) for item in items if item.strip()]
    return frozenset(values) if field in LISTS else values[0]


def call_arguments(text, start):
    depth, arguments, current = 0, [], ""
    for char in text[start:]:
        if char == ")" and depth == 0:
            return arguments + [current]
        depth += (char in "({") - (char in ")}")
        if char == "," and depth == 0:
            arguments, current = arguments + [current], ""
        else:
            current += char
    return arguments


def lua_metadata(code, constants):
    text = lua_script(code).read_text(encoding="utf-8")
    found = dict(re.findall(r"^\s*s\.(\w+)\s*=\s*(.*?)\s*(?:--.*)?$", text, re.M))
    found = {field: found.get(value[2:], value) if value.startswith("s.") else value for field, value in found.items()}
    ritual = RITUAL_CODE.search(text)
    if ritual and "fit_monster" not in found:
        found["fit_monster"] = "{" + ",".join(call_arguments(text, ritual.end())[3 if ritual[1] else 4:]) + "}"
    if "AddMustBeSpecialSummonedByDarkFusion" in text:
        found.setdefault("dark_calling", "true")
    local = dict(re.findall(r"^(?:local\s+)?([A-Z_][A-Z0-9_]*)\s*=\s*(.*?)\s*(?:--.*)?$", text, re.M))
    lookup = lambda name: code if name == "id" else evaluate(local[name], lookup) if name in local else constants[name]
    metadata = {field: (value, parse(field, value, lookup)) for field, value in found.items() if field in FIELDS}
    return metadata, {field for field, call in SKIPPED.items() if call in text}


def rust_metadata(code, constants):
    lines = (OFFICIAL / f"c{code}.rs").read_text(encoding="utf-8").splitlines()
    code_only = "\n".join(line.split("//")[0] for line in lines)
    consts = dict(re.findall(r"^(?:pub )?const (\w+):[^=]*=\s*(.*?);", code_only, re.M | re.S))
    lookup = lambda name: evaluate(consts[name], lookup) if name in consts else constants[name]
    found = {field: consts.get(field.upper()) for field in FIELDS}
    for field, value in found.items():
        while value and value.lstrip("&") in consts:
            found[field] = value = consts[value.lstrip("&")]
    return {field: (" ".join(value.split()), parse(field, value, lookup)) for field, value in found.items() if value}


def read(field, metadata):
    default = None if field in NUMBERS else frozenset() if field in LISTS else False
    value = metadata.get(field if field in metadata else FALLBACK.get(field), (None, default))[1]
    # Card.ListsArchetype compares with ==, so a card code in listed_series (c31086840) can never match a setcode.
    return (frozenset(v for v in value if v <= 0xFFFF) if field == "listed_series" else value), default


def main():
    constants = {}
    for name, expression in lua_constants():
        if not expression.startswith("{"):
            constants[name] = constants[name.upper()] = evaluate(expression, constants.__getitem__)
    names = dict((code, name) for code, name in cdb("select id, name from texts"))
    wanted = [arg.lower() for arg in sys.argv[1:]] or [field for field in FIELDS if field != "fit_monster"]
    mismatches = {field: [] for field in wanted}
    for code in sorted(code for code in ported_codes() if lua_script(code).exists()):
        try:
            (lua, skipped), rust = lua_metadata(code, constants), rust_metadata(code, constants)
        except (KeyError, ValueError) as error:
            mismatches.setdefault("unresolved", []).append(f"unresolved {code} {names.get(code)}: {error!r}")
            continue
        for field in set(wanted) - skipped:
            (lua_value, default), (rust_value, _) = read(field, lua), read(field, rust)
            if lua_value != rust_value:
                kind = "missing" if rust_value == default else "extra" if lua_value == default else "different"
                shown = lambda side: " / ".join(side[name][0] for name in (field, FALLBACK.get(field)) if name in side) or "-"
                mismatches[field].append(f"{kind:9} {code:>9} {names.get(code)}: lua {shown(lua)}, rust {shown(rust)}")
    sys.stdout.reconfigure(encoding="utf-8")
    for field, lines in mismatches.items():
        print(f"{field.upper()}: {len(lines)}", *(f"  {line}" for line in lines), sep="\n")
    sys.exit(any(mismatches.values()))


main()
