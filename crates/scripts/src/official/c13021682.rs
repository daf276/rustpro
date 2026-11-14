// Starjunk Synchron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 13021682;

pub const LISTED_SERIES: &[u16] = &[SET_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_WARRIOR];

// Copies in the hand and Graveyard, Doron (Level 2) in the Graveyard to Special Summon, and Junk Warrior to protect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (756652, GRAVE),
    (756652, GRAVE),
    (CARD_JUNK_WARRIOR, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, 20932152);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 2) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, None) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel
        .select_target(tp, |tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && tc.is_relate_to_effect(duel, e)
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        tc.negate_effects(duel, c, 0, false);
    }
    duel.special_summon_complete()?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_ONFIELD, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_JUNK_WARRIOR) && c.is_faceup(duel));
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
