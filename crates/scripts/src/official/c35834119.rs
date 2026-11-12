// Fish Lamp
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 35834119;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// On the field as the Tuner of a Synchro Summon of Iron Chain Dragon with White Dolphin, a Level 2 Normal Monster, and in the
// Graveyard to banish with a Fish Synchro Monster (Toxic Boxfish) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (92409659, MZONE),
    (92409659, MZONE),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (48882106, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r & REASON_SYNCHRO > 0
    });
    e1.set_target(duel, tkntg);
    e1.set_operation(duel, tknop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 2, RACE_FISH, ATTRIBUTE_FIRE)
}

fn tkntg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn tknop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_FISH)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_FISH));
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) < 2
        || !duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats())
    {
        return Ok(());
    }
    for _ in 0..2 {
        if let Some(token) = duel.create_token(tp, ID + 1) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_FISH) && c.is_type(duel, TYPE_SYNCHRO)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
