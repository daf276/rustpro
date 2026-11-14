// Traptrix Sera
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 73639099;

pub const LISTED_SERIES: &[u16] = &[SET_TRAPTRIX, SET_TRAP_HOLE, SET_HOLE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRAPTRIX], Some(lc), sumtype, tp) && !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// Two copies on the field (each one's Traptrix effect lets the other set), Traptrix Atrax to activate Trap Hole from the hand, and
// Trap Hole and Waboku (Normal Traps) in the hand, set, and in the Deck against the opponent's summons and attacks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (55428242, MZONE),
    (55428242, MZONE),
    (55428242, DECK),
    (55428242, DECK),
    (4206964, HAND),
    (4206964, HAND),
    (4206964, SZONE),
    (4206964, SZONE),
    (4206964, DECK),
    (4206964, DECK),
    (12607053, SZONE),
    (12607053, SZONE),
    (12607053, SZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::Effect(|duel, _, te| te.is_trap_effect(duel)));
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SET);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_CHAINING);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_condition(duel, setcon);
    e5.set_target(duel, settg);
    e5.set_operation(duel, setop);
    c.register_effect(duel, e5);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.get_active_type(duel) == TYPE_TRAP && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn namefilter(c: CardId, duel: &Duel, cd: u32) -> bool {
    c.is_code(duel, cd) && c.is_faceup(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TRAPTRIX])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !duel.is_existing_matching_card(|tc, duel| namefilter(tc, duel, c.get_code(duel)), tp, LOCATION_ONFIELD, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    a.rp == a.tp && re.is_monster_effect(duel) && rc.is_set_card(duel, &[SET_TRAPTRIX]) && rc != a.e.get_handler(duel)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRAP_HOLE, SET_HOLE]) && c.is_normal_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.first() {
        duel.sset(tp, tc)?;
    }
    Ok(())
}
