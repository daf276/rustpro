// Ghost Mourner & Moonlit Chill
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, OPPONENT};

const ID: u32 = 52038441;

// Fusionist, Petit Angel, Mystical Sheep #2 and Polymerization in the opponent's hand and Extra Deck to Special Summon, and Raigeki
// to destroy what is negated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (12580477, HAND),
    (1641882, EXTRA | OPPONENT),
    (24094653, HAND | OPPONENT),
    (38142739, HAND | OPPONENT),
    (83464209, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let g = CardSet::default();
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CUSTOM + ID);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    e1.set_label_object_group(duel, Some(g));
    c.register_effect(duel, e1);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1a.set_range(duel, LOCATION_ALL);
    e1a.set_label_object(duel, Some(e1));
    e1a.set_operation(duel, regop);
    c.register_effect(duel, e1a);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_MZONE)
        && (c.has_non_zero_attack(duel) || c.is_negatable_monster(duel))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, e, tp), None);
    let Some(e1) = e.get_label_object(duel) else { return Ok(()) };
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let mut g = e1.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e1.set_label_object_group(duel, Some(g));
    if duel.get_flag_effect(tp, ID) == 0 {
        duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
        duel.raise_event(a.eg.unwrap_or_default(), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| cfilter(c, duel, e, tp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && cfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tg = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        g.select(duel, tp, 1, 1, None)?
    } else {
        g
    };
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if tc.is_negatable_monster(duel) {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    }
    if tc.is_faceup(duel) {
        let fid = c.get_field_id(duel);
        let reset = RESET_EVENT | RESET_MSCHANGE | RESET_OVERLAY | RESET_TURN_SET | RESET_PHASE | PHASE_END;
        tc.register_flag_effect_full(duel, ID, reset, 0, 1, i64::from(fid), 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_LEAVE_FIELD);
        e1.set_label(duel, i64::from(fid));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_operation(duel, leaveop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn leaveop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if a.eg.is_some_and(|eg| eg.is_contains(tc)) && tc.get_flag_effect_label(duel, ID) == Some(e.get_label(duel)) {
        let p = tc.get_previous_controler(duel);
        if duel.damage(p, tc.get_base_attack(duel) as u32, REASON_EFFECT)? > 0 {
            duel.hint(HINT_CARD, 0, ID.into());
        }
        tc.reset_flag_effect(duel, ID);
        e.reset(duel);
    }
    Ok(())
}
