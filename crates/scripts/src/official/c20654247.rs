// Blue-Eyes Chaos Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 20654247;
const CHAOS_FORM: u32 = 21082832;

pub const LISTED_NAMES: &[u32] = &[CHAOS_FORM, CARD_BLUEEYES_W_DRAGON];

// In the hand, Ritual Summoned by Chaos Form with Blue-Eyes White Dragon from the Graveyard, and the opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (CHAOS_FORM, HAND),
    (CHAOS_FORM, HAND),
    (CHAOS_FORM, DECK),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::ritlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(indval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_POSITION + CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_condition(duel, poscon);
    e4.set_target(duel, postg);
    e4.set_operation(duel, posop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_MATERIAL_CHECK);
    e5.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e5);
}

fn indval(duel: &Duel, e: EffectId, _: EffectId, tp: u8) -> bool {
    tp != e.get_handler_player(duel)
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_ritual_summoned(duel) && c.get_flag_effect(duel, ID) != 0
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tg = duel.get_matching_group(CardId::is_can_change_position, tp, 0, LOCATION_MZONE, None);
    if !tg.is_empty()
        && duel.change_position_full(
            tg,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )? > 0
    {
        let og = duel.get_operated_group().filter(duel, CardId::is_faceup, None);
        for tc in og {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_value(duel, 0);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
            e2.set_value(duel, 0);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
        }
    }
    if c.is_relate_to_effect(duel, e) {
        let e3 = duel.create_effect(c);
        e3.set_description(duel, 3208);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_PIERCE);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e3);
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, CARD_BLUEEYES_W_DRAGON), 1, None)
    {
        c.register_flag_effect(
            duel,
            ID,
            RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE)),
            0,
            1,
        );
    }
}
