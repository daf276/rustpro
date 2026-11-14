// Evil HERO Neos Lord
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 13708888;

pub const LISTED_SERIES: &[u16] = &[SET_HERO, SET_EVIL_HERO];
pub const LISTED_NAMES: &[u32] = &[CARD_NEOS, CARD_DARK_FUSION];
pub const MATERIAL_SETCODE: &[u16] = &[SET_ELEMENTAL_HERO, SET_HERO, SET_NEOS];
pub const DARK_CALLING: bool = true;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Table(&[FusionMaterial::Code(CARD_NEOS), FusionMaterial::Filter(neosfusionmatfilter)]),
        FusionMaterial::Filter(effectmatfilter),
    ];
}

// In the Extra Deck and on the field, Special Summoned by Dark Fusion with Neos (or Nebula Neos, which mentions it) and Sangan,
// to take control of Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_DARK_FUSION, HAND),
    (CARD_NEOS, HAND),
    (40080312, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_special_summoned_by_dark_fusion(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, ctrltg);
    e1.set_operation(duel, ctrlop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(
        duel,
        EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP,
        EFFECT_FLAG2_CHECK_SIMULTANEOUS,
    );
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, ctrlcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e4);
}

fn neosfusionmatfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp) && c.lists_code(duel, CARD_NEOS)
}

fn effectmatfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_EFFECT, Some(fc), sumtype, tp) && c.is_on_field(duel)
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_controler_can_be_changed(duel)
            && chkc.is_faceup(duel));
    }
    let filter = aux::faceup_filter(CardId::is_controler_can_be_changed);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.get_control(tc, a.tp)?;
    }
    Ok(())
}

fn ctrlcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp) && c.is_monster(duel), 1, None))
}
