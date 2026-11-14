// Vanquish Soul Razen
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 29302858;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// On the field and in the hand (FIRE) with Vanquish Soul Heavy Borger (DARK non-Warrior) in the Deck to search and in the hand to
// reveal, and Unicycular on the field to be destroyed in the column.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (92895501, DECK),
    (92895501, DECK),
    (92895501, HAND),
    (57308711, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, opccost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, opccost);
    e3.set_target(duel, vstg);
    e3.set_operation(duel, vsop);
    c.register_effect(duel, e3);
}

fn opccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_flag_effect(a.tp, ID) == 0);
    }
    duel.register_flag_effect(a.tp, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_monster(duel) && !c.is_race(duel, RACE_WARRIOR) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn vscostfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && !c.is_public(duel)
}

fn vsrescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = |att| sg.filter_count(duel, |c, duel| c.is_attribute(duel, att), None) > 0;
    (count(ATTRIBUTE_FIRE) && count(ATTRIBUTE_DARK), false)
}

fn column_monsters(duel: &Duel, c: CardId) -> CardSet {
    let mut colg = c.get_column_group(duel);
    colg.keep_matching(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None);
    colg
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cg1 = duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_FIRE), tp, LOCATION_HAND, 0, None);
    let b1 = !cg1.is_empty();
    let cg2 = cg1.union(duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_DARK), tp, LOCATION_HAND, 0, None));
    let colg = column_monsters(duel, e.get_handler(duel));
    let b2 = !colg.is_empty() && aux::select_unselect_group_check(duel, cg2, 1, 2, &vsrescon);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let g = cg1.select(duel, tp, 1, 1, None)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, 0);
        }
        Some(2) => {
            let g = aux::select_unselect_group_full(duel, cg2, 1, 2, &vsrescon, tp, HINTMSG_CONFIRM, Some(&vsrescon), false)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, CATEGORY_DESTROY);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(colg), colg.len() as u8, 0, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let op = a.e.get_label(duel);
    if op == 1 && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3001);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    } else if op == 2 {
        let cg = column_monsters(duel, c);
        if cg.is_empty() {
            return Ok(());
        }
        duel.destroy(cg, REASON_EFFECT)?;
    }
    Ok(())
}
