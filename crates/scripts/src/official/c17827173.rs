// Floowandereeze & Toccan
use super::c18940725::{cost, register_banish_effects, summon_winged_beast};
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED};

const ID: u32 = 17827173;

pub const LISTED_SERIES: &[u16] = &[SET_FLOOWANDEREEZE];

// Copies in hand to Normal Summon, banished Floowandereeze to add, and Robina in hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, REMOVED),
    (18940725, REMOVED),
    (80433039, REMOVED),
    (18940725, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost::<ID>);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    register_banish_effects::<ID>(duel, c);
}

fn addfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && addfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(addfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, addfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, LOCATION_REMOVED as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        if !tc.is_location(duel, LOCATION_HAND) {
            return Ok(());
        }
        summon_winged_beast::<ID>(duel, tp)?;
    }
    Ok(())
}
