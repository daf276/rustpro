// Dinowrestler Giga Spinosavate
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 58672736;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DINOSAUR, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// A copy on the field and one in the Extra Deck, Synchro Summoned with Jurrac Brachis (a Level 3 Dinosaur Tuner) and Garoozis
// (Level 5), with Hammer Shot in the opponent's hand to destroy it, the other monsters to replace the destruction, and a monster on the other side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (8594079, MZONE),
    (14977074, MZONE),
    (14977074, MZONE),
    (26412047, HAND | OPPONENT),
    (26412047, HAND | OPPONENT),
    (1761063, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    e1.set_condition(duel, actcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, destg);
    e3.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_target(duel, reptg);
    e4.set_operation(duel, repop);
    c.register_effect(duel, e4);
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) || duel.get_attack_target() == Some(c)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn repfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_destructable_full(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && !c.is_reason(duel, REASON_RULE)
            && c.is_on_field(duel)
            && c.is_faceup(duel)
            && duel.is_existing_matching_card(|tc, duel| repfilter(tc, duel, e), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, |tc, duel| repfilter(tc, duel, e), tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_target_card(g);
    if let Some(tc) = g.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    }
    Ok(true)
}

fn repop(duel: &mut Duel, _: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    if let Some(tc) = g.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
    }
    duel.destroy(g, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
