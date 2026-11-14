// Prank-Kids Dodo-Doodle-Doo
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 17382973;

pub const LISTED_SERIES: &[u16] = &[SET_PRANK_KIDS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_PRANK_KIDS], Some(lc), sumtype, tp)
    }
}

// Prank-Kids Dropsies, Fansies and Lampsies to Link Summon with and to add from the Graveyard. No Prank-Kids Spell/Trap is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (55725117, MZONE),
    (81119816, MZONE),
    (18236002, MZONE),
    (55725117, MZONE),
    (55725117, GRAVE),
    (81119816, GRAVE),
    (18236002, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, thop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, thtg2);
    e2.set_operation(duel, thop2);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_link_summoned(duel)
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PRANK_KIDS]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter1, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter1, LOCATION_DECK)
}

fn thfilter2(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_set_card(duel, &[SET_PRANK_KIDS]) && !c.is_link_monster(duel) && c.is_can_be_effect_target(duel, e) && c.is_able_to_hand(duel)
}

fn thcheck(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 2, false)
}

pub fn thtg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| thfilter2(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &thcheck));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &thcheck, tp, HINTMSG_ATOHAND)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), tg.len() as u8, 0, 0);
    Ok(false)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
