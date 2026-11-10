// Granguignol the Dusk Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 24915933;
const BLAZING_CARTESIA: u32 = 95515789;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(BLAZING_CARTESIA),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(fc), sumtype, tp)
        }),
    ];
}

pub const LISTED_NAMES: &[u32] = &[BLAZING_CARTESIA];

// Cartesia and Feral Imp (DARK) with Polymerization to Fusion Summon it, a copy in the Graveyard and on the field for the
// Special Summon, which the opponent's Cartesia triggers; Dogmatika Ecclesia in the Deck and Despian Luluwalilith in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (BLAZING_CARTESIA, HAND),
    (41392891, HAND),
    (BLAZING_CARTESIA, HAND | OPPONENT),
    (60303688, DECK),
    (60303688, DECK),
    (53971455, EXTRA),
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
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 6) && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let trig_p: u8 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_PLAYER);
    a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel)) && trig_p == 1 - a.tp
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: Option<CardId>) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && if c.is_location(duel, LOCATION_DECK) {
            c.is_set_card(duel, &[SET_DOGMATIKA]) && duel.get_mzone_count(tp, ec.map(CardSet::from)) > 0
        } else {
            c.is_set_card(duel, &[SET_DESPIA]) && duel.get_location_count_from_ex(tp, tp, ec.map(CardSet::from), c) > 0
        }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ec = Some(e.get_handler(duel));
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
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    // The Lua passes no excluded card to the filter here.
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, None),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
