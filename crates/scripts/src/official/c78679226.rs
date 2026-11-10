// Future Silence
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, SZONE};

const ID: u32 = 78679226;

pub const LISTED_NAMES: &[u32] = &[CARD_SHINING_SARCOPHAGUS];

// Shining Sarcophagus on the field and Dark Magician Girl the Magician's Apprentice (lists it) on the field and in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_SHINING_SARCOPHAGUS, SZONE),
    (2501624, MZONE),
    (2501624, DECK),
    (2501624, DECK),
    (2501624, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END | TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_SHINING_SARCOPHAGUS) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct1 = 6 - duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
    let ct2 = 6 - duel.get_field_group_count(tp, 0, LOCATION_HAND) as i32;
    let draw = duel.is_battle_phase()
        && a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_SHINING_SARCOPHAGUS)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.lists_code(duel, CARD_SHINING_SARCOPHAGUS)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        );
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
            && (!draw
                || (ct1 > 0 && duel.is_player_can_draw(tp, ct1 as usize) && ct2 > 0 && duel.is_player_can_draw(1 - tp, ct2 as usize))));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    if draw {
        a.e.set_label(duel, 1);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct1);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, ct2);
    } else {
        a.e.set_label(duel, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    if a.e.get_label(duel) == 0 {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.shuffle_deck(tp)?;
    let turn_p = duel.get_turn_player();
    let ct1 = 6 - duel.get_field_group_count(turn_p, LOCATION_HAND, 0) as i32;
    let ct2 = 6 - duel.get_field_group_count(turn_p, 0, LOCATION_HAND) as i32;
    if ct1 > 0 || ct2 > 0 {
        duel.break_effect()?;
    }
    if ct1 > 0 {
        duel.draw(turn_p, ct1 as u32, REASON_EFFECT)?;
    }
    if ct2 > 0 {
        duel.draw(1 - turn_p, ct2 as u32, REASON_EFFECT)?;
    }
    Ok(())
}
