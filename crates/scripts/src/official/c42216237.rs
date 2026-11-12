// Angel of Zera
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 42216237;

// On the field and in the Extra Deck, with Paleozoic Dinomischus and Primite Drillbeam to banish it, and monsters to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (42026772, MZONE),
    (14851496, MZONE),
    (38761908, SZONE),
    (38761908, SZONE),
    (29095457, HAND),
    (29095457, HAND),
    (10071456, HAND),
    (10071456, HAND),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_operation(duel, spreg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_field_group_count(c.get_controler(duel), 0, LOCATION_REMOVED) as i32 * 100
}

fn spreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let turn = i64::from(duel.get_turn_count());
    a.e.set_label(duel, turn);
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.e.get_label_object(duel)
        .is_some_and(|e2| e2.get_label(duel) != i64::from(duel.get_turn_count()))
        && c.get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
