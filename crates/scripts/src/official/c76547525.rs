// Red Wyvern
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 76547525;

// In the Extra Deck, Synchro Summoned from Water Spirit (Level 1 Tuner) and Shapesnatch (Level 5), with a stronger monster of the
// opponent's (Pesanta, 2500 ATK).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (66429798, MZONE | OPPONENT),
    (66429798, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) > atk
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let atk = c.get_attack(duel);
    c.is_synchro_summoned(duel)
        && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, atk), a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let tg = g.get_max_group(duel, |c, duel| c.get_attack(duel).into()).map(|(tg, _)| tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty()
        && let Some((tg, _)) = g.get_max_group(duel, |c, duel| c.get_attack(duel).into())
    {
        if tg.len() > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = tg.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(sg);
            duel.destroy(sg, REASON_EFFECT)?;
        } else {
            duel.destroy(tg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
