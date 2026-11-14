// Kerolia the Frolicsome
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 101402085;

// In the hand, Graveyard and on the field, with Spatial Trunade to return Set Mirror Forces, and a banished one to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (2729965, HAND),
    (2729965, HAND),
    (44095762, SZONE),
    (44095762, SZONE | OPPONENT),
    (44095762, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_previous_location(duel, LOCATION_ONFIELD), 1, None)
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_type_on_field(duel, TYPE_SPELL | TYPE_TRAP) && c.is_spell_trap(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, spconfilter, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_DEFENSE)? > 0 {
        if c.is_facedown(duel) {
            duel.confirm_cards(1 - tp, c)?;
        }
        if c.is_summon_location(duel, LOCATION_GRAVE) {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3300);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
            e1.set_value(duel, LOCATION_REMOVED as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
            c.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_ssetable(duel) && c.is_faceup(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(setfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SET, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.sset(a.tp, tc)?;
    }
    Ok(())
}
