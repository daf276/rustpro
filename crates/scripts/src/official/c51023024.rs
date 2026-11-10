// Reeshaddoll Wendi
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 51023024;

pub const LISTED_NAMES: &[u32] = &[ID];

// Face-down on the field, to be flipped by an attack and sent to the Graveyard by Dark Hole drawn later, with Shaddoll monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (30328508, DECK),
    (3717252, DECK),
    (53129443, DECK),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg::<POS_DEFENSE>);
    e1.set_operation(duel, spop::<POS_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, sptg::<POS_FACEDOWN_DEFENSE>);
    e2.set_operation(duel, spop::<POS_FACEDOWN_DEFENSE>);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, pos: u32) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, pos, tp, 0xff)
        && !c.is_code(duel, ID)
}

fn sptg<const POS: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, POS), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn spop<const POS: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, POS), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
