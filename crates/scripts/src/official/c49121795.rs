// Heavy Armored Train Ironwolf
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 49121795;

// In the Extra Deck, Xyz Summoned with Level 4 Machines (Overdrive, Mechanicalchaser, Golgoil), with more in the Deck to search
// and Raigeki and Dark Hole on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2311603, MZONE),
    (7359741, MZONE),
    (7526150, MZONE),
    (7526150, MZONE),
    (2311603, DECK),
    (7359741, DECK),
    (12580477, DECK | OPPONENT),
    (53129443, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_MACHINE, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, datg);
    e1.set_operation(duel, daop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn dafilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_none()
}

fn datg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && dafilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(dafilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, dafilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn daop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, tc.get_field_id(duel).into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if tc.is_relate_to_effect(duel, e) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_DIRECT_ATTACK);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_controler(duel, a.tp) && a.rp == 1 - a.tp && c.is_reason(duel, REASON_DESTROY)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.get_level(duel) == 4 && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
