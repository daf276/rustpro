// Harpie Conductor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 85696777;

// Link Summoned from two WIND monsters, with Spell/Trap cards to destroy in its place and Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (76812113, MZONE),
    (76812113, MZONE),
    (76812113, MZONE),
    (22359980, SZONE),
    (22359980, SZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (32295838, MZONE | OPPONENT),
    (ID, MZONE),
    (ID, MZONE),
];
// The bounce trigger needs a Harpie returned from the field to the hand, which no ported card does.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_SELECT_EFFECTYN),
    Check::Report(&Check::Replaced),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WIND, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e0.set_code(duel, EFFECT_CHANGE_CODE);
    e0.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e0.set_value(duel, CARD_HARPIE_LADY as i32);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, reptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e1.set_operation(duel, repop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcond);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_ONFIELD)
        && c.is_set_card(duel, &[SET_HARPIE])
        && c.is_monster(duel)
        && (c.is_reason(duel, REASON_BATTLE) || c.is_reason(duel, REASON_EFFECT))
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_spell_trap(duel)
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED + STATUS_BATTLE_DESTROYED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    if duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        let g = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, e, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        e.set_label_object_group(duel, Some(g));
        if let Some(tc) = g.get_first() {
            tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
        }
        return Ok(true);
    }
    Ok(false)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}

fn rtfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_set_card(duel, &[SET_HARPIE])
        && c.is_previous_controler(duel, tp)
        && c.is_controler(duel, tp)
        && c != e.get_handler(duel)
}

fn thcond(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| rtfilter(c, duel, e, tp), 1, None)
}

fn rttgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel) && c.is_special_summoned(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rttgfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, rttgfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
