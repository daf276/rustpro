// Tri-Brigade Shuraig the Ominous Omen
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 99726621;

pub const LISTED_SERIES: &[u16] = &[SET_TRI_BRIGADE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACES_BEAST_BWARRIOR_WINGB, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck and on the field with Beast, Beast-Warrior and Winged Beast monsters to Link Summon it, Tri-Brigade
// Fraktall and Springans Kitt to Special Summon more, banished ones for the search, and the opponent's Dark Hole and
// Raigeki to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (6297941, MZONE),
    (6297941, MZONE),
    (7805359, MZONE),
    (87209160, MZONE),
    (87209160, MZONE),
    (19304410, HAND),
    (19304410, HAND),
    (6297941, GRAVE),
    (6297941, GRAVE),
    (7805359, GRAVE),
    (6297941, REMOVED),
    (7805359, REMOVED),
    (6297941, REMOVED),
    (6297941, DECK),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, rmvcon);
    e2.set_target(duel, rmvtg);
    e2.set_operation(duel, rmvop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_controler(duel, tp)
}

fn rmvcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, Some(a.e.get_handler(duel).into())))
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 0, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, lv: usize) -> bool {
    c.is_level_below(duel, lv as u32) && c.is_able_to_hand(duel) && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_monster(duel)
}

fn banished_count(duel: &Duel, tp: u8) -> usize {
    let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB));
    duel.get_matching_group_count(filter, tp, LOCATION_REMOVED, 0, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rgc = banished_count(duel, tp);
    if chk == 0 {
        return Ok(rgc > 0 && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, rgc), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rgc = banished_count(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, rgc), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
