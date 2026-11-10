// Armed Dragon LV3
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 980973;
const ARMED_DRAGON_LV5: u32 = 46384672;

pub const LISTED_NAMES: &[u32] = &[ARMED_DRAGON_LV5];
// Armed Dragon LV5 in the Deck to level up to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ARMED_DRAGON_LV5, DECK), (ARMED_DRAGON_LV5, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ARMED_DRAGON_LV5) && c.is_can_be_special_summoned(duel, e, 0, tp, true, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(
            ft > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first() {
        duel.special_summon(tc, 0, tp, tp, true, true, POS_FACEUP)?;
        tc.complete_procedure(duel);
    }
    Ok(())
}
