// Fairy Tail - Rochka
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 65953423;

// A copy in the hand to Normal Summon and on the field to battle an opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, exctg);
    e1.set_operation(duel, excop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_CONFIRM);
    e2.set_condition(duel, sgycon);
    e2.set_target(duel, sgytg);
    e2.set_operation(duel, sgyop);
    c.register_effect(duel, e2);
}

fn exctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_decktop_group(tp, 3);
        return Ok(g.len() >= 3 && g.filter_count(duel, CardId::is_able_to_hand, None) > 0);
    }
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, 500);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn excop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.recover(1 - tp, 500, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    duel.confirm_decktop(tp, 3)?;
    let g = duel.get_decktop_group(tp, 3);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
        let sc = g.select(duel, 1 - tp, 1, 1, None)?.get_first();
        // The script indexes a nil card, a Lua error.
        unported!(duel, sc.is_none());
        let Some(sc) = sc else { return Ok(()) };
        if sc.is_able_to_hand(duel) {
            duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sc)?;
            duel.shuffle_hand(tp)?;
        } else {
            duel.send_to_grave(sc, REASON_RULE)?;
        }
        duel.shuffle_deck(tp)?;
    }
    Ok(())
}

fn sgycon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel)
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_faceup(duel) && bc.is_relate_to_battle(duel))
}

fn sgytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn sgyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_grave(c, REASON_EFFECT)?;
    }
    Ok(())
}
