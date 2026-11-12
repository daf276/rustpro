// Number 35: Ravenous Tarantula
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 90162951;

pub const XYZ_NUMBER: u32 = 35;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 10 monsters (Carnot, Uria, Ten Thousand Dragon), with Sangan on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13567610, MZONE),
    (13567610, MZONE),
    (6007213, MZONE),
    (6007213, MZONE),
    (10000, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, damcon);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e5.set_code(duel, ID);
    c.register_effect(duel, e5);
}

fn val(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    (duel.get_lp(0) - duel.get_lp(1)).abs()
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.e.get_handler(duel).get_overlay_count(duel) > 0
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID as u64);
    duel.damage(1 - a.tp, 600, REASON_EFFECT)?;
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, atk)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let atk = a.e.get_handler(duel).get_attack(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| desfilter(c, duel, atk), tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, atk), tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let atk = c.get_attack(duel);
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, atk), a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
