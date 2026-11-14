// Condolence Puppet
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, SZONE};

const ID: u32 = 57093995;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// Set, with Gimmick Puppets in the Deck to send, and in the Graveyard with Gigantes Doll (a Machine Xyz) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (29216967, DECK),
    (29216967, DECK),
    (92418590, DECK),
    (7593748, MZONE),
    (7593748, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, indtg);
    e2.set_operation(duel, indop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(|c, duel| c.is_summon_location(duel, LOCATION_EXTRA), tp, 0, LOCATION_MZONE, None) + 1;
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, ct, &aux::dncheck, tp, HINTMSG_TOGRAVE)?;
    duel.send_to_grave(sg, REASON_EFFECT)?;
    Ok(())
}

fn indfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_race(duel, RACE_MACHINE)
}

fn indtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && indfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(indfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, indfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3060);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| e.get_handler_player(duel) == 1 - rp));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
