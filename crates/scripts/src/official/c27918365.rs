// World Legacy - "World Crown"
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 27918365;

// In the hand, to Special Summon to the zone a Link Monster (Haggard Lizardose, Cyber Dragon Sieger) points to. Its search needs a
// "World Legacy" Spell/Trap, none of which is ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (9763474, MZONE), (46724542, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target_range(duel, POS_FACEUP_DEFENSE, 0);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_value(
        duel,
        Value::SpSummonProcedure(|duel, _, c| (0, aux::get_mm_zones_pointed_to(duel, c.get_controler(duel)) as i32)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_RELEASE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_normal_summoned(duel));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let summon_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_SUMMON_LOCATION);
    trig_loc == LOCATION_MZONE
        && summon_loc == LOCATION_EXTRA
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(ev)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
