// Griffoh
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 97462632;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

struct Tribute;

impl WholeLevelTribute for Tribute {
    fn condition(c: CardId, duel: &Duel, _: EffectId) -> bool {
        c.is_level(duel, &[8])
    }
}

// In the hand as the whole tribute for Magician of Black Chaos (Level 8) with Black Magic Ritual. No Normal Monster or ported
// card is a Quick-Play Spell or Trap that mentions "Ritual of Light and Darkness", so the Set option never resolves.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (76792184, HAND), (30208479, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::RitualMaterial];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    Ritual::add_whole_level_tribute::<Tribute>(duel, c);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_quick_play_spell(duel) || c.is_trap(duel)) && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL) && c.is_ssetable(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b2 = duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(true);
    }
    let op = duel.select_effect(tp, &[(true, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => e.set_category(duel, 0),
        Some(2) => {
            e.set_category(duel, CATEGORY_SET);
            duel.set_operation_info(0, CATEGORY_SET, None, 1, tp, LOCATION_DECK as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    match e.get_label(duel) {
        1 => {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 3));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
            e1.set_target_range(duel, 1, 0);
            e1.set_value(
                duel,
                Value::ChangeDamage(|duel, e, _, val, r, _, _| {
                    if r & (REASON_BATTLE | REASON_EFFECT) == 0 {
                        return val;
                    }
                    duel.hint(HINT_CARD, 0, u64::from(ID));
                    e.reset(duel);
                    0
                }),
            );
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if let Some(sc) = g.get_first()
                && duel.sset(tp, sc)? > 0
            {
                let eff_code = if sc.is_quick_play_spell(duel) {
                    EFFECT_QP_ACT_IN_SET_TURN
                } else {
                    EFFECT_TRAP_ACT_IN_SET_TURN
                };
                let e2 = duel.create_effect(c);
                e2.set_description(duel, aux::stringid(ID, 4));
                e2.set_type(duel, EFFECT_TYPE_SINGLE);
                e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
                e2.set_code(duel, eff_code);
                e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e2);
            }
        }
        _ => {}
    }
    Ok(())
}
