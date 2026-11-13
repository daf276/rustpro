// Tindangle Protector
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 67744384;

// Face-down to be flipped by Elfnote Lucina, with Tindangle Angel in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (68860936, DECK),
    (68860936, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_facedown, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, None, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TINDANGLE]) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_matching_card(tp, CardId::is_facedown, tp, LOCATION_MZONE, 0, 1, 3, None)?;
    if !g.is_empty() {
        let ct = duel.change_position(g, POS_FACEUP_DEFENSE)? as usize;
        if ct == g.len() && g.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_TINDANGLE]), ct, None) {
            let sg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
            if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
                duel.break_effect()?;
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
                let tg = sg.select(duel, tp, 1, ct as u8, None)?;
                duel.send_to_hand(tg, tp, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, tg)?;
            }
        }
    }
    Ok(())
}
