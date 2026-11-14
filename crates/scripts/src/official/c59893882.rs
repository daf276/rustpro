// Evil HERO Inferno Wing - Backfire
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 59893882;
const AVIAN: u32 = 21844576;
const BURSTINATRIX: u32 = 58932615;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Table(&[FusionMaterial::Code(AVIAN), FusionMaterial::Code(BURSTINATRIX)]),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_HERO], Some(fc), sumtype, tp)),
    ];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];
pub const LISTED_NAMES: &[u32] = &[AVIAN, BURSTINATRIX, CARD_DARK_FUSION];
pub const LISTED_SERIES: &[u16] = &[SET_HERO];
pub const DARK_CALLING: bool = true;

// In the Extra Deck, Special Summoned by Dark Fusion with Avian and Burstinatrix, with Dark Fusion in the Deck to add and the
// opponent's weak Sangan to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_DARK_FUSION, HAND),
    (AVIAN, HAND),
    (BURSTINATRIX, MZONE),
    (BURSTINATRIX, MZONE),
    (CARD_DARK_FUSION, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_special_summoned_by_dark_fusion(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_DARK_FUSION) || c.lists_code(duel, CARD_DARK_FUSION)) && !c.is_type(duel, TYPE_FUSION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let location = LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, location as i32);
    Ok(false)
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

fn damconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if !c.is_status(duel, STATUS_OPPO_BATTLE) {
        return false;
    }
    if c.is_relate_to_battle(duel) {
        c.is_set_card(duel, &[SET_HERO]) && c.is_controler(duel, tp)
    } else {
        c.is_previous_set_card(duel, &[SET_HERO]) && c.is_previous_controler(duel, tp)
    }
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| damconfilter(c, duel, tp), 1, None))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(2100);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 2100);
    Ok(false)
}
