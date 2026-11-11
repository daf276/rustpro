// Plunder Patrollship Brann
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 94253655;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned with Goldenhair (Level 4 Tuner) and Zombino (Level 4), with Whitebeard in the hand to discard and a Set Trap on
// the opponent's side to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (81344070, MZONE),
    (81344070, MZONE),
    (14575467, MZONE),
    (14575467, MZONE),
    (31374201, HAND),
    (31374201, HAND),
    (14315573, SZONE | OPPONENT),
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
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| c.is_race(duel, RACE_FIEND) && c != e.get_handler(duel));
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| !equipped_plunder_patroll(duel, a));
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMING_SSET | TIMING_EQUIP | TIMING_END_PHASE);
    e3.set_condition(duel, equipped_plunder_patroll);
    c.register_effect(duel, e3);
}

fn equipped_plunder_patroll(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_group(duel)
        .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_PLUNDER_PATROLL]), 1, None)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL]) && c.is_discardable(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, tgfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let location = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_location(duel));
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, location as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
    {
        let sg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = sg.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
