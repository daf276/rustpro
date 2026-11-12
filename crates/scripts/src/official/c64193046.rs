// Superheavy Samurai Brave Masurawo
use super::c90587641::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64193046;

// A copy and Big Benkei on the field for the opponent's Smashing Ground and Hammer Shot to destroy, and a copy in the Extra
// Deck, Synchro Summoned from Draluvice (a Level 7 Tuner) and Shapesnatch (Level 5).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (3117804, MZONE),
    (11212437, MZONE),
    (4035199, MZONE),
    (97169186, HAND | OPPONENT),
    (26412047, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
    });
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drawop);
    c.register_effect(duel, e3);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI])
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI])
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    if duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
        let g = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        let Some(tc) = g.get_first() else { return Ok(false) };
        duel.hint_selection_full(tc, true);
        e.set_label_object_group(duel, Some(tc.into()));
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
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

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let h = duel.get_field_group_count(tp, LOCATION_HAND, 0) as u32;
    if chk == 0 {
        return Ok(h < 3 && duel.is_player_can_draw(tp, (3 - h) as usize));
    }
    duel.set_target_player(tp);
    duel.set_target_param(3 - h);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 3 - h as i32);
    Ok(false)
}

fn drawop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let h = duel.get_field_group_count(p, LOCATION_HAND, 0) as u32;
    if h >= 3 {
        return Ok(());
    }
    duel.draw(p, 3 - h, REASON_EFFECT)?;
    Ok(())
}
