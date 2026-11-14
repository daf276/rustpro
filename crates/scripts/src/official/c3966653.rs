// Flower Cardian Boardefly
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 3966653;

pub const LISTED_SERIES: &[u16] = &[SET_FLOWER_CARDIAN];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 2;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Melffy Pinny (a Level 2 Tuner) and two Melffy Rabby (Level 2), with Flower Cardian Pine to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (34800281, MZONE),
    (34800281, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (20129614, MZONE),
    (81752019, GRAVE),
    (81752019, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_PIERCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_FLOWER_CARDIAN]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, cost);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOWER_CARDIAN])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_target_range(duel, 0, 1);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_location(duel, LOCATION_GRAVE));
    e2.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 1)));
    Ok(())
}
