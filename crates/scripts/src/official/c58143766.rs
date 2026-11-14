// Volcanic Emission
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, SZONE};

const ID: u32 = 58143766;

pub const LISTED_SERIES: &[u16] = &[SET_VOLCANIC];

// Set, with Volcanic Troopers in the Deck to add or summon and on the field to burn with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (22411609, DECK),
    (22411609, DECK),
    (22411609, MZONE),
    (33112041, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_set_card(duel, &[SET_VOLCANIC])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)))
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PYRO) && c.is_faceup(duel) && c.get_base_attack(duel) > 0
}

// Whether each effect can be chosen, the once-per-turn flags skipped when the cost was not paid.
fn options(duel: &Duel, e: EffectId, tp: u8, cost_skip: bool) -> (bool, bool) {
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let b1 = (cost_skip || !duel.has_flag_effect(tp, ID))
        && duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, None);
    let b2 =
        (cost_skip || !duel.has_flag_effect(tp, ID + 1)) && duel.is_existing_target(damfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    (b1, b2)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, -100);
    let (b1, b2) = options(duel, a.e, a.tp, false);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && damfilter(chkc, duel));
    }
    let cost_skip = e.get_label(duel) != -100;
    let (b1, b2) = options(duel, e, tp, cost_skip);
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
        e.set_property(duel, 0);
        if !cost_skip {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DAMAGE);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        let tc = duel
            .select_target(tp, damfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
            .and_then(|g| g.get_first());
        let Some(tc) = tc else {
            unported!(duel, true);
            return Ok(false);
        };
        let mut dam = tc.get_base_attack(duel);
        let ctrl = tc.get_controler(duel);
        if ctrl == tp {
            dam /= 2;
        }
        duel.set_target_param(u32::from(ctrl));
        if !cost_skip {
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let ft = duel.get_location_count(tp, LOCATION_MZONE);
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
            let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, ft);
            let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
                return Ok(());
            };
            aux::to_hand_or_else(
                duel,
                sc,
                tp,
                |_, duel| ft > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, true, false),
                |duel| {
                    if duel.special_summon_step(sc, 0, tp, tp, true, false, POS_FACEUP)? {
                        aux::delayed_operation_full::<ReturnToHand>(duel, sc.into(), PHASE_END, 0, ID, e, tp, aux::stringid(ID, 5), 0);
                    }
                    duel.special_summon_complete()?;
                    Ok(())
                },
                aux::stringid(ID, 4),
            )?;
        }
        2 => {
            let Some(tc) = duel.get_first_target() else { return Ok(()) };
            if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
                let mut dam = tc.get_base_attack(duel);
                let ctrl: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
                if ctrl == tp {
                    dam /= 2;
                }
                duel.damage(1 - tp, dam as u32, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
