// Vermillion Dragon Mech
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 66698383;

// In the Extra Deck, Synchro Summoned with Ally Mind (a Level 5 Tuner) and Kojikocy (Level 4), with Ally Mind in the Graveyard to banish
// and already banished to add, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (40155554, MZONE),
    (40155554, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (40155554, GRAVE),
    (40155554, HAND),
    (53129443, HAND | OPPONENT),
    (40155554, REMOVED),
    (40155554, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE) || c.is_faceup(duel))
        && c.is_type(duel, TYPE_TUNER)
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, true))
        && duel.is_existing_target(
            aux::TRUE,
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(c.get_equip_group(duel).union(c.into())),
        )
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp), tp, location, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_synchro_summoned(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_TUNER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
