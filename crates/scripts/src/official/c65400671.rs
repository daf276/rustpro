// Crystron Halqifibrax Prism
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 65400671;
const TOKEN_CRYSTRON: u32 = 55326323;

pub const LISTED_NAMES: &[u32] = &[TOKEN_CRYSTRON];

// On the field for the tokens, and Link Summoned from Crystron Tuners (Quan, Citree, Rion), with Crystron Quandax in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (93665266, MZONE),
    (20050865, MZONE),
    (66938505, MZONE),
    (66938505, MZONE),
    (39964797, EXTRA),
    (39964797, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_TUNER, Some(lc), sumtype, tp), 1, None)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let opp = 1 - a.tp;
        duel.is_turn_player(opp) && (duel.is_main_phase() || duel.is_battle_phase())
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tokentg);
    e2.set_operation(duel, tokenop);
    e2.set_hint_timing(
        duel,
        0,
        TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER,
    );
    c.register_effect(duel, e2);
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let sum = c
        .get_material(duel)
        .get_sum(duel, |mc, duel| i64::from(mc.get_original_level(duel)));
    e.set_label(duel, sum);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_type(duel, TYPE_TUNER)
        && c.is_level_below(duel, lv)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn material_level(duel: &Duel, e: EffectId) -> u32 {
    e.get_label_object(duel).map_or(0, |e0| e0.get_label(duel)) as u32
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let lv = material_level(duel, e);
        return Ok(lv > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let lv = material_level(duel, e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

pub fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_CRYSTRON, TYPES_TOKEN, 0, 0, 1, RACE_MACHINE, ATTRIBUTE_WATER)
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) >= 3
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_player_can_special_summon_monster(tp, TOKEN_CRYSTRON, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 3, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, 0);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !(duel.get_location_count(tp, LOCATION_MZONE) >= 3
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.is_player_can_special_summon_monster(tp, TOKEN_CRYSTRON, token_stats()))
    {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for _ in 0..3 {
        let Some(token) = duel.create_token(tp, TOKEN_CRYSTRON) else {
            continue;
        };
        if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(c);
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
    }
    duel.special_summon_complete()?;
    Ok(())
}
