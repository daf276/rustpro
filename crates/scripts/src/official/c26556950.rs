// Number 84: Pain Gainer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_DAMAGE;

const ID: u32 = 26556950;
pub const XYZ_NUMBER: u32 = 84;

// In the Extra Deck, Xyz Summoned with two Level 11 monsters, with the opponent's Dark Hole and Raigeki to activate and monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (25833572, MZONE),
    (25833572, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
// The damage effect has no chain link, so its damage is checked by message.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::Message(MSG_DAMAGE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(11);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e1.set_value(duel, Value::Card(defval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, damcon);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let rk = c.get_rank(duel);
    c.is_faceup(duel)
        && c.get_overlay_count(duel) >= 2
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && (8..=10).contains(&rk)
}

fn defval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = duel.get_matching_group(CardId::is_faceup, c.get_controler(duel), LOCATION_MZONE, 0, None);
    g.get_sum(duel, |c, duel| i64::from(c.get_rank(duel))) as i32 * 200
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.rp == a.tp || !a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) {
        return Ok(());
    }
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_CHAIN, 0, 1);
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) > 0
        && a.ep != a.tp
        && c.get_flag_effect(duel, ID) != 0
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.damage(1 - a.tp, 600, REASON_EFFECT)?;
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, def: i32) -> bool {
    c.is_faceup(duel) && c.is_defense_below(duel, def)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let def = a.e.get_handler(duel).get_defense(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| desfilter(c, duel, def), tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, def), tp, 0, LOCATION_MZONE, None);
    let count = g.len() as u8;
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), count, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let def = c.get_defense(duel);
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, def), tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
