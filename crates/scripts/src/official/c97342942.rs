// Ectoplasmer
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 97342942;

// In the hand to activate, with Normal Monsters (Mystical Elf) on both sides to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (15025844, MZONE), (15025844, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, tribtg);
    e1.set_operation(duel, tribop);
    c.register_effect(duel, e1);
}

fn tribtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let turn_player = duel.get_turn_player();
    duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, turn_player, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - turn_player, 0);
    Ok(false)
}

fn tribop(duel: &mut Duel, _: &Args) -> Script<()> {
    let turn_player = duel.get_turn_player();
    let filter = aux::faceup_filter(CardId::is_releasable_by_effect);
    let g = duel.select_release_group_full(turn_player, filter, (1, 1), false, ReleaseOptions::new(turn_player))?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(sc);
    if duel.release(sc, REASON_EFFECT)? > 0 {
        let atk = (sc.get_text_attack(duel) + 1) / 2;
        if atk > 0 {
            duel.damage(1 - turn_player, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
