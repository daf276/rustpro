// Maiden of White
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 17947697;
const TRUE_LIGHT: u32 = 62089826;

pub const LISTED_NAMES: &[u32] = &[TRUE_LIGHT, CARD_BLUEEYES_W_DRAGON];

// True Light in the Deck to place, Blue-Eyes White Dragon in the hand and Graveyard to Special Summon and to Summon this card back
// from the Graveyard, and the opponent's Mage Power to target it and monster to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (TRUE_LIGHT, DECK),
    (TRUE_LIGHT, DECK),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (49161188, HAND | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfspcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_BECOME_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| a.eg.unwrap_or_default().is_contains(a.e.get_handler(duel)));
    e3.set_target(duel, gysptg);
    e3.set_operation(duel, gyspop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_BE_BATTLE_TARGET);
    c.register_effect(duel, e4);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, TRUE_LIGHT) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| plfilter(c, duel, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(|c, duel| plfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn selfspconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_summon_player(duel, tp) && c.is_faceup(duel)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| selfspconfilter(c, duel, a.tp), 1, None)
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_BLUEEYES_W_DRAGON)
        || (c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_type(duel, TYPE_TUNER)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| gyspfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}
