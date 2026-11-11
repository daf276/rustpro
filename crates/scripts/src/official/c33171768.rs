// Axon Kicker Oracle
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 33171768;
const SYNCHRO_XYZ_LINK: u32 = TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(psychic), FusionMaterial::Filter(synchro_xyz_link)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Mekk-Knight Avram (Psychic) and Link Disciple, with
// Emergency Teleport in the Graveyard to banish and the opponent's to Special Summon Psychic monsters (Chosen by the World
// Chalice) for it to negate, and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (84754430, MZONE),
    (84754430, MZONE),
    (32995276, MZONE),
    (32995276, MZONE),
    (67723438, GRAVE),
    (67723438, HAND | OPPONENT),
    (22916281, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(
        duel,
        Value::MaterialCheck(|duel, e, c| {
            let count = c
                .get_material(duel)
                .filter_count(duel, |c, duel| c.is_type(duel, SYNCHRO_XYZ_LINK), None);
            e.set_label(duel, count as i64);
        }),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_label_object(duel, Some(e0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_SPSUMMON);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.ep == 1 - a.tp && duel.get_current_chain() == 0);
    e2.set_target(duel, negsumtg);
    e2.set_operation(duel, negsumop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_fusion_summoned(duel) && c.is_previous_location(duel, LOCATION_MZONE)
    });
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn psychic(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_PSYCHIC, Some(fc), sumtype, tp)
}

fn synchro_xyz_link(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, SYNCHRO_XYZ_LINK, Some(fc), sumtype, tp)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(e0) = a.e.get_label_object(duel) else { return Ok(()) };
    let amount = e0.get_label(duel) as i32 * 1000;
    a.e.get_handler(duel)
        .update_attack(duel, amount, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, Some(2));
    Ok(())
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, CardId::is_able_to_remove, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(eg), eg.len() as u8, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(eg), eg.len() as u8, a.tp, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELEPORT]) && (c.is_normal_spell(duel) || c.is_quick_play_spell(duel)) && c.is_able_to_remove_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spcostfilter, LOCATION_GRAVE)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}
