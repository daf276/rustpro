// Number 2: Ninja Shadow Mosquito
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 32453837;

pub const COUNTER_PLACE_LIST: &[u16] = &[0x1101];
pub const XYZ_NUMBER: u32 = 2;
const HALLUCINATION_COUNTER: u16 = 0x1101;

// In the Extra Deck, Xyz Summoned with Level 2 monsters, and an opponent's copy to attack into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (7892180, MZONE),
    (59053232, MZONE),
    (68638985, MZONE),
    (ID, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_MUST_ATTACK);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, ctdmgtg);
    e4.set_operation(duel, ctdmgop);
    c.register_effect(duel, e4);
}

fn hcounter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_counter(duel, HALLUCINATION_COUNTER) > 0 && c.get_attack(duel) > 0
}

fn ctdmgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = e.get_handler(duel).check_remove_overlay_card(duel, tp, 1, REASON_EFFECT)
        && duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None);
    let b2 = duel.is_existing_matching_card(hcounter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])?;
    e.set_label(duel, op.unwrap_or(0) as i64);
    if op == Some(1) {
        e.set_category(duel, CATEGORY_COUNTER);
        duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, tp, i32::from(HALLUCINATION_COUNTER));
    } else {
        e.set_category(duel, CATEGORY_DAMAGE);
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    }
    Ok(false)
}

fn ctdmgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.get_label(duel) == 1 {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0 {
            let cg = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
            if cg.is_empty() {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
            let Some(tc) = cg.select(duel, tp, 1, 1, None)?.get_first() else {
                return Ok(());
            };
            duel.hint_selection_full(tc, true);
            if tc.add_counter(duel, HALLUCINATION_COUNTER, 1) {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, EFFECT_DISABLE);
                e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, HALLUCINATION_COUNTER) > 0);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
            }
        }
    } else {
        let g = duel.get_matching_group(hcounter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if g.is_empty() {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.hint_selection_full(tc, true);
        duel.damage(1 - tp, tc.get_attack(duel).max(0) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
