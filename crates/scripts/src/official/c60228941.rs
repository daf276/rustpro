// Snoww, Unlight of Dark World
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT};

const ID: u32 = 60228941;

// In the hand, to be discarded by Dark World Dealings from either side, with Renge and Zure in the Deck to search and a
// monster in the opponent's Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (74117290, HAND),
    (74117290, HAND | OPPONENT),
    (60606759, DECK),
    (7459013, DECK),
    (6297941, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition_mut(duel, super::c34230233::descon);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_WORLD]) && c.is_able_to_hand(duel)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter2(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    if tp != a.rp
        && i64::from(tp) == e.get_label(duel)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(|c, duel| filter2(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(tp, |c, duel| filter2(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    } else {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter1, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.break_effect()?;
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}
