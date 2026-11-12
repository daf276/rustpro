// Judgment, the Dragon of Heaven
use crate::prelude::*;
use crate::proc_synchro::Requirement;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 41659072;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;
    const REQM: Option<Requirement> = Some(syncheck);

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Majestic Dragon (Level 7 LIGHT Tuner) and Key Mace (Level 1 LIGHT), with four different
// Tuners (Water Spirit, Flamvell Guard, Lunar Rabbit Omajinai, Majestic Dragon) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (21159309, MZONE),
    (21159309, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (487395, GRAVE),
    (21615956, GRAVE),
    (47643326, GRAVE),
    (21159309, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e3.set_target(duel, target2);
    e3.set_operation(duel, operation2);
    c.register_effect(duel, e3);
}

fn syncheck(duel: &Duel, g: CardSet, sc: CardId, tp: u8) -> bool {
    aux::check_same_property(duel, g, &|c, duel| {
        u64::from(c.get_attribute_full(duel, Some(sc), SUMMON_TYPE_SYNCHRO.into(), tp))
    })
    .0
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(|c, duel| c.is_type(duel, TYPE_TUNER), a.tp, LOCATION_GRAVE, 0, None);
    a.e.get_handler(duel).is_synchro_summoned(duel) && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) > 3
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.pay_lp_cost(tp, duel.get_lp(tp) as u32 / 2)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.destroy(sg, REASON_EFFECT)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_DRAGON));
    duel.register_effect(e1, tp);
    Ok(())
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let rg = duel.get_decktop_group(tp, 4);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), 4, 0, 0);
    Ok(false)
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    let rg = duel.get_decktop_group(a.tp, 4);
    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
