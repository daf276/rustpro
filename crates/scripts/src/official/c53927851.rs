// Radiant Typhoon Varuroon, the Vibrant Vortex
use super::c16922142::{quickplaycon, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 53927851;

pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// A copy in the hand and one in the GY, Mystical Space Typhoon (a Quick-Play Spell) in the hand and in the GY twice for the
// destruction, and an opponent's Effect Monster on the field to negate (Surge Blitzclique, Swen).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_MYSTICAL_SPACE_TYPHOON, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, GRAVE),
    (CARD_MYSTICAL_SPACE_TYPHOON, GRAVE),
    (22912101, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, quickplaycon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_handler(duel).is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON))
    });
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_disablable(a.ev as u8)
        && duel.is_existing_matching_card(
            |c, duel| c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON),
            a.tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
    let rc = a.re.map(|re| re.get_handler(duel));
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, rc.map(Into::into), 1, a.tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_effect(a.ev as u8)
        && rc.is_relate_to_effect(duel, re)
        && rc.is_destructable(duel)
        && duel.is_existing_matching_card(
            |c, duel| c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON),
            tp,
            LOCATION_GRAVE,
            0,
            2,
            None,
        )
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
