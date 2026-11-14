// Borrelend Dragon
use engine::constants::MSG_ATTACK;

use super::c85289965::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 98630720;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
// Link Summoned in the Extra Deck with three Rokket Tracers (Effect Monsters), and a Rokket Loader in the GY to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68464358, MZONE),
    (68464358, MZONE),
    (68464358, MZONE),
    (25554552, GRAVE),
    (25554552, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_ATTACK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_value(duel, 1);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1c.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel)));
    c.register_effect(duel, e1c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn negatable_effect_monster(c: CardId, duel: &Duel) -> bool {
    c.is_effect_monster(duel) && c.is_negatable_monster(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(negatable_effect_monster, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g1 = duel.select_target(tp, negatable_effect_monster, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g2 = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, g1, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g2, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let field_tc = tg.filter(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None).get_first();
    if let Some(field_tc) = field_tc
        && field_tc.is_monster(duel)
        && field_tc.is_faceup(duel)
        && field_tc.is_can_be_disabled_by_effect(duel, e, true)
    {
        field_tc.negate_effects(duel, e.get_handler(duel), 0, false);
        duel.adjust_instantly(Some(field_tc));
        let gy_tc = tg.difference(field_tc.into());
        if field_tc.is_disabled(duel) && !gy_tc.is_empty() {
            duel.special_summon(gy_tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
