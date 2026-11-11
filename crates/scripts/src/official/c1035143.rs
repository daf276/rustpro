// Darkuriboh
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 1035143;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// Copies in hand and a Fiend (Zoa) on the field for the opponent's Mage Power to target, Polymerization in the Deck to add, and
// the opponent's Hecahands in the Graveyard to make the player draw and discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (24311372, MZONE),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, DECK),
    (49161188, HAND | OPPONENT),
    (20415050, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_BECOME_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, effcon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn effconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_race(duel, RACE_FIEND) || c.is_type(duel, TYPE_FUSION))
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| effconfilter(c, duel, tp), 1, None))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(
        duel,
        |c, duel| effconfilter(c, duel, tp) && c.is_can_be_effect_target(duel, e),
        None,
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tg = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        g.select(duel, tp, 1, 1, None)?
    } else {
        g
    };
    duel.set_target_card(tg);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_monster(duel)) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    tc.register_flag_effect(
        duel,
        ID,
        RESET_EVENT | RESET_TURN_SET | RESET_TOFIELD | RESET_PHASE | PHASE_END,
        0,
        1,
    );
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    tc.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_DISEFFECT);
    e3.set_target_range(duel, 1, 1);
    e3.set_value(duel, Value::Chain(effectfilter));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn effectfilter(duel: &Duel, _: EffectId, ct: u8) -> bool {
    let Some(eff) = duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let loc: u32 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_LOCATION);
    eff.get_handler(duel).has_flag_effect(duel, ID) && loc & LOCATION_ONFIELD > 0
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_HAND) && c.is_reason(duel, REASON_EFFECT)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK | LOCATION_GRAVE)
}
