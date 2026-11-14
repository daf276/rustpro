// Witchcrafter Seed
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 59106048;

pub const LISTED_SERIES: &[u16] = &[SET_WITCHCRAFTER];
pub const LISTED_NAMES: &[u32] = &[ID, CARD_REGULUS_THE_PRINCE_OF_ENDYMION];

// In the hand and the Graveyard, with Witchcrafter monsters on the field (some are placed face-down) and Pot of Greed in the hand to
// shuffle, and Sangan of the opponent to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (64756282, MZONE),
    (64756282, MZONE),
    (21744288, MZONE),
    (55144522, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, rthtg);
    e1a.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drawtg);
    e2.set_operation(duel, drawop);
    c.register_effect(duel, e2);
}

fn rthconfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_monster(duel) && !c.is_code(duel, ID))
        || c.is_code(duel, CARD_REGULUS_THE_PRINCE_OF_ENDYMION))
        && c.is_faceup(duel)
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && rthfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rthconfilter, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(rthfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, rthfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn handfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck(duel) && !c.is_public(duel)
}

fn drawtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 0)
            && duel.is_existing_matching_card(|c, duel| c.is_spell(duel) && handfilter(c, duel), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.is_exists(duel, CardId::is_spell, 1, None), false)
}

fn drawop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(handfilter, tp, LOCATION_HAND, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group_full(duel, g, 1, g.len(), &rescon, tp, HINTMSG_TODECK, Some(&rescon), false)?;
    if !sg.is_empty() {
        duel.confirm_cards(1 - tp, sg)?;
        let returned_count = duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        if returned_count > 0 {
            duel.shuffle_deck(tp)?;
            if duel.is_player_can_draw(tp, 0) {
                duel.break_effect()?;
                duel.draw(tp, returned_count as u32, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
