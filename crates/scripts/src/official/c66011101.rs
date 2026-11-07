// Number 60: Dugares the Timeless
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE};

const ID: u32 = 66011101;

// In the Extra Deck, Xyz Summoned with Mystical Elf and Flying Kamakiri #2, and monsters in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, GRAVE),
    (15025844, GRAVE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_player_can_draw(tp, 2);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    let b3 = duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
            e.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(tp);
            duel.set_target_param(2);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
            duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_ATKCHANGE);
            e.set_property(duel, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let c = e.get_handler(duel);
    let (skipped_phase, skip_effect_code) = match op {
        1 => (PHASE_DRAW, EFFECT_SKIP_DP),
        2 => (PHASE_MAIN1, EFFECT_SKIP_M1),
        _ => (PHASE_BATTLE, EFFECT_SKIP_BP),
    };
    let turn_ct = duel.get_turn_count();
    let curr_phase = duel.get_current_phase();
    let skip_next_turn = (duel.is_turn_player(tp) && curr_phase >= skipped_phase) || skipped_phase == PHASE_BATTLE;
    let reset_ct = if skip_next_turn { 2 } else { 1 };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3 + op as u64));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, skip_effect_code);
    e1.set_target_range(duel, 1, 0);
    if skip_next_turn {
        e1.set_label(duel, i64::from(turn_ct));
        e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
    }
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, reset_ct);
    duel.register_effect(e1, tp);
    match op {
        1 => {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
            if duel.draw(p, d, REASON_EFFECT)? == 2 {
                duel.shuffle_hand(tp)?;
                duel.break_effect()?;
                duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
            }
        }
        2 => {
            if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
            }
        }
        _ => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
            let tc = duel
                .select_matching_card(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?
                .get_first();
            if let Some(tc) = tc {
                duel.hint_selection(tc);
                let e2 = duel.create_effect(c);
                e2.set_type(duel, EFFECT_TYPE_SINGLE);
                e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e2.set_code(duel, EFFECT_SET_ATTACK_FINAL);
                e2.set_value(duel, tc.get_attack(duel) * 2);
                e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect(duel, e2);
            }
        }
    }
    Ok(())
}
