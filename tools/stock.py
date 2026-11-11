"""Folds card functions that are exactly a `crate::stock` shape into a call of it, so ports stay written line by line.
A function matches when it equals the shape after its constants, numbers and filter are taken out and those satisfy
the shape's rule; anything else is left alone. Without --write it only counts.

  python tools/stock.py [--write]
"""
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

OFFICIAL = Path(__file__).resolve().parent.parent / "crates/scripts/src/official"
HEAD = re.compile(r"^(pub )?fn (\w+)\(")
ALIASES = [
    (r"let \(c, tp\) = \(a\.e\.get_handler\(duel\), a\.tp\); ", "let c = a.e.get_handler(duel); "),
    (r"let tp = a\.tp; ", ""),
    (r"let \(e, tp\) = \(a\.e, a\.tp\); ", ""),
    (r"let e = a\.e; ", ""),
    (r"\ba\.tp\b", "tp"),
    (r"\ba\.e\b", "e"),
]
CONST = r"[A-Z][A-Z0-9_]{2,}\b(?: [|+] [A-Z][A-Z0-9_]{2,}\b)*"
SLOT = re.compile(
    rf"\((?P<cast>{CONST})\) as i32|(?P<const>\b{CONST})(?P<ascast> as i32)?|(?P<number>\b\d+\b)"
    r"|(?P<closure>\|c, duel\| (?:super::)?(?:c\d+::)?\w+\(c, duel, e, tp\))|(?P<filter>\b(?:super::)?(?:c\d+::)?\w*filter\w*\b)"
)
TARGET = "fn _(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool>"
COST = "fn _(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool>"
OPERATION = "fn _(duel: &mut Duel, a: &Args) -> Script<()>"
EXISTS = "if chk == N { return Ok(duel.is_existing_matching_card(F, tp, K, N, N, None)); }"
SELECT = "duel.hint(K, tp, K); let g = duel.select_matching_card(tp, F, tp, K, N, N, N, None)?;"
FIRST_TARGET = "if let Some(tc) = duel.get_first_target() && tc.is_relate_to_effect(duel, e)"
HANDLER = "let c = e.get_handler(duel); if c.is_relate_to_effect(duel, e)"
SELF_SUMMON_CHECK = (
    "let c = e.get_handler(duel); if chk == N { return Ok(duel.get_location_count(tp, K) > N"
    " && c.is_can_be_special_summoned(duel, e, N, tp, false, false)); }"
)
CHAIN_INFO = "let p: u8 = duel.get_chain_info(N, K); let d: u32 = duel.get_chain_info(N, K);"
CHAIN_INFO_VALUES = ["0", "CHAININFO_TARGET_PLAYER", "0", "CHAININFO_TARGET_PARAM", "REASON_EFFECT"]
SELF_SUMMON_VALUES = ["0", "LOCATION_MZONE", "0", "0", "0", "CATEGORY_SPECIAL_SUMMON", "1"]
NEGATE_INFO = "duel.set_operation_info(N, K, a.eg, N, N, N);"


def generic(value):
    return value if re.fullmatch(r"\w+", value) else "{ " + value + " }"


def fixed(values, expected):
    return all(values[index] == value for index, value in expected.items())


def call(text, condition):
    return lambda v: ("call", text.format(*v)) if condition(v) else None


def name(text, condition):
    return lambda v: ("name", text.format(*map(generic, v))) if condition(v) else None


