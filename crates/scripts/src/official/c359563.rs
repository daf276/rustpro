// Evilswarm Nightmare
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 359563;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with Mechanicalchaser (Level 4, DARK), with Link Apple and Elfnote Lucina on the opponent's side
// to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7359741, MZONE),
    (7359741, MZONE),
    (7359741, MZONE),
    (7359741, MZONE),
    (7925734, HAND | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: Option<EffectId>, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel) && c.is_summon_player(duel, 1 - tp) && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| filter(c, duel, None, tp), 1, None));
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(eg), eg.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, Some(e), tp), None);
    duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    Ok(())
}
