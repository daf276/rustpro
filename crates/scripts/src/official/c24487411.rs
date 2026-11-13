// Detonate Deleter
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 24487411;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Protrons (Cyberse Normal Monsters), with monsters of the opponent to battle and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, dtarget);
    e1.set_operation(duel, doperation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, tdcost);
    e2.set_target(duel, tdtarget);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn dtarget(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_faceup(duel) && !(tc.is_link_monster(duel) && tc.is_link_above(duel, 3))));
    }
    let Some(tc) = tc else { return Ok(false) };
    duel.set_operation_info(
        0,
        CATEGORY_DESTROY,
        Some(tc.into()),
        1,
        tc.get_controler(duel),
        LOCATION_MZONE as i32,
    );
    Ok(false)
}

fn doperation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, cg: CardSet) -> bool {
    cg.is_contains(c) && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, Some(c.into()))
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let cg = a.e.get_handler(duel).get_linked_group(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp, cg), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp, cg), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn tdtarget(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tg = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}
