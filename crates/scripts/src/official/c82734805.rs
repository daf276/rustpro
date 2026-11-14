// Infernoid Tierra
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 82734805;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];
pub const MATERIAL_SETCODE: &[u16] = &[SET_INFERNOID];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_INFERNOID], Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(14799437), FusionMaterial::Code(23440231)];
}

// Fusion Summoned by Polymerization from Infernoid Onuncu, Devyaty and other Infernoids of different names.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (14799437, HAND),
    (23440231, HAND),
    (58446973, HAND),
    (60551528, MZONE),
    (58446973, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
    });
    e2.set_target(duel, tg);
    e2.set_operation(duel, op);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn valcheck(duel: &mut Duel, e: EffectId, _: CardId) {
    let hc = e.get_handler(duel);
    let player = hc.get_controler(duel);
    let ct = hc.get_material(duel).get_class_count(duel, |c, duel| {
        let codes = c.get_codes_full(duel, Some(hc), u64::from(SUMMON_TYPE_FUSION), player);
        codes.first().copied().map_or(0, i64::from)
    });
    if let Some(le) = e.get_label_object(duel) {
        le.set_label(duel, ct as i64);
    }
}

fn able_to_grave(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_grave(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = a.e.get_label(duel);
    let con3 = ct >= 3
        && duel.is_existing_matching_card(able_to_grave, tp, LOCATION_EXTRA, 0, 3, None)
        && duel.is_existing_matching_card(able_to_grave, tp, 0, LOCATION_EXTRA, 3, None);
    let con5 = ct >= 5 && duel.is_player_can_discard_deck(tp, 3) && duel.is_player_can_discard_deck(1 - tp, 3);
    let con8 = ct >= 8
        && duel.is_existing_matching_card(|_, _| true, tp, LOCATION_REMOVED, 0, 1, None)
        && duel.is_existing_matching_card(|_, _| true, tp, 0, LOCATION_REMOVED, 1, None);
    let con10 = ct >= 10 && duel.get_field_group_count(tp, LOCATION_HAND, LOCATION_HAND) > 0;
    if chk == 0 {
        return Ok(con3 || con5 || con8 || con10);
    }
    if con5 {
        duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, PLAYER_ALL, 3);
    }
    Ok(false)
}

fn select_three_from_extra(duel: &mut Duel, tp: u8, location1: u32, location2: u32, selector: u8) -> Script<CardSet> {
    let g = duel.get_matching_group(able_to_grave, tp, location1, location2, None);
    if g.len() < 3 {
        return Ok(g);
    }
    duel.hint(HINT_SELECTMSG, selector, HINTMSG_TOGRAVE);
    g.select(duel, selector, 3, 3, None)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_label(duel);
    if ct >= 3 {
        let mut sg1 = select_three_from_extra(duel, tp, LOCATION_EXTRA, 0, tp)?;
        let sg2 = select_three_from_extra(duel, tp, 0, LOCATION_EXTRA, 1 - tp)?;
        sg1.merge(sg2);
        if !sg1.is_empty() {
            duel.send_to_grave(sg1, REASON_EFFECT)?;
        }
    }
    if ct >= 5 {
        duel.break_effect()?;
        duel.discard_deck(tp, 3, REASON_EFFECT)?;
        duel.discard_deck(1 - tp, 3, REASON_EFFECT)?;
    }
    if ct >= 8 {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let mut g1 = duel.select_matching_card(tp, |_, _| true, tp, LOCATION_REMOVED, 0, 1, 3, None)?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
        let g2 = duel.select_matching_card(1 - tp, |_, _| true, 1 - tp, LOCATION_REMOVED, 0, 1, 3, None)?;
        g1.merge(g2);
        if !g1.is_empty() {
            duel.send_to_grave(g1, REASON_EFFECT | REASON_RETURN)?;
        }
    }
    if ct >= 10 {
        duel.break_effect()?;
        let g1 = duel.get_field_group(tp, LOCATION_HAND, LOCATION_HAND);
        duel.send_to_grave(g1, REASON_EFFECT)?;
    }
    Ok(())
}
