// Weaver of Fairy Tails
use super::c56725612::{disfilter, disop};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 78021082;

pub const LISTED_NAMES: &[u32] = &[CARD_PRINCE_OF_FAIRIES];
pub const MATERIAL_SETCODE: &[u16] = &[SET_FAIRY_TAIL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_FAIRY_TAIL], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

// On the field and in the Extra Deck, Fusion Summoned with Polymerization from two Fairy Tail - Luna, with another Luna in
// the Deck and banishment to Special Summon and an opponent's Elfnote Lucina to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (86937530, HAND),
    (86937530, HAND),
    (86937530, DECK),
    (86937530, REMOVED),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_DISABLE);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2a.set_condition(duel, discon);
    e2a.set_target(duel, distg);
    e2a.set_operation(duel, disop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CHANGE_RACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_PRINCE_OF_FAIRIES));
    e3.set_value(duel, RACE_SPELLCASTER as i32);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FAIRY_TAIL])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_REMOVED,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_REMOVED,
        POS_FACEUP,
    )
}

fn disconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp)
        && c.is_attribute_except(duel, ATTRIBUTE_EARTH)
        && c.is_set_card(duel, &[SET_FAIRY_TAIL])
        && c.is_faceup(duel)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(eg) = a.eg else { return false };
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| disconfilter(c, duel, a.tp), 1, None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}
