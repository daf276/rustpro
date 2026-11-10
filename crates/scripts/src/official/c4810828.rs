// Sauravis, the Ancient and Ascended
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 4810828;

pub const LISTED_NAMES: &[u32] = &[37626500];

// A copy in the hand to negate the opponent's Mage Power targeting a monster, and one on the field to negate their Special
// Summon by Dark Grepher.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (70781052, MZONE),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
    (14536035, HAND | OPPONENT),
    (70781052, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, |duel, a, chk, _| {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
        }
        Ok(chk == 0)
    });
    e1.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_SPSUMMON);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.ep == 1 - a.tp && duel.get_current_chain() == 0);
    e2.set_cost(duel, Cost::self_to_hand);
    e2.set_target(duel, negsumtg);
    e2.set_operation(duel, negsumop);
    c.register_effect(duel, e2);
}

fn negconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    if !(a.rp == 1 - tp && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) && duel.is_chain_negatable(ev)) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(ev, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| negconfilter(c, duel, tp), 1, None))
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, CardId::is_able_to_remove, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(eg), eg.len() as u8, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(eg), eg.len() as u8, a.tp, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
