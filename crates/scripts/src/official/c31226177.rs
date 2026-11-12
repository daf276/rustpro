// Imduk the World Chalice Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 31226177;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_NORMAL, Some(lc), sumtype, tp) && !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with a Normal Monster (Digitron), with Lee the World Chalice Fairy in the hand to Special Summon,
// and the opponent's Sangans to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (32295838, MZONE),
    (32295838, MZONE),
    (21893603, HAND),
    (21893603, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// The battle destruction is reported: it needs a battle against the monster in the one zone it points to (1 in 64).
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_WORLD_CHALICE]));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| c.get_linked_group(duel).contains(bc)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(bc) = c.get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn spcon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WORLD_CHALICE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

pub fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

pub fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
