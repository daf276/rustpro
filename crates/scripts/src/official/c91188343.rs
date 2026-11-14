// The Agent of Mystery - Earth
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 91188343;

pub const LISTED_SERIES: &[u16] = &[SET_THE_AGENT];

const MASTER_HYPERION: u32 = 55794644;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_SANCTUARY_SKY, MASTER_HYPERION];
// Normal Summoned from the hand, with The Agent of Creation - Venus and Master Hyperion in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (64734921, DECK),
    (64734921, DECK),
    (MASTER_HYPERION, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel, sanctuary: bool) -> bool {
    ((c.is_set_card(duel, &[SET_THE_AGENT]) && c.is_monster(duel) && !c.is_code(duel, ID))
        || (sanctuary && c.is_code(duel, MASTER_HYPERION)))
        && c.is_able_to_hand(duel)
}

fn sanctuary_check(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_SANCTUARY_SKY)),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        None,
    ) || duel.is_environment(CARD_SANCTUARY_SKY)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let sanctuary = sanctuary_check(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, sanctuary), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let sanctuary = sanctuary_check(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, sanctuary), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