SELECT_ONE = {4: "0", 5: "1", 6: "1"}
COST_VALUES = {0: "0", 3: "0", 4: "1", 5: "HINT_SELECTMSG", 9: "0", 10: "1", 11: "1"}
SHAPES = {
    f"{TARGET} {{ {EXISTS} duel.set_operation_info(N, K, None, N, tp, K as i32); Ok(false) }}": call(
        "stock::exists_target(duel, a, chk, {1}, {2}, {6})",
        lambda v: fixed(v, {0: "0", 3: "0", 4: "1", 5: "0", 7: "1"}) and v[2] == v[8],
    ),
    f"{TARGET} {{ if chk == N {{ return Ok(duel.get_location_count(tp, K) > N && duel.is_existing_matching_card(F, tp, K, N, N, None)); }}"
    " duel.set_operation_info(N, K, None, N, tp, K as i32); Ok(false) }": call(
        "stock::special_summon_target(duel, a, chk, {3}, {4})",
        lambda v: fixed(v, {0: "0", 1: "LOCATION_MZONE", 2: "0", 5: "0", 6: "1", 7: "0", 8: "CATEGORY_SPECIAL_SUMMON", 9: "1"}) and v[4] == v[10],
    ),
    f"{OPERATION} {{ {SELECT} if !g.is_empty() {{ duel.send_to_hand(g, K, K)?; duel.confirm_cards(N - tp, g)?; }} Ok(()) }}": call(
        "stock::search(duel, a, {2}, {3})",
        lambda v: fixed(v, {0: "HINT_SELECTMSG", 1: "HINTMSG_ATOHAND", 7: "PLAYER_NONE", 8: "REASON_EFFECT", 9: "1", **SELECT_ONE}),
    ),
    f"{OPERATION} {{ if duel.get_location_count(tp, K) <= N {{ return Ok(()); }} {SELECT}"
    " if !g.is_empty() { duel.special_summon(g, N, tp, tp, false, false, K)?; } Ok(()) }": call(
        "stock::special_summon(duel, a, {4}, {5}, {10})",
        lambda v: fixed(v, {0: "LOCATION_MZONE", 1: "0", 2: "HINT_SELECTMSG", 3: "HINTMSG_SPSUMMON", 6: "0", 7: "1", 8: "1", 9: "0"}),
    ),
    f"{OPERATION} {{ {SELECT} if !g.is_empty() {{ duel.send_to_grave(g, K)?; }} Ok(()) }}": call(
        "stock::send_to_grave(duel, a, {2}, {3})",
        lambda v: fixed(v, {0: "HINT_SELECTMSG", 1: "HINTMSG_TOGRAVE", 7: "REASON_EFFECT", **SELECT_ONE}),
    ),
    f"{COST} {{ {EXISTS} {SELECT} duel.send_to_grave(g, K)?; Ok(false) }}": call(
        "stock::send_to_grave_cost(duel, a, chk, {1}, {2})",
        lambda v: fixed(v, {6: "HINTMSG_TOGRAVE", 12: "REASON_COST", **COST_VALUES}) and v[1] == v[7] and v[2] == v[8],
    ),
    f"{COST} {{ {EXISTS} {SELECT} duel.remove(g, K, K)?; Ok(false) }}": call(
        "stock::banish_cost(duel, a, chk, {1}, {2})",
        lambda v: fixed(v, {6: "HINTMSG_REMOVE", 12: "POS_FACEUP", 13: "REASON_COST", **COST_VALUES}) and v[1] == v[7] and v[2] == v[8],
    ),
    f"{OPERATION} {{ {HANDLER} {{ duel.special_summon(c, N, tp, tp, false, false, K)?; }} Ok(()) }}": name(
        "stock::special_summon_self::<{1}>", lambda v: v[0] == "0"
    ),
    f"{OPERATION} {{ {FIRST_TARGET} {{ duel.special_summon(tc, N, tp, tp, false, false, K)?; }} Ok(()) }}": name(
        "stock::special_summon_first_target::<{1}>", lambda v: v[0] == "0"
    ),
    f"{TARGET} {{ if chk == N {{ return Ok(duel.is_player_can_draw(tp, N)); }} duel.set_target_player(tp); duel.set_target_param(N);"
    " duel.set_operation_info(N, K, None, N, tp, N); Ok(false) }": name(
        "stock::draw_target::<{1}>", lambda v: fixed(v, {0: "0", 3: "0", 4: "CATEGORY_DRAW", 5: "0"}) and v[1] == v[2] == v[6]
    ),
    f"{TARGET} {{ if chk == N {{ return Ok(true); }} {NEGATE_INFO} if let Some(re) = a.re {{ let rc = re.get_handler(duel);"
    f" if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {{ {NEGATE_INFO} }} }} Ok(false) }}": name(
        "stock::negate_destroy_target", lambda v: v == ["0", "0", "CATEGORY_NEGATE", "1", "0", "0", "0", "CATEGORY_DESTROY", "1", "0", "0"]
    ),
    f"{TARGET} {{ if chk == N {{ return Ok(true); }} duel.set_operation_info(N, K, None, N, tp, K as i32); Ok(false) }}": name(
        "stock::operation_info_target::<{2}, {4}>", lambda v: fixed(v, {0: "0", 1: "0", 3: "1"})
    ),
    f"{OPERATION} {{ {FIRST_TARGET} {{ duel.destroy(tc, K)?; }} Ok(()) }}": name("stock::destroy_first_target", lambda v: v == ["REASON_EFFECT"]),
    f"{OPERATION} {{ {FIRST_TARGET} {{ duel.send_to_hand(tc, K, K)?; }} Ok(()) }}": name(
        "stock::first_target_to_hand", lambda v: v == ["PLAYER_NONE", "REASON_EFFECT"]
    ),
    f"{OPERATION} {{ {HANDLER} {{ duel.send_to_hand(c, K, K)?; }} Ok(()) }}": name("stock::self_to_hand", lambda v: v == ["PLAYER_NONE", "REASON_EFFECT"]),
    f"fn _(duel: &mut Duel, _: &Args) -> Script<()> {{ {CHAIN_INFO} duel.draw(p, d, K)?; Ok(()) }}": name("stock::draw", lambda v: v == CHAIN_INFO_VALUES),
    f"fn _(duel: &mut Duel, _: &Args) -> Script<()> {{ {CHAIN_INFO} duel.damage(p, d, K)?; Ok(()) }}": name(
        "stock::damage", lambda v: v == CHAIN_INFO_VALUES
    ),
    f"{TARGET} {{ {SELF_SUMMON_CHECK} duel.set_operation_info(N, K, Some(c.into()), N, N, N); Ok(false) }}": name(
        "stock::special_summon_self_target_player_0", lambda v: v == SELF_SUMMON_VALUES + ["0", "0"]
    ),
    f"{TARGET} {{ {SELF_SUMMON_CHECK} duel.set_operation_info(N, K, Some(c.into()), N, tp, N); Ok(false) }}": name(
        "stock::special_summon_self_target", lambda v: v == SELF_SUMMON_VALUES + ["0"]
    ),
}


