// Vanquish Soul Rocks
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 77894049;
const ROCK_OF_THE_VANQUISHER: u32 = 28168628;

pub const LISTED_NAMES: &[u32] = &[ROCK_OF_THE_VANQUISHER];

// In the Extra Deck, Xyz Summoned with two of Razen (FIRE) and Dr. Mad Love (DARK), or with one of Hollie Sue (EARTH) after a battle,
// with the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (29302858, MZONE),
    (29302858, MZONE),
    (29280200, MZONE),
    (29280200, MZONE),
    (93156774, MZONE),
    (93156774, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_VANQUISH_SOUL], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0 && duel.get_flag_effect(0, ID + 1) > 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_condition(duel, attrcon::<ATTRIBUTE_DARK>);
    e1.set_value(duel, -800);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, attrcon::<ATTRIBUTE_FIRE>);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_VANQUISH_SOUL]));
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, attrcon::<ATTRIBUTE_EARTH>);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLED);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn regop(duel: &mut Duel, _: &Args) -> Script<()> {
    let (a, b) = duel.get_battle_monster(0);
    if [a, b].into_iter().flatten().any(|c| c.is_set_card(duel, &[SET_VANQUISH_SOUL])) {
        duel.register_flag_effect(0, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    (c.is_set_card_full(duel, &[SET_VANQUISH_SOUL], Some(lc), SUMMON_TYPE_XYZ as u64, tp) || c.is_code(duel, ROCK_OF_THE_VANQUISHER))
        && c.is_faceup(duel)
}

fn attrcon<const ATTR: u32>(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_attribute(duel, ATTR), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
