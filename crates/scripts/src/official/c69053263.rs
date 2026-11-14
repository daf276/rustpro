// Black Rose Garden
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 69053263;

pub const LISTED_SERIES: &[u16] = &[SET_ROSE_DRAGON];
pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_ROSE_DRAGON];
// Rose Dragons (Red Rose, White Rose) in the Deck to add, Plants (Mushroom Man) in the GY for damage, the opponent's
// Mystical Space Typhoons to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26118970, DECK),
    (12213463, DECK),
    (14181608, GRAVE),
    (14181608, GRAVE),
    (5318639, SZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
        Ok(false)
    });
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CHANGE_RACE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, RACE_PLANT as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ROSE_DRAGON]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn damage(duel: &Duel, tp: u8) -> u32 {
    let count = duel.get_matching_group_count(
        |c, duel| c.is_faceup(duel) && c.is_race(duel, RACE_PLANT),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        None,
    );
    100 * count as u32
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let mut dam = damage(duel, tp);
    if a.e.get_handler(duel).is_reason(duel, REASON_EFFECT)
        && a.re.is_some_and(|re| re.get_handler(duel).is_code(duel, CARD_BLACK_ROSE_DRAGON))
    {
        a.e.set_label(duel, 1);
        if dam > 0 {
            dam += 2400;
        }
    } else {
        a.e.set_label(duel, 0);
    }
    duel.set_target_player(1 - tp);
    if dam > 0 {
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    }
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = damage(duel, a.tp);
    if dam > 0 && duel.damage(p, dam, REASON_EFFECT)? > 0 && a.e.get_label(duel) == 1 {
        duel.break_effect()?;
        duel.damage(p, 2400, REASON_EFFECT)?;
    }
    Ok(())
}
