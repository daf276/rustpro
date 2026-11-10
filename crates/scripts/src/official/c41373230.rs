// Titaniklad the Ash Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 41373230;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_attack_above(duel, 2500)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Fallen of Albaz and Spiral Serpent, with Albaz in the Deck and
// the opponent's Raigeki and Dark Hole to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_ALBAZ, MZONE),
    (32626733, MZONE),
    (24094653, HAND),
    (24094653, HAND),
    (CARD_ALBAZ, DECK),
    (CARD_ALBAZ, DECK),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.get_turn_id(duel) == duel.get_turn_count() && !c.is_reason(duel, REASON_RETURN)
    });
    e3.set_target(duel, thsptg);
    e3.set_operation(duel, thspop);
    c.register_effect(duel, e3);
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let atk = c.get_material(duel).get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) * 100;
    if atk == 0 {
        return;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_fusion_summoned(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(immval));
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let tc = te.get_handler(duel);
    let trig_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    let trig_sum_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_SUMMON_LOCATION);
    if !(te.is_activated(duel) && te.is_monster_effect(duel) && trig_loc == LOCATION_MZONE && tc != e.get_handler(duel)) {
        return false;
    }
    if !duel.is_chain_solving() || (tc.is_relate_to_effect(duel, te) && tc.is_faceup(duel) && tc.is_location(duel, trig_loc)) {
        tc.is_summon_location(duel, LOCATION_EXTRA)
    } else {
        trig_sum_loc & LOCATION_EXTRA > 0
    }
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    (c.is_set_card(duel, &[SET_DOGMATIKA]) || c.is_code(duel, CARD_ALBAZ))
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, mmz_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let Some(sc) = duel
        .select_matching_card(tp, |c, duel| thspfilter(c, duel, e, tp, mmz_chk), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |_, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}
