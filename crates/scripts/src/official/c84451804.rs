// Des Frog
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 84451804;
const TADPOLE: u32 = 10456559;

pub const LISTED_NAMES: &[u32] = &[TADPOLE];

// In the hand and Deck, Tribute Summoned over a Normal Monster, with T.A.D.P.O.L.E. in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (TADPOLE, GRAVE),
    (TADPOLE, GRAVE),
    (89631139, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| c.is_code(duel, TADPOLE), tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let ct = duel
        .get_matching_group_count(|c, duel| c.is_code(duel, TADPOLE), tp, LOCATION_GRAVE, 0, None)
        .min(ft.max(0) as usize);
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        ct as u8,
        None,
    )?;
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