def functions(lines):
    index = 0
    while index < len(lines):
        match = HEAD.match(lines[index])
        if not match:
            index += 1
            continue
        end = index
        while end < len(lines) and lines[end] != "}":
            end += 1
        yield index, end, bool(match.group(1)), match.group(2)
        index = end + 1


def shape(function, body):
    text = re.sub(r"\s+", " ", " ".join(body).replace(f"fn {function}(", "fn _(", 1)).replace("( ", "(").replace(", )", ")")
    for old, new in ALIASES:
        text = re.sub(old, new, text)
    values = []

    def slot(match):
        for group, mark in (("cast", "K as i32"), ("const", "K"), ("number", "N"), ("closure", "F"), ("filter", "F")):
            if match.group(group):
                values.append(match.group(group).replace("(c, duel, e, tp)", "(c, duel, a.e, a.tp)"))
                return "K as i32" if group == "const" and match.group("ascast") else mark
        return match.group(0)

    return SLOT.sub(slot, text), values


folded, changed = Counter(), []
for path in sorted(OFFICIAL.glob("c*.rs")):
    lines = path.read_text(encoding="utf-8").split("\n")
    text = "\n".join(lines)
    edits, renames = [], {}
    for start, end, public, function in functions(lines):
        template, values = shape(function, lines[start:end + 1])
        result = SHAPES[template](values) if template in SHAPES else None
        if not result:
            continue
        kind, replacement = result
        if kind == "call" and lines[start].endswith("{"):
            edits.append((start, end, [lines[start], f"    {replacement}", "}"]))
        elif kind == "name" and not public:
            # Only a function that is nothing but a setter's argument can go.
            setter = rf"\.set_(?:target|operation|cost)\(duel, {function}\)"
            if len(re.findall(rf"\b{function}\b", text)) - 1 != len(re.findall(setter, text)) or not re.search(setter, text):
                continue
            edits.append((start, end + (lines[end + 1:end + 2] == [""]), []))
            renames[function] = replacement
        else:
            continue
        folded[re.split(r"\(|::<", replacement)[0]] += 1
    if not edits:
        continue
    for start, end, new in sorted(edits, reverse=True):
        lines[start:end + 1] = new
    text = "\n".join(lines)
    for function, replacement in renames.items():
        text = re.sub(rf"(\.set_(?:target|operation|cost)\(duel, ){function}\)", lambda match: match.group(1) + replacement + ")", text)
    changed.append(path)
    if "--write" in sys.argv:
        path.write_text(re.sub(r"\n+$", "\n", text), encoding="utf-8", newline="\n")
print(f"{sum(folded.values())} functions in {len(changed)} files" + ("" if "--write" in sys.argv else " would be folded (--write to do it)"))
for helper, count in folded.most_common():
    print(f"  {count:4d}  {helper}")
if "--write" in sys.argv:
    # Windows limits a command line to 32k characters.
    for index in range(0, len(changed), 100):
        subprocess.run(["rustfmt", "--edition", "2024", *map(str, changed[index:index + 100])], check=True)
