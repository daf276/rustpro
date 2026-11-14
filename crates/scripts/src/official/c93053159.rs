// Alba System Dogmatikalamity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 93053159;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::GroupFilter(matfilter),
        FusionMaterial::GroupFilter(matfilter),
        FusionMaterial::GroupFilter(matfilter),
        FusionMaterial::GroupFilter(matfilter),
        FusionMaterial::GroupFilter(matfilter),
        FusionMaterial::GroupFilter(matfilter),
    ];
}

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
// On the field with copies in the Extra Deck to pay its attack cost; its Fusion Summon from six Graveyard monsters, which
// its last effect needs, no scenario reaches.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (ID, EXTRA)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ATTACK_ALL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_COST);
    e2.set_cost_condition(duel, |duel, _, _, tp, _| {
        duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_EXTRA, 0, 1, None)
    });
    e2.set_operation(duel, atkcostop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, tgcon);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let code = c.get_codes_full(duel, Some(fc), sumtype, tp).into_iter().min().unwrap_or(0);
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_controler(duel, tp)
        && sg.is_none_or(|sg| !sg.is_exists(duel, |mc, duel| fusfilter(mc, duel, code, fc, sumtype, tp), 1, Some(c.into())))
}

fn fusfilter(c: CardId, duel: &Duel, code: u32, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_summon_code(duel, Some(fc), sumtype, tp, &[code]) && c.is_has_effect(duel, 511002961).is_none()
}

fn atkcostop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.is_attack_cost_paid() != 2 && c.is_location(duel, LOCATION_MZONE) {
        let g = duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_EXTRA, 0, None);
        let finishcon = |duel: &Duel, _: CardSet, _: CardSet, _: Option<CardId>| (duel.is_attack_cost_paid() == 0, false);
        let sg = aux::select_unselect_group_full(
            duel,
            g,
            0,
            1,
            &|_, _, _, _| (true, false),
            tp,
            HINTMSG_TOGRAVE,
            Some(&finishcon),
            false,
        )?;
        if sg.len() == 1 {
            duel.send_to_grave(sg, REASON_COST)?;
            duel.attack_cost_paid();
        } else {
            duel.attack_cost_paid_full(2);
        }
    }
    Ok(())
}

fn tgconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION) && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    if !a.e.get_handler(duel).is_fusion_summoned(duel) {
        return false;
    }
    let g = duel.get_matching_group(tgconfilter, a.tp, LOCATION_GRAVE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 6
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_field_group(a.tp, LOCATION_EXTRA, LOCATION_EXTRA);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, a.tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_EXTRA, LOCATION_EXTRA);
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
