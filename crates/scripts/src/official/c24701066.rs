// Infinitrack River Stormer
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 24701066;

// Xyz Summoned from Steel Ogre Grotto #1 (Level 5 EARTH Machine, also the one to search), with an opponent's monster to
// battle, a copy in the Graveyard, and Clockwork Knight (Machine Link Monster) to release.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (29172562, MZONE),
    (29172562, MZONE),
    (29172562, MZONE),
    (29172562, MZONE),
    (29172562, DECK),
    (29172562, DECK),
    (41739381, MZONE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, aux::bdocon);
    e1.set_target(duel, atchtg);
    e1.set_operation(duel, atchop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtgtg);
    e2.set_operation(duel, thtgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn atchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ)
            && bc.is_monster(duel)
            && bc.is_faceup(duel)
            && bc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT));
    }
    duel.set_target_card(bc);
    if bc.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(bc.into()), 1, tp, 0);
    }
    Ok(false)
}

fn atchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, a.e)
        && c.is_type(duel, TYPE_XYZ)
        && bc.is_relate_to_effect(duel, a.e)
        && bc.is_monster(duel)
        && bc.is_faceup(duel)
    {
        duel.overlay_full(c, bc, true)?;
    }
    Ok(())
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_MACHINE) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    if let Some(sc) = duel
        .select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    {
        let send = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(drop);
        aux::to_hand_or_else(duel, sc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)?;
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_link_monster(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| spcostfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
