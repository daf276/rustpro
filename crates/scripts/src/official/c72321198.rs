// Arcana Force XII - The Hangman
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 72321198;

pub const LISTED_SERIES: &[u16] = &[SET_ARCANA_FORCE];
pub const TOSS_COIN: bool = true;

// Copies in the hand to reveal and Normal Summon, and one on the field, with monsters on both sides for the coin to destroy; the
// opponent's Dark Hole and Sangan chain the reveal.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_COIN + CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_target(duel, cointg);
    e2a.set_operation(duel, coinop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e2c = e2b.clone_effect(duel);
    e2c.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2c);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ARCANA_FORCE])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP_DEFENSE)
}

fn cointg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    if g.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == 2
        || duel.is_player_affected_by_effect(tp, CARD_LIGHT_BARRIER).is_some()
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_EITHER, 0);
    Ok(false)
}

fn coinop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let coin = if duel.is_player_affected_by_effect(tp, CARD_LIGHT_BARRIER).is_some() {
        let b1 = duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, 0, 1, None);
        let b2 = duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None);
        let hints = [
            (b1, aux::get_coin_effect_hint_string(COIN_HEADS)),
            (b2, aux::get_coin_effect_hint_string(COIN_TAILS)),
        ];
        let Some(op) = duel.select_effect(tp, &hints)? else { return Ok(()) };
        Some(if op == 1 { COIN_HEADS } else { COIN_TAILS })
    } else {
        duel.toss_coin(tp, 1)?.first().map(|&coin| u32::from(coin))
    };
    let (player, locations) = match coin {
        Some(COIN_HEADS) => (tp, (LOCATION_MZONE, 0)),
        Some(COIN_TAILS) => (1 - tp, (0, LOCATION_MZONE)),
        _ => return Ok(()),
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, locations.0, locations.1, 1, 1, None)?;
    let Some(sc) = g.first() else { return Ok(()) };
    duel.hint_selection(sc);
    if duel.destroy(sc, REASON_EFFECT)? > 0 {
        duel.damage(player, sc.get_text_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
