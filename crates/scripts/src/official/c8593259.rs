// Elemental HERO Clay Guardian
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 8593259;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ELEMENTAL_HERO], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_WARRIOR, Some(fc), sumtype, tp)),
    ];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Avian and Sparkman (Elemental HERO Warriors), with Elemental HERO
// monsters in the Deck and Sangan of the opponent to count.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (21844576, HAND),
    (20721928, HAND),
    (58932615, DECK),
    (58932615, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_target_range(duel, LOCATION_MZONE, 0);
    e2a.set_target_card(duel, |duel, e, c| {
        c.is_set_card(duel, &[SET_ELEMENTAL_HERO]) && c != e.get_handler(duel)
    });
    e2a.set_value(duel, 1);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2b);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ELEMENTAL_HERO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    let dam = 400 * duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) as i32;
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty()
            && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
            && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            let dam = 400 * duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) as u32;
            duel.break_effect()?;
            duel.damage(1 - tp, dam, REASON_EFFECT)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_HERO])
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
