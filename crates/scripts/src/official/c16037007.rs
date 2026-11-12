// Number 74: Master of Blades
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 16037007;
pub const XYZ_NUMBER: u32 = 74;

// In the Extra Deck, Xyz Summoned with two Level 7 Normal Monsters (Gaia The Fierce Knight), with the opponent's Mage Power and Mind
// Control in hand to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6368038, MZONE),
    (6368038, MZONE),
    (6368038, MZONE),
    (6368038, MZONE),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return false };
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.contains(c)) && duel.is_chain_negatable(a.ev as u8)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
