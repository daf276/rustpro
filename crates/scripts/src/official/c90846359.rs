// Rivalry of Warlords
use engine::constants::MSG_ANNOUNCE_RACE;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 90846359;

// In the hand, with Cyberse, Zombie and Fiend monsters (Protron, Skull Servant, Sangan) on both
// sides to be sent to the Graveyard for sharing a Type.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (24154052, MZONE),
    (32274490, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ANNOUNCE_RACE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_STANDBY_PHASE, TIMING_STANDBY_PHASE | TIMINGS_CHECK_MONSTER);
    e1.set_target(duel, acttg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EVENT_ADJUST);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_operation(duel, adjustop);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EFFECT_FORCE_SPSUMMON_POSITION);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_target_range(duel, 1, 1);
    e4.set_target_spsummon(duel, sumlimit);
    e4.set_value(duel, POS_FACEDOWN as i32);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_CANNOT_SUMMON);
    c.register_effect(duel, e5);
    let e6 = e4.clone_effect(duel);
    e6.set_code(duel, EFFECT_CANNOT_FLIP_SUMMON);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_FIELD);
    e7.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e7.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e7.set_range(duel, LOCATION_SZONE);
    e7.set_target_range(duel, 1, 1);
    e7.set_value(duel, Value::EffectPlayer(activatelimit));
    c.register_effect(duel, e7);
}

// s[0] and s[1]: the Type each player's monsters share.
fn key(tp: u8) -> u64 {
    u64::from(ID) << 16 | u64::from(tp)
}

fn acttg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_script_state(key(0), 0);
    duel.set_script_state(key(1), 0);
    Ok(false)
}

fn getrace(duel: &Duel, g: CardSet) -> u64 {
    g.get_bitwise_or(duel, CardId::get_race)
}

fn sumlimit(duel: &Duel, _: EffectId, c: CardId, sump: u8, _: u32, _: u8, targetp: u8, _: Option<EffectId>) -> bool {
    let player = if targetp == PLAYER_NONE { sump } else { targetp };
    let rc = getrace(duel, duel.get_matching_group(CardId::is_faceup, player, LOCATION_MZONE, 0, None));
    rc != 0 && c.get_race(duel) != rc
}

fn activatelimit(duel: &Duel, _: EffectId, re: EffectId, tp: u8) -> bool {
    let race = duel.get_script_state(key(tp)) as u64;
    re.has_self_change_position_cost(duel) && race > 0 && re.get_handler(duel).is_race_except(duel, race)
}

fn adjustop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let phase = duel.get_current_phase();
    if (phase == PHASE_DAMAGE && !duel.is_damage_calculated()) || phase == PHASE_DAMAGE_CAL {
        return Ok(());
    }
    let mut sides = [
        (tp, duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None)),
        (1 - tp, duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None)),
    ];
    for (player, g) in &mut sides {
        if g.is_empty() {
            duel.set_script_state(key(*player), 0);
            continue;
        }
        let mut rac = getrace(duel, *g);
        if rac & (rac - 1) != 0 {
            let stored = duel.get_script_state(key(*player)) as u64;
            if stored == 0 || stored & rac == 0 {
                duel.hint(HINT_SELECTMSG, *player, aux::stringid(ID, 0));
                rac = duel.announce_race(*player, 1, rac)?;
            } else {
                rac = stored;
            }
        }
        g.keep_matching(duel, |c, duel| c.get_race(duel) != rac, None);
        duel.set_script_state(key(*player), rac as i64);
    }
    let mut readjust = false;
    for (player, g) in sides {
        if !g.is_empty() {
            duel.send_to_grave_full(g, REASON_RULE, PLAYER_NONE, player)?;
            readjust = true;
        }
    }
    if readjust {
        duel.readjust()?;
    }
    Ok(())
}
