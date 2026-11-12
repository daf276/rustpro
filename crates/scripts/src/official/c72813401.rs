// Hi-Speedroid Rubber Band Shooter
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 72813401;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WIND, Some(lc), sumtype, tp)
    }
}

// Link Summoned from WIND Speedroids, with Level 1 and 3 Speedroids in the Deck to search and WIND Synchro Monsters (Puzzle, Level 4)
// in the Extra Deck to banish, WIND monsters in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (70939418, MZONE),
    (70939418, MZONE),
    (62899696, MZONE),
    (62899696, MZONE),
    (86943389, EXTRA),
    (86943389, EXTRA),
    (62899696, DECK),
    (70939418, DECK),
    (81275020, DECK),
    (96945958, DECK),
    (81275020, HAND),
    (81275020, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOGRAVE + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(nsfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_SYNCHRO) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, e.get_handler(duel), tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPEEDROID]) && c.is_monster(duel) && c.is_level_above(duel, 1) && c.is_able_to_hand(duel)
}

fn rescon(lv: i64) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let sum = sg.get_sum(duel, |c, duel| i64::from(c.get_level(duel)));
        (
            sum == lv && sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 2,
            false,
        )
    }
}

fn cfilter(c: CardId, duel: &Duel, g: CardSet) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_able_to_remove_as_cost(duel)
        && aux::select_unselect_group_check(duel, g, 2, 2, &rescon(i64::from(c.get_level(duel))))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, g), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let sc = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, g), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = sc.get_first() {
        duel.remove(sc, POS_FACEUP, REASON_COST)?;
        let level = i64::from(sc.get_level(duel));
        e.set_label(duel, level);
    }
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    let lv = e.get_label(duel);
    let mut sg = aux::select_unselect_group(duel, g, 2, 2, &rescon(lv), tp, HINTMSG_SELECT)?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_deck(tp)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
    let cg = sg.select(duel, 1 - tp, 1, 1, None)?;
    if let Some(tc) = cg.get_first() {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        sg.remove(tc);
    }
    duel.send_to_grave(sg, REASON_EFFECT)?;
    Ok(())
}
