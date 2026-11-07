// Dark Rebellion Xyz Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 16195942;

// On the field with Xyz materials, to halve Sangan's ATK.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (26202165, MZONE | OPPONENT)];

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
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

pub(crate) fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.has_non_zero_attack(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        let c = e.get_handler(duel);
        let val = (f64::from(tc.get_attack(duel)) / 2.0).ceil() as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        e1.set_value(duel, val);
        tc.register_effect(duel, e1);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_UPDATE_ATTACK);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            e2.set_value(duel, val);
            c.register_effect(duel, e2);
        }
    }
    Ok(())
}
