// Fish Sonar
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 22819092;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI];

// Umi in the hand to activate, and Island Turtle, a Level 4 WATER Normal Monster, in the Deck to add and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_UMI, HAND), (4042268, DECK), (4042268, DECK), (4042268, DECK)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(4042268)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 7)
        && c.is_able_to_hand(duel)
        && ((c.is_type(duel, TYPE_NORMAL) && c.is_attribute(duel, ATTRIBUTE_WATER)) || c.lists_code(duel, CARD_UMI))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? < 1 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let umi = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI));
    if (duel.is_existing_matching_card(umi, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None) || duel.is_environment(CARD_UMI))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if sg.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
