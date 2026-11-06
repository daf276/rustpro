// Trap Hole
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let Some(eg) = a.eg else { return Ok(false) };
    let Some(tc) = eg.get_first() else { return Ok(false) };
    if let Some(chkc) = chkc {
        return Ok(chkc == tc);
    }
    if chk == 0 {
        return Ok(a.ep != a.tp
            && tc.is_faceup(duel)
            && tc.get_attack(duel) >= 1000
            && tc.is_on_field(duel)
            && tc.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) && tc.get_attack(duel) >= 1000 {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
