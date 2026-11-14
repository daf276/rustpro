// Vahram, the Magistus Divinity Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 61272280;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Two in the Extra Deck, Synchro Summoned with Infernoble Knight Oliver (a Level 4 Tuner) and Noble Knight Artorigus (Level 4), and two in the
// Graveyard for Zoroa, the Magistus Verethragna to equip, with Dark Hole in either hand or Deck to destroy it and attackers on their side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (58346901, MZONE),
    (58346901, MZONE),
    (92125819, MZONE),
    (37260677, MZONE),
    (37260677, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, DECK | OPPONENT),
    (12580477, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_value(duel, Value::EffectPlayer(indesvalue));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon1);
    e2.set_target(duel, destg1);
    e2.set_operation(duel, desop1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon2);
    e3.set_target(duel, destg2);
    e3.set_operation(duel, desop2);
    c.register_effect(duel, e3);
}

fn indesvalue(duel: &Duel, e: EffectId, re: EffectId, rp: u8) -> bool {
    re.is_spell_trap_effect(duel) && rp == 1 - e.get_handler_player(duel)
}

fn descon1(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_synchro_summoned(duel)
}

fn destg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn descon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|ec| ec.get_battle_target(duel).is_some())
}

fn destg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ec = a.e.get_handler(duel).get_equip_target(duel);
    unported!(duel, ec.is_none());
    let bc = ec.and_then(|ec| ec.get_battle_target(duel));
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let ec = c.get_equip_target(duel);
    unported!(duel, ec.is_none());
    let bc = ec.and_then(|ec| ec.get_battle_target(duel));
    if let Some(bc) = bc
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
