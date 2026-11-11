// Shiranui Spectralsword Shade
use crate::prelude::*;
use crate::testing::{Check, MZONE, REMOVED};

const ID: u32 = 79783880;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Shiranui Squire and Zombie Normal Monsters banished, to Special Summon from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (41562624, REMOVED),
    (41562624, REMOVED),
    (32274490, REMOVED),
    (32274490, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, mg, c| {
        let ok = aux::chkf_mmz(tp, 1)(duel, sg, mg, c).0 && sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_SHIRANUI]), 1, None);
        (ok, false)
    }
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| filter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &rescon(tp)));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon(tp), tp, 0)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), tg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = g.filter(duel, |tc, duel| tc.is_relate_to_effect(duel, e), None);
    if sg.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    for tc in sg {
        if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
            for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, code);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect_full(duel, e1, true);
            }
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    !c.is_race(duel, RACE_ZOMBIE)
}
