// Volcanic Trooper
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 22411609;

pub const LISTED_SERIES: &[u16] = &[SET_VOLCANIC];
pub const LISTED_NAMES: &[u32] = &[ID, TOKEN_BOMB];

// Copies in hand to Normal Summon, with Volcanic Shell in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (33365932, DECK), (33365932, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_BOMB),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, tkcost);
    e3.set_target(duel, tktg);
    e3.set_operation(duel, tkop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VOLCANIC]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 1000, 1000, 1, RACE_PYRO, ATTRIBUTE_FIRE);
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_player_can_special_summon_monster_full(tp, TOKEN_BOMB, stats, POS_FACEUP, 1 - tp, 0)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(can_summon_token(duel, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !can_summon_token(duel, tp) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, TOKEN_BOMB) else {
        return Ok(());
    };
    if duel.special_summon_step(token, 0, tp, 1 - tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_LEAVE_FIELD);
        e1.set_operation(duel, damop);
        token.register_effect_full(duel, e1, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_DESTROY) {
        let p = c.get_previous_controler(duel);
        duel.damage(p, 500, REASON_EFFECT)?;
    }
    a.e.reset(duel);
    Ok(())
}
