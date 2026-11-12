// Topologic Bomber Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 5821478;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Sangans (Effect Monsters), with monsters of the opponent to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    if c.is_location(duel, LOCATION_MZONE) {
        ec.get_linked_group(duel).contains(c)
    } else {
        ec.get_linked_zone_full(duel, c.get_previous_controler(duel)) & (1 << c.get_previous_sequence(duel)) != 0
    }
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    if eg.is_contains(a.e.get_handler(duel)) {
        return false;
    }
    let g = duel.get_matching_group(|c, duel| c.is_type(duel, TYPE_LINK), a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    g.iter().any(|tc| eg.is_exists(duel, |c, duel| cfilter(c, duel, tc), 1, None))
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) < 5
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(desfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    let c = e.get_handler(duel);
    let field_id = i64::from(c.get_field_id(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, field_id);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel)) && duel.get_attack_target().is_some()
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let damage = duel.get_attack_target().map_or(0, |bc| bc.get_base_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, damage);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = duel.get_attack_target()
        && bc.is_relate_to_battle(duel)
        && bc.is_faceup(duel)
    {
        let damage = bc.get_base_attack(duel) as u32;
        duel.damage(1 - a.tp, damage, REASON_EFFECT)?;
    }
    Ok(())
}
