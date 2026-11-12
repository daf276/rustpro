// Stellarknight Constellar Diamond
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 9272381;

// Xyz Summoned from the Extra Deck with three Level 5 LIGHT monsters (Orion the Battle King) or over Tellarknight Ptolemaeus.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2971090, MZONE),
    (2971090, MZONE),
    (2971090, MZONE),
    (18326736, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, tp, xyzc| {
        c.is_faceup(duel)
            && c.is_set_card_full(duel, &[SET_TELLARKNIGHT], Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && !c.is_summon_code(duel, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp, &[ID])
            && duel.is_phase(PHASE_MAIN2)
    });
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_TO_GRAVE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_DECK, LOCATION_DECK);
    e1.set_condition(duel, effcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_DISCARD_DECK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_condition(duel, effcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_TO_HAND_REDIRECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_GRAVE, LOCATION_GRAVE);
    e3.set_value(duel, LOCATION_REMOVED as i32);
    e3.set_condition(duel, effcon);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_condition(duel, condition);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, stock::negate_destroy_target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_overlay_count(duel) > 0
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && a.re
            .is_some_and(|re| re.is_monster_effect(duel) && re.get_handler(duel).is_attribute(duel, ATTRIBUTE_DARK))
        && duel.is_chain_negatable(a.ev as u8)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
