// The Weather Painter Rainbow
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 54178659;

pub const LISTED_SERIES: &[u16] = &[SET_THE_WEATHER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_THE_WEATHER], Some(lc), sumtype, tp)
    }
}

// Copies on the field to grant each other the negation and banish themselves as its cost, and Link Summoned from three The Weather
// Painter Rain, with Pot of Greed and Elfnote Lucina on the opponent's side to chain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (92411493, MZONE),
    (92411493, MZONE),
    (92411493, MZONE),
    (92411493, MZONE),
    (13597785, MZONE | OPPONENT),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesGranted(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_SPSUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_operation(duel, spreg::<ID>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_condition(duel, spcon::<ID>);
    e3.set_target(duel, sptg::<ID>);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, discon2);
    e4.set_cost(duel, announcecost);
    e4.set_target(duel, stock::negate_destroy_target);
    e4.set_operation(duel, disop2);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_MZONE, 0);
    e5.set_target_card(duel, |duel, e, c| {
        c.is_type(duel, TYPE_EFFECT) && c.is_set_card(duel, &[SET_THE_WEATHER]) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.tp != a.ep && duel.get_current_chain() == 0 && a.e.get_handler(duel).is_link_summoned(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

// Shared with The Weather Painter Rain, which writes the same functions with its own code as the flag.
pub fn spreg<const FLAG: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_COST) && re.get_handler(duel).is_set_card(duel, &[SET_THE_WEATHER]) {
        let turn = i64::from(duel.get_turn_count()) + 1;
        a.e.set_label(duel, turn);
        c.register_flag_effect(duel, FLAG, RESETS_STANDARD_PHASE_END, 0, 2);
    }
    Ok(())
}

pub fn spcon<const FLAG: u32>(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel)
        .is_some_and(|object| object.get_label(duel) == i64::from(duel.get_turn_count()))
        && a.e.get_handler(duel).get_flag_effect(duel, FLAG) > 0
}

pub fn sptg<const FLAG: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, FLAG);
    Ok(false)
}

fn discon2(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
}

fn announcecost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Cost::self_banish(duel, a, chk);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Cost::self_banish(duel, a, chk)
}

fn disop2(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
