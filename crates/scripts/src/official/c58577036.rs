// Reasoning
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 58577036;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_any(tp)
            && duel.is_player_can_discard_deck(tp, 1)
            && duel.is_existing_matching_card(CardId::is_summonable_card, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_player_affected_by_effect(tp, CARD_EHERO_BLAZEMAN).is_none());
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_player_can_special_summon_any(tp) || !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_LVRANK);
    let lv = duel.announce_level(1 - tp)?;
    let g = duel.get_matching_group(CardId::is_summonable_card, tp, LOCATION_DECK, 0, None);
    let dcount = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    let Some((seq, spcard)) = g.iter().map(|tc| (tc.get_sequence(duel) as usize, tc)).max_by_key(|&(seq, _)| seq) else {
        duel.confirm_decktop(tp, dcount)?;
        duel.shuffle_deck(tp)?;
        return Ok(());
    };
    duel.confirm_decktop(tp, dcount - seq)?;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && lv != u64::from(spcard.get_level(duel))
        && spcard.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    {
        duel.disable_shuffle_check(true);
        if dcount - seq == 1 {
            duel.special_summon(spcard, 0, tp, tp, false, false, POS_FACEUP)?;
        } else {
            duel.special_summon_step(spcard, 0, tp, tp, false, false, POS_FACEUP)?;
            duel.discard_deck(tp, (dcount - seq - 1) as u16, REASON_EFFECT)?;
            duel.special_summon_complete()?;
        }
    } else {
        duel.discard_deck(tp, (dcount - seq) as u16, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}
