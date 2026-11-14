// Floowandereeze & Robina
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, REMOVED};

const ID: u32 = 18940725;

// Copies in hand to Normal Summon and banished to return, Winged Beasts in the Deck to add and in hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, REMOVED),
    (76812113, DECK),
    (3170832, DECK),
    (30532390, DECK),
    (76812113, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost::<ID>);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    register_banish_effects::<ID>(duel, c);
}

// The banish-on-leaving and return-from-banishment effects all Floowandereeze monsters register after their first.
pub fn register_banish_effects<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_faceup(duel));
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_cost(duel, cost::<ID>);
    e3.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| thfilter(c, duel, a.tp), 1, None))
    });
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

pub fn cost<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_SPSUMMON) == 0);
    }
    let handler = a.e.get_handler(duel);
    let e1 = duel.create_effect(handler);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, handler, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_able_to_hand(duel)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WINGEDBEAST) && c.is_summonable(duel, true, None)
}

// The optional Normal Summon of a Winged Beast that ends every Floowandereeze monster's first effect.
pub fn summon_winged_beast<const ID: u32>(duel: &mut Duel, tp: u8) -> Script<()> {
    let sg1 = duel.get_matching_group(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    if !sg1.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.break_effect()?;
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
        if let Some(sg2) = sg1.select(duel, tp, 1, 1, None)?.get_first() {
            duel.summon(tp, sg2, true, None)?;
        }
    }
    Ok(())
}

pub fn search_and_summon<const ID: u32>(duel: &mut Duel, tp: u8, filter: fn(CardId, &Duel) -> bool) -> Script<()> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
        if !tc.is_location(duel, LOCATION_HAND) {
            return Ok(());
        }
        summon_winged_beast::<ID>(duel, tp)?;
    }
    Ok(())
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    search_and_summon::<ID>(duel, a.tp, filter)
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_race(duel, RACE_WINGEDBEAST)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}
