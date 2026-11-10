// Ectoplasmic Fortification
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, SZONE};

const ID: u32 = 16734927;
const CALL_OF_THE_FORGOTTEN: u32 = 80749819;
pub const LISTED_NAMES: &[u32] = &[CALL_OF_THE_FORGOTTEN, CARD_CALL_OF_THE_HAUNTED];

// Call of the Forgotten in the Deck to add with no monsters, then Zombino (a Zombie Normal Monster) revived from the Graveyard
// by Call of the Haunted for the ATK gain and draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (CALL_OF_THE_FORGOTTEN, DECK),
    (CALL_OF_THE_FORGOTTEN, DECK),
    (14575467, GRAVE),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_level(duel, &[6]) && c.is_race(duel, RACE_ZOMBIE)) || c.is_code(duel, CALL_OF_THE_FORGOTTEN)) && c.is_able_to_hand(duel)
}

fn zombie(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_ZOMBIE)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 =
        duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_matching_card(zombie, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DRAW);
        duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if op == 2 {
        let g = duel.get_matching_group(zombie, tp, LOCATION_MZONE, 0, None);
        if g.is_empty() {
            return Ok(());
        }
        let c = e.get_handler(duel);
        for tc in g {
            tc.update_attack(duel, 400, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        }
        let draw_count = duel.get_matching_group_count(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_CALL_OF_THE_HAUNTED)),
            tp,
            LOCATION_ONFIELD,
            0,
            None,
        );
        if draw_count > 0 && duel.is_player_can_draw(tp, draw_count) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            duel.break_effect()?;
            duel.draw(tp, draw_count as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
