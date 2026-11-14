// A.I. Connect
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 79015062;

pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

// In the hand with Wizard @Ignister (DARK) and Bururu @Ignister (WIND) in the hand to reveal and in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (3723262, HAND),
    (42429678, HAND),
    (3723262, DECK),
    (42429678, DECK),
    (42429678, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(3723262)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn revfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !c.is_race(duel, RACE_CYBERSE) || c.is_public(duel) {
        return false;
    }
    if c.is_attribute(duel, ATTRIBUTE_DARK) {
        mmz_chk
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(thfilter1, tp, LOCATION_DECK, 0, 1, None)
    } else {
        c.is_able_to_deck(duel) && {
            let attr = c.get_attribute(duel);
            duel.is_existing_matching_card(|c, duel| thfilter2(c, duel, attr), tp, LOCATION_DECK, 0, 1, None)
        }
    }
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_attribute_except(duel, ATTRIBUTE_DARK) && c.is_level_below(duel, 4) && c.is_race(duel, RACE_CYBERSE) && c.is_able_to_hand(duel)
}

fn thfilter2(c: CardId, duel: &Duel, attr: u32) -> bool {
    c.is_set_card(duel, &[SET_IGNISTER]) && c.is_monster(duel) && c.is_attribute_except(duel, attr) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let filter = |c, duel: &Duel| revfilter(c, duel, e, tp, mmz_chk);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rc = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, rc)?;
    duel.shuffle_hand(tp)?;
    e.set_label_object_group(duel, Some(rc));
    duel.set_target_card(rc);
    let category = match rc.get_first().is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_DARK)) {
        true => CATEGORY_SPECIAL_SUMMON,
        false => CATEGORY_TODECK,
    };
    duel.set_operation_info(0, category, Some(rc), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(rc) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && rc.is_relate_to_effect(duel, e)
    {
        let dark = rc.is_attribute(duel, ATTRIBUTE_DARK);
        let success = if dark {
            duel.special_summon(rc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        } else {
            duel.confirm_cards(1 - tp, rc)?;
            duel.send_to_deck(rc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        };
        if success {
            let attr = rc.get_attribute(duel);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = match dark {
                true => duel.select_matching_card(tp, thfilter1, tp, LOCATION_DECK, 0, 1, 1, None)?,
                false => duel.select_matching_card(tp, |c, duel| thfilter2(c, duel, attr), tp, LOCATION_DECK, 0, 1, 1, None)?,
            };
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_CYBERSE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
