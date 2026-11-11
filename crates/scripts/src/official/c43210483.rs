// Symphonic Warrior Guitariss
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, PZONE};

const ID: u32 = 43210483;
const SYMPH_AMPLIFIRE: u32 = 75304793;

pub const LISTED_NAMES: &[u32] = &[SYMPH_AMPLIFIRE];

// Copies in the hand, Pendulum Zones and Monster Zone, with Symph Amplifire in the hand to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, PZONE),
    (ID, PZONE),
    (ID, MZONE),
    (SYMPH_AMPLIFIRE, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sthtg);
    e1.set_operation(duel, sthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_environment_full(SYMPH_AMPLIFIRE, a.tp, LOCATION_FZONE));
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn sthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SYMPHONIC_WARRIOR]) && c.is_able_to_hand(duel)
}

fn sthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    // The Lua's chkc check calls the undefined s.thfilter, which errors.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_target(sthfilter, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, sthfilter, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g.map(|g| g.union(c.into())), 2, 0, 0);
    Ok(false)
}

fn sthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, a.e)
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(CardSet::from_iter([c, tc]), PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE) && chkc.is_controler(duel, tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_able_to_hand(duel), tp, LOCATION_PZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, |c, duel| c.is_able_to_hand(duel), tp, LOCATION_PZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
