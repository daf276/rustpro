// Snake-Eyes Poplar
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 90241276;
const SNAKE_EYE_ASH: u32 = 9674034;
const DRAMATIC_SNAKE_EYE_CHASE: u32 = 26700718;
const KITCHEN_DRAGONMAID: u32 = 16960120;

// In the hand and on the field, with Snake-Eye Ash to add it, Dramatic Snake-Eye Chase in the Deck to search, and a FIRE
// monster (Kitchen Dragonmaid) in the Graveyard to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (SNAKE_EYE_ASH, MZONE),
    (SNAKE_EYE_ASH, DECK),
    (DRAMATIC_SNAKE_EYE_CHASE, DECK),
    (DRAMATIC_SNAKE_EYE_CHASE, DECK),
    (KITCHEN_DRAGONMAID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, pltg);
    e4.set_operation(duel, plop);
    c.register_effect(duel, e4);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SNAKE_EYE]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    let p = c.get_owner(duel);
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && duel.get_location_count(p, LOCATION_SZONE) > 0
        && c.check_unique_on_field(duel, p)
        && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && plfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(plfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, plfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
        && duel.move_to_field(tc, tp, tc.get_owner(duel), LOCATION_SZONE, POS_FACEUP, true)?
    {
        treat_as_continuous_spell(duel, e.get_handler(duel), tc);
    }
    Ok(())
}

pub fn treat_as_continuous_spell(duel: &mut Duel, owner: CardId, tc: CardId) {
    let e1 = duel.create_effect(owner);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
    tc.register_effect(duel, e1);
}
