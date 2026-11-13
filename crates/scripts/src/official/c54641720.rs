// Destiny HERO - Dreadnought Master
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 54641720;
const DREADMASTER: u32 = 40591390;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 2];
}

pub const LISTED_NAMES: &[u32] = &[DREADMASTER];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DESTINY_HERO, SET_HERO];

// In the Extra Deck, Special Summoned by Tributing Destiny HERO - Dreadmaster or Fusion Summoned by Polymerization with Dreadmaster and
// Dogma (Level 8), with Destiny HERO monsters in the Deck to add and in the Graveyard for its ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (DREADMASTER, MZONE),
    (DREADMASTER, HAND),
    (17132130, HAND),
    (24094653, HAND),
    (56570271, DECK),
    (56570271, DECK),
    (16605586, DECK),
    (63362460, GRAVE),
    (13093792, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e0a = duel.create_effect(c);
    e0a.set_description(duel, aux::stringid(ID, 0));
    e0a.set_type(duel, EFFECT_TYPE_FIELD);
    e0a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0a.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0a.set_range(duel, LOCATION_EXTRA);
    e0a.set_condition(duel, selfspcon);
    e0a.set_target(duel, selfsptg);
    e0a.set_operation(duel, selfspop);
    e0a.set_value(duel, 1);
    c.register_effect(duel, e0a);
    let e0b = duel.create_effect(c);
    e0b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0b.set_condition(duel, regcon);
    e0b.set_operation(duel, regop);
    c.register_effect(duel, e0b);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_SET_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| {
            let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DESTINY_HERO]));
            let g = duel.get_matching_group(filter, c.get_controler(duel), LOCATION_MZONE | LOCATION_GRAVE, 0, Some(c.into()));
            g.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel))) as i32
        }),
    );
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_above(duel, 5) && c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    c.is_summon_code(duel, Some(fc), MATERIAL_FUSION, tp, &[DREADMASTER])
        && c.is_can_be_fusion_material(duel, Some(fc), MATERIAL_FUSION)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), fc) > 0
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, |rc, duel| selfspcostfilter(rc, duel, tp, c), 1, release_options(c, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(
        tp,
        |rc, duel| selfspcostfilter(rc, duel, tp, c),
        (1, 1),
        true,
        release_options(c, tp),
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_fusion_summoned(duel) || c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        const SPECIAL_1: u32 = SUMMON_TYPE_SPECIAL + 1;
        c.is_original_code_rule(duel, ID) && (sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || sumtype & SPECIAL_1 == SPECIAL_1)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_monster(duel)) || c.lists_code_with_archetype(duel, &[SET_DESTINY_HERO]))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 2, 2, None)?;
    if g.len() == 2 {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
