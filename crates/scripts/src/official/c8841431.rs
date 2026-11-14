// Centur-Ion Primera Primus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 8841431;

pub const LISTED_SERIES: &[u16] = &[SET_EMBLEMA, SET_CENTUR_ION];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck and on the field, Synchro Summoned from Centur-Ion Primera (Level 4 Tuner) and Centur-Ion Trudea (Level 4), with
// a copy in the Graveyard, and Emblema Oath in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (77765207, DECK),
    (77765207, DECK),
    (15005145, MZONE),
    (42493140, MZONE),
    (42493140, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indescon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn indescon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_special_summoned(duel) && !c.is_summon_location(duel, LOCATION_EXTRA)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EMBLEMA]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 2, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    let g1 = duel.get_field_group(tp, LOCATION_ONFIELD, 0);
    let g2 = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if g1.is_empty() || g2.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    let rescon = aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)));
    let sg = aux::select_unselect_group(duel, g1.union(g2), 2, 2, &rescon, tp, HINTMSG_DESTROY)?;
    if sg.len() == 2 {
        duel.hint_selection(sg);
        duel.break_effect()?;
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_set_card(duel, &[SET_CENTUR_ION])
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.contains(a.e.get_handler(duel)) && a.rp == 1 - a.tp && eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}
