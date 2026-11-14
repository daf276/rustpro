// Avendread Savior
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT};

const ID: u32 = 91420202;

pub const LISTED_SERIES: &[u16] = &[SET_VENDREAD];

const REVENDREAD_SLAYER: u32 = 4388680;

pub const LISTED_NAMES: &[u32] = &[REVENDREAD_SLAYER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_ZOMBIE, Some(lc), sumtype, tp)
    }
}

// Copies on the field to attack into the opponent's monsters, with Revendread Slayer in the Graveyard to add and Zombies in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (REVENDREAD_SLAYER, GRAVE),
    (20277860, DECK),
    (20277860, DECK),
    (17733394, DECK),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, REVENDREAD_SLAYER as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, atkcon);
    e3.set_cost(duel, atkcost);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VENDREAD]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, sg, sg.map_or(0, |sg| sg.len() as u8), 0, 0);
    Ok(false)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel)
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_faceup(duel) && bc.is_relate_to_battle(duel))
}

fn atkcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_able_to_grave_as_cost(duel)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atkcfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, atkcfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.send_to_grave(tc, REASON_COST)?;
        let level = i64::from(tc.get_level(duel));
        e.set_label(duel, level);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let ct = e.get_label(duel) * 200;
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if c.is_faceup(duel) && c.is_relate_to_battle(duel) && bc.is_faceup(duel) && bc.is_relate_to_battle(duel) && ct > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -ct as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        bc.register_effect(duel, e1);
    }
    Ok(())
}
