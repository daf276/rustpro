// Crystron Rosenix
use super::c29838323::{register_synchro_limit, spfilter, summon_tuner};
use super::c65400671::token_stats;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 55326322;
const TOKEN_CRYSTRON: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_CRYSTRON];
pub const LISTED_SERIES: &[u16] = &[SET_CRYSTRON];
// Copies on the field and in the Graveyard, and Crystron Tuners in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (93665266, DECK),
    (20050865, DECK),
    (66938505, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(TOKEN_CRYSTRON),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tokentg);
    e2.set_operation(duel, tokenop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && desfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None)
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        summon_tuner(duel, e, tp)?;
    }
    let c = e.get_handler(duel);
    register_synchro_limit(duel, c, tp, aux::stringid(ID, 2));
    Ok(())
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, TOKEN_CRYSTRON, token_stats())
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(can_summon_token(duel, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !can_summon_token(duel, tp) {
        return Ok(());
    }
    if let Some(token) = duel.create_token(tp, TOKEN_CRYSTRON)
        && duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3303);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
        token.register_effect_full(duel, e2, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
