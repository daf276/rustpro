// One-Eyed Skill Gainer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 75620895;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters (Prevent Rat), with the opponent's Divine Dragon Knight Felgrand to copy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (1639384, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_NO_TURN_RESET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let code = tc.get_original_code(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, code as i32);
        c.register_effect(duel, e1);
        c.copy_effect(duel, code, RESET_EVENT | RESETS_STANDARD, 1);
    }
    Ok(())
}
