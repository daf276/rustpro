// Geargiagear Gigant XG
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19891310;

pub const LISTED_SERIES: &[u16] = &[SET_GEARGIA];
// Three times in the Extra Deck, Xyz Summoned with Level 3 Machine Normal Monsters, with Geargiarsenals in the Graveyard, and the
// opponent's Elfnote Lucinas to battle and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (10859908, MZONE),
    (38916461, MZONE),
    (47372349, MZONE),
    (92421852, MZONE),
    (45945685, MZONE),
    (73428497, GRAVE),
    (73428497, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_ATTACK | TIMING_BATTLE_PHASE, TIMING_ATTACK | TIMING_BATTLE_PHASE);
    e1.set_condition(duel, discon);
    // The Lua passes Cost.SoftOncePerBattle uncalled, so it is a truthy function that limits nothing.
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_position(duel, POS_FACEUP));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (bc, _) = duel.get_battle_monster(a.tp);
    bc.is_some_and(|bc| bc.is_race(duel, RACE_MACHINE) && bc.is_faceup(duel))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(|c, duel| c.is_negatable(duel), tp, 0, LOCATION_ONFIELD, None);
    for nc in g.iter() {
        nc.negate_effects(duel, c, RESET_PHASE | PHASE_DAMAGE, true);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEARGIA]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && chkc != c && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
