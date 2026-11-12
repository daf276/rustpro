// Flower Cardian Pine
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT};

const ID: u32 = 81752019;

// Three in the hand to Normal Summon, and Raigeki in the opponent's hand to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (12580477, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, nsdrtg);
    e1.set_operation(duel, nsdrop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, desdrcon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn nsdrtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn nsdrop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? > 0 {
        let Some(dc) = duel.get_operated_group().get_first() else {
            return Ok(());
        };
        duel.confirm_cards(1 - tp, dc)?;
        if !(dc.is_set_card(duel, &[SET_FLOWER_CARDIAN]) && dc.is_monster(duel)) {
            duel.send_to_grave(dc, REASON_EFFECT)?;
        }
        duel.shuffle_hand(tp)?;
    }
    Ok(())
}

fn desdrcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE)
        || (a.rp == 1 - a.tp
            && c.is_reason(duel, REASON_DESTROY)
            && c.is_reason(duel, REASON_EFFECT)
            && c.is_previous_controler(duel, a.tp))
}
