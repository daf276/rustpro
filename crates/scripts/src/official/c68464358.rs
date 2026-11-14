// Rokket Tracer
use super::c25554552::register_extra_deck_limit;
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 68464358;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field, with Rokket Loader in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (25554552, DECK), (25554552, DECK), (25554552, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(25554552)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn desfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && (ft > 0 || (c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && desfilter(chkc, duel, ft));
    }
    if chk == 0 {
        return Ok(ft > -1
            && duel.is_existing_target(|c, duel| desfilter(c, duel, ft), tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, ft), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? != 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    register_extra_deck_limit(duel, e.get_handler(duel), tp, aux::stringid(ID, 1));
    Ok(())
}
