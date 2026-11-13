// Witchcrafter Madame Rilliona
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 33475154;
const VERRE: u32 = 21522601;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(VERRE),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[VERRE];
pub const MATERIAL_SETCODE: &[u16] = &[SET_WITCHCRAFTER];

// In the Extra Deck and the Graveyard, Fusion Summoned by Polymerization with Witchcrafter Madame Verre, Genni and Schmietta, with
// Witchcrafter monsters of different Attributes in the Deck and Pot of Greed in the hand to reveal.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (24094653, HAND),
    (VERRE, HAND),
    (64756282, HAND),
    (21744288, HAND),
    (55144522, HAND),
    (64756282, DECK),
    (21744288, DECK),
    (84523092, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, witchsptg);
    e1.set_operation(duel, witchspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_cost(duel, |duel, a, chk| {
        Cost::reveal(duel, a, chk, |c, duel, _, _| c.is_spell(duel), false, (1, 1), None, LOCATION_HAND)
    });
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

fn witchspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 7) && c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn witchsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| witchspfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
    )
}

fn witchspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = duel.get_matching_group(|c, duel| witchspfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    if ft > 0 && !g.is_empty() {
        let attr_count = g.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel)));
        ft = ft.min(3).min(attr_count as i32);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let rescon = aux::dpcheck(|c, duel| i64::from(c.get_attribute(duel)));
        let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_fusion_monster(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon_self::<POS_FACEUP_DEFENSE>(duel, a)
}
