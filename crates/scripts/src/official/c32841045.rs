// Magical Musketeer Caspar
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 32841045;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];

const DESPERADO: u32 = 20745268;

// On the field in several columns, with Magical Musket - Desperado in the hand to activate in one of them, and Magical Musketeer
// Kidbrave in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (5230799, DECK),
    (5230799, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, LOCATION_DECK>(duel, c);
}

// The effects of Caspar and Magical Musketeer Doc, which adds from LOCATION instead.
pub fn register<const CODE: u32, const LOCATION: u32>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(CODE, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_operation(duel, regop::<CODE>);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_condition(duel, regcon);
    e4.set_operation(duel, regop2);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(CODE, 0));
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_code(duel, EVENT_CUSTOM + CODE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, CODE);
    e5.set_label_object_group(duel, Some(CardSet::default()));
    e5.set_target(duel, thtg::<CODE, LOCATION>);
    e5.set_operation(duel, thop::<LOCATION>);
    c.register_effect(duel, e5);
    // The Lua's group, shared by e3 and e5, is kept on e5.
    e3.set_label_object(duel, Some(e5));
}

fn regop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let flageff = c.get_flag_effect_labels(duel, 1);
    let Some(e5) = e.get_label_object(duel) else { return Ok(()) };
    if !flageff.contains(&i64::from(a.ev)) {
        return Ok(());
    }
    let mut g = e5.get_label_object_group(duel).unwrap_or_default();
    if c.get_flag_effect(duel, 2) == 0 {
        g.clear();
        c.register_flag_effect(duel, 2, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_CHAIN, 0, 1);
        duel.raise_single_event(c, EVENT_CUSTOM + CODE, Some(e), 0, 0, 0, 0)?;
    }
    if let Some(re) = a.re {
        g.insert(re.get_handler(duel));
    }
    e5.set_label_object_group(duel, Some(g));
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let p = duel.get_chain_info::<u8>(a.ev as u8, CHAININFO_TRIGGERING_CONTROLER);
    let seq = duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_SEQUENCE);
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && a.e.get_handler(duel).is_column(duel, seq, p, LOCATION_SZONE)
}

fn regop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let reset = RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_CHAIN;
    a.e.get_handler(duel)
        .register_flag_effect_full(duel, 1, reset, 0, 1, i64::from(a.ev), 0);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_set_card(duel, &[SET_MAGICAL_MUSKET]) && !c.is_code(duel, code) && c.is_able_to_hand(duel)
}

fn chkfilter<const LOCATION: u32>(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, code), tp, LOCATION, 0, 1, Some(c.into()))
}

fn thtg<const CODE: u32, const LOCATION: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e.get_label_object_group(duel).unwrap_or_default();
    if chk == 0 {
        return Ok(g.is_exists(duel, |c, duel| chkfilter::<LOCATION>(c, duel, tp), 1, None));
    }
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(CODE, 1));
        g.select(duel, tp, 1, 1, None)?.get_first()
    } else {
        g.get_first()
    };
    let code = tc.map_or(0, |tc| tc.get_code(duel));
    e.set_label(duel, i64::from(code));
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION as i32);
    Ok(false)
}

fn thop<const LOCATION: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let code = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let except = e.get_label_object_group(duel);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, code), tp, LOCATION, 0, 1, 1, except)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
