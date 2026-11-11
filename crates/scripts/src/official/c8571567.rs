// Blackwing - Shamal the Sandstorm
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 8571567;

pub const LISTED_NAMES: &[u32] = &[7602800, CARD_BLACK_WINGED_DRAGON];

// Black Feather Whirlwind in the Deck, and in the GY with Blackwing - Bora the Spear to add and a Synchro Summon of Blackwing - Boreastorm the Wicked Wind
// (Blackwing - Blizzard the Far North and Blackwing - Zephyros the Elite on the field).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (7602800, DECK),
    (7602800, DECK),
    (49003716, GRAVE),
    (10602628, EXTRA),
    (10602628, EXTRA),
    (22835145, MZONE),
    (22835145, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, tftg);
    e1.set_operation(duel, tfop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tffilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, 7602800) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn tftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0
        || duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| tffilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn tfop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let tc = duel
        .select_matching_card(tp, |c, duel| tffilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    ((c.is_set_card(duel, &[SET_BLACKWING]) && c.is_type(duel, TYPE_SYNCHRO)) || c.is_code(duel, CARD_BLACK_WINGED_DRAGON))
        && c.is_controler(duel, tp)
        && c.is_faceup(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

// The Lua's target function takes no chkc, so there is no early return for one.
fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 700);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? == 1
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, tc)?;
        duel.break_effect()?;
        duel.damage(tp, 700, REASON_EFFECT)?;
    }
    Ok(())
}
