// Elemental HERO Flame Wingman - Infernal Rage
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 93347961;
pub const MATERIAL_SETCODE: &[u16] = &[SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Sparkman and Clayman (Normal Monsters), which
// unlocks the Tribute effect to Special Summon Gaia from the Extra Deck, and Favorite Contact in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (20721928, HAND),
    (84327329, HAND),
    (16304628, EXTRA),
    (75047173, DECK),
    (75047173, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
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
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_fusion_summoned(duel) && c.has_flag_effect(duel, ID)
    });
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e3);
}

pub fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let attr = c.get_attribute(duel);
    c.is_set_card_full(duel, &[SET_ELEMENTAL_HERO], Some(fc), sumtype, tp)
        && attr > 0
        && sg.is_none_or(|sg| !sg.is_exists(duel, |mc, duel| fusfilter(mc, duel, attr, fc, sumtype, tp), 1, Some(c.into())))
}

fn fusfilter(c: CardId, duel: &Duel, attr: u32, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, attr, Some(fc), sumtype, tp) && c.is_has_effect(duel, 511002961).is_none()
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FAVORITE]) && c.is_able_to_hand(duel)
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

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: CardId) -> bool {
    c.is_set_card(duel, &[SET_ELEMENTAL_HERO])
        && c.is_level_below(duel, 7)
        && !c.is_summonable_card(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
        && ((c.is_location(duel, LOCATION_DECK) && duel.get_mzone_count(tp, Some(ec.into())) > 0)
            || (c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, Some(ec.into()), c) > 0))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ec = e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, ec),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ec = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, ec),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_type(duel, TYPE_NORMAL), 1, None)
    {
        c.register_flag_effect(
            duel,
            ID,
            RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_TEMP_REMOVE | RESET_LEAVE)),
            0,
            1,
        );
    }
}
