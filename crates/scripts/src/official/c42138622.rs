// Vortex of Time
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 42138622;

pub const LISTED_NAMES: &[u32] = &[CARD_CALL_OF_THE_HAUNTED];

// Set, with a Zombie on the field and Call of the Haunted in the hand to bring one back, against the opponent's Spells.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (32274490, MZONE),
    (32274490, GRAVE),
    (CARD_CALL_OF_THE_HAUNTED, HAND),
    (CARD_CALL_OF_THE_HAUNTED, HAND),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COIN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, changecon);
    e1.set_target(duel, changetg);
    e1.set_operation(duel, changeop);
    c.register_effect(duel, e1);
}

fn changecon(duel: &Duel, a: &Args) -> bool {
    let (tp, Some(re)) = (a.tp, a.re) else { return false };
    let rc = re.get_handler(duel);
    let zombie = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_ZOMBIE));
    let call = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_CALL_OF_THE_HAUNTED));
    a.rp == 1 - tp
        && (re.is_monster_effect(duel) || (re.is_spell_effect(duel) && (rc.is_normal_spell(duel) || rc.is_quick_play_spell(duel))))
        && duel.is_existing_matching_card(zombie, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(call, tp, LOCATION_ONFIELD, 0, 1, None)
}

fn changetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, 1 - tp, 1);
    Ok(false)
}

fn changeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    duel.change_target_card(ev, CardSet::default());
    let coin = duel.toss_coin(1 - a.tp, 1)?.first().map(|&coin| u32::from(coin));
    if coin == Some(COIN_HEADS) {
        duel.change_chain_operation(ev, repop1);
    } else if coin == Some(COIN_TAILS) {
        duel.change_chain_operation(ev, repop2);
    }
    Ok(())
}

fn repop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let opp = 1 - a.tp;
    duel.hint(HINT_SELECTMSG, opp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(opp, CardId::is_able_to_remove, opp, LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.remove_full(g, POS_FACEUP, REASON_RULE, PLAYER_NONE, Some(opp))?;
    }
    Ok(())
}

fn repop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_RULE)?;
    }
    Ok(())
}
