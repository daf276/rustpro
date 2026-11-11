// Hecahands Jauzah
use super::c95132593::add_indestructable_battle;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 67021206;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_HECAHANDS], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_ILLUSION, Some(fc), sumtype, tp)),
    ];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_HECAHANDS];

// In the Extra Deck, Fusion Summoned by Polymerization with Hecahands Gaigas and Breus (Illusion), with Hecahands Dandalos to
// take a monster of the opponent's, which Jauzah can then Release as a cost to Special Summon itself, and Hecahands cards to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (95132593, HAND),
    (21637502, HAND),
    (31411835, MZONE),
    (95132593, MZONE),
    (26202165, MZONE | OPPONENT),
    (95132593, DECK),
    (21637502, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    add_indestructable_battle(duel, c);
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    (c.is_race(duel, RACE_ILLUSION) || (c.is_faceup(duel) && c.is_owner(duel, 1 - tp)))
        && c.is_releasable(duel)
        && c.is_can_be_fusion_material(duel, Some(fc), MATERIAL_FUSION)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8, handler: CardId) -> (bool, bool) {
    (
        duel.get_location_count_from_ex(tp, tp, Some(sg), handler) > 0
            && sg.is_exists(duel, |c, duel| c.is_race(duel, RACE_ILLUSION), 1, None)
            && sg.is_exists(duel, |c, duel| c.is_faceup(duel) && c.is_owner(duel, 1 - tp), 1, None),
        false,
    )
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let mg = duel.get_matching_group(|mc, duel| selfspcostfilter(mc, duel, tp, c), tp, LOCATION_MZONE, 0, None);
    mg.len() >= 2 && aux::select_unselect_group_check(duel, mg, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp, a.e.get_handler(duel)))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let mg = duel.get_matching_group(|mc, duel| selfspcostfilter(mc, duel, tp, c), tp, LOCATION_MZONE, 0, None);
    let handler = a.e.get_handler(duel);
    let g = aux::select_unselect_group_full(
        duel,
        mg,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, tp, handler),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST | REASON_MATERIAL)?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HECAHANDS]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
