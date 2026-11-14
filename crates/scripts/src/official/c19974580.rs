// Iron Chain Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 19974580;

pub const LISTED_SERIES: &[u16] = &[SET_IRON_CHAIN];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned (Level 6) with Angel Trumpeter (a Level 4 Tuner) and Hourglass of Life (Level 2), or Tune Warrior (Level 3
// Tuner) and Lady of Faith (Level 3). It banishes itself, an Iron Chain monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (8783685, MZONE),
    (74093656, MZONE),
    (17358176, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, attg);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, |_, a| a.ep != a.tp);
    e2.set_target(duel, ddtg);
    e2.set_operation(duel, |duel, a| {
        duel.discard_deck(1 - a.tp, 3, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn rfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_IRON_CHAIN]) && c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    let location = if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
        LOCATION_MZONE
    } else {
        LOCATION_GRAVE
    };
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, location as i32);
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(rfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let ct = duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    if ct > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct * 200);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn ddtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, 1 - a.tp, 3);
    Ok(false)
}
