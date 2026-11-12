// Trickstar Holly Angel
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 32448765;

pub struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// On the field with Trickstar Links beside it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (77307161, MZONE),
    (14365823, MZONE),
    (41302052, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_condition(duel, damcon);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, indtg);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_DAMAGE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_condition(duel, atkcon);
    e5.set_operation(duel, atkop);
    c.register_effect(duel, e5);
}

fn cfilter(c: CardId, duel: &Duel, g: CardSet) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && g.is_contains(c)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, lg), 1, None)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.damage(1 - a.tp, 200, REASON_EFFECT)?;
    Ok(())
}

fn indtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp
        && a.r & REASON_EFFECT != 0
        && a.re
            .is_some_and(|re| re.is_monster_effect(duel) && re.get_handler(duel).is_set_card(duel, &[SET_TRICKSTAR]))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, a.ev as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
