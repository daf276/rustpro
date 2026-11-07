// Called by the Grave
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, OPPONENT};

const ID: u32 = 24224830;

// In the hand, banishing Ash Blossom & Joyous Spring from the opponent's GY, whose copy in the opponent's hand it then negates.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (14558127, GRAVE | OPPONENT), (14558127, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? != 0 {
        let c = e.get_handler(duel);
        let code = tc.get_original_code_rule(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
        e1.set_target_card(duel, |duel, e, c| c.get_original_code_rule(duel) == e.get_label(duel) as u32);
        e1.set_label(duel, code.into());
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_CHAIN_SOLVING);
        e2.set_condition(duel, discon);
        e2.set_operation(duel, disop);
        e2.set_label(duel, code.into());
        e2.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    re.is_monster_effect(duel) && re.get_handler(duel).get_original_code_rule(duel) == a.e.get_label(duel) as u32
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.negate_effect(a.ev as u8);
    Ok(())
}
