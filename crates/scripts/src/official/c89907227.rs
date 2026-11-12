// White Aura Bihamut
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 89907227;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// A copy on the field and two in the Extra Deck, Synchro Summoned with Formula Synchron (a Level 2 Synchro Tuner) and two Level 4
// Normal Monsters, with Raigeki in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (50091196, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, tkcon);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, tkcon2);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e3);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 3300, 3000, 10, RACE_FISH, ATTRIBUTE_WATER)
}

fn tkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_monster_full(tp, ID + 1, token_stats(), POS_FACEUP_DEFENSE, tp, 0));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    summon_token(duel, a.tp, POS_FACEUP_DEFENSE)
}

fn summon_token(duel: &mut Duel, tp: u8, position: u32) -> Script<()> {
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster_full(tp, ID + 1, token_stats(), position, tp, 0)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, tp, false, false, position)?;
    }
    Ok(())
}

fn tknfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TOKEN) || c.is_original_type(duel, TYPE_TOKEN)
}

fn tkcon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_turn_player(1 - tp) && !duel.is_existing_matching_card(tknfilter, tp, LOCATION_ONFIELD, 0, 1, None)
}

fn tkop2(duel: &mut Duel, a: &Args) -> Script<()> {
    summon_token(duel, a.tp, POS_FACEUP)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_reason(duel, REASON_DESTROY)
        && a.rp == 1 - tp
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
            && duel.is_existing_matching_card(|c, duel| c.is_code(duel, ID + 1), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}
