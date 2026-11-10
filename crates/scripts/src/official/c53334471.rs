// Gozen Match
use engine::constants::MSG_ANNOUNCE_ATTRIB;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 53334471;

// In hand with monsters of several Attributes (Psychic Kappa WATER, Volcanic Rat FIRE, Dark Plant DARK) on both sides, and
// more of them in hand to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (7892180, MZONE),
    (33112041, MZONE),
    (13193642, MZONE),
    (7892180, MZONE | OPPONENT),
    (33112041, MZONE | OPPONENT),
    (13193642, MZONE | OPPONENT),
    (7892180, HAND),
    (33112041, HAND),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ANNOUNCE_ATTRIB)];

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
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EFFECT_FORCE_SPSUMMON_POSITION);
    e4.set_range(duel, LOCATION_SZONE);
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

// The `s[tp]` table: the Attribute each player is limited to.
fn state(tp: u8) -> u64 {
    u64::from(ID) << 16 | u64::from(tp)
}

fn acttg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_script_state(state(0), 0);
    duel.set_script_state(state(1), 0);
    Ok(false)
}

fn getattribute(duel: &Duel, g: CardSet) -> u32 {
    g.get_bitwise_or(duel, |c, duel| u64::from(c.get_attribute(duel))) as u32
}

// The summoning player is the target player's, which a nil target player stands for in Lua.
fn sumlimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, targetp: u8, _: Option<EffectId>) -> bool {
    let at = getattribute(duel, duel.get_matching_group(CardId::is_faceup, targetp, LOCATION_MZONE, 0, None));
    at != 0 && c.get_attribute(duel) != at
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
        let player = *player;
        if g.is_empty() {
            duel.set_script_state(state(player), 0);
            continue;
        }
        let mut att = getattribute(duel, *g);
        let previous = duel.get_script_state(state(player)) as u32;
        if att & att.wrapping_sub(1) != 0 {
            if previous == 0 || previous & att == 0 {
                duel.hint(HINT_SELECTMSG, player, aux::stringid(ID, 0));
                att = duel.announce_attribute(player, 1, att)?;
            } else {
                att = previous;
            }
        }
        g.remove_matching(duel, |c, duel| c.get_attribute(duel) == att, None);
        duel.set_script_state(state(player), i64::from(att));
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

fn activatelimit(duel: &Duel, _: EffectId, re: EffectId, tp: u8) -> bool {
    let at = duel.get_script_state(state(tp)) as u32;
    re.has_self_change_position_cost(duel) && at > 0 && re.get_handler(duel).is_attribute_except(duel, at)
}
