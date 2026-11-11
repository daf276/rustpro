// Destiny HERO - Dangerous
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 30757127;
const MALICIOUS: u32 = 9411399;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_DESTINY_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(ffilter),
    ];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Destiny HERO - Malicious and Sangan, to discard
// Sangan and send Malicious from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (MALICIOUS, HAND),
    (26202165, HAND),
    (26202165, HAND),
    (MALICIOUS, DECK),
    (MALICIOUS, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMING_END_PHASE);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, atkcost);
    e1.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_TOGRAVE, { LOCATION_HAND | LOCATION_DECK }>,
    );
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp) && c.is_type_full(duel, TYPE_EFFECT, Some(fc), sumtype, tp)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_discardable(duel) && duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, Some(c.into()))
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, |c, duel| cfilter(c, duel, tp), 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_faceup(duel)
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_monster(duel)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT)? != 0 && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        let tg = duel.get_matching_group(atkfilter, tp, LOCATION_MZONE, 0, None);
        if tg.is_empty() {
            return Ok(());
        }
        let ct = duel.get_matching_group_count(ctfilter, tp, LOCATION_GRAVE, 0, None) as i32;
        for tc in tg.iter() {
            let e1 = duel.create_effect(a.e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, ct * 200);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
