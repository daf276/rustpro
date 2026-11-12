// Saber Slash
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 11052544;

// X-Saber monsters (Anu Piranha) on the field to count, and cards on the opponent's field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (23115241, MZONE),
    (23115241, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attack_pos(duel) && c.is_set_card(duel, &[SET_X_SABER])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, 0, None);
        e.set_label(duel, ct as i64);
        return Ok(duel.is_existing_matching_card(
            aux::faceup_filter(aux::TRUE),
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            ct,
            Some(c.into()),
        ));
    }
    let ct = e.get_label(duel);
    let sg = duel.get_matching_group(
        aux::faceup_filter(aux::TRUE),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(c.into()),
    );
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), ct as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, 0, None);
    let g = duel.get_matching_group(
        aux::faceup_filter(aux::TRUE),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if g.len() >= ct {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg = g.select(duel, tp, ct as u8, ct as u8, None)?;
        duel.hint_selection(sg);
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}
