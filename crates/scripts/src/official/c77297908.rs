// Abyss Playhouse - Fantastic Theater
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 77297908;

// Activated from the hand. The search needs an Abyss Actor Pendulum Monster and two Abyss Script Spells, and the replacement an Abyss
// Actor Pendulum Summoned, none of which is ported, so both are only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, chcon1);
    e3.set_operation(duel, chop1);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ABYSS_ACTOR]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_public(duel)
}

fn cfilter3(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_set_card(duel, &[SET_ABYSS_SCRIPT]) && c.is_spell(duel) && !c.is_code(duel, code) && c.is_able_to_hand(duel)
}

fn cfilter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_set_card(duel, &[SET_ABYSS_SCRIPT])
        && c.is_spell(duel)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|sc, duel| cfilter3(sc, duel, code), tp, LOCATION_DECK, 0, 1, None)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| cfilter2(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let mut g1 = duel.select_matching_card(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    let g2 = duel.select_matching_card(tp, |c, duel| cfilter2(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(sc) = g2.get_first() {
        e.set_label(duel, i64::from(sc.get_code(duel)));
    }
    g1.merge(g2);
    duel.confirm_cards(1 - tp, g1)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let code = e.get_label(duel) as u32;
    let g = duel.select_matching_card(tp, |c, duel| cfilter3(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn confilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ABYSS_ACTOR]) && c.is_pendulum_summoned(duel)
}

fn chcon1(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_existing_matching_card(confilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn chop1(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_spell_trap(duel)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, ID.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
