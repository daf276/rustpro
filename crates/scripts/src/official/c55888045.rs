// Number C106: Giant Red Hand
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 55888045;
pub const XYZ_NUMBER: u32 = 106;

// On the field with Xyz Import to attach the opponent's Numbers (Mannequin Cat, Djinn Buster), and effects of theirs on the field to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (54191698, MZONE | OPPONENT),
    (3790062, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_F);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, negcon);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let c = a.e.get_handler(duel);
    loc & LOCATION_ONFIELD != 0
        && !c.is_status(duel, STATUS_CHAINING)
        && c.get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_NUMBER]), 1, None)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e)
        || u32::from(duel.get_current_chain()) != a.ev + 1
        || c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? < 1
    {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    for tc in g {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}
