// Trident Dragion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 39402797;

// In the Extra Deck, Synchro Summoned with Labradorite Dragon (Level 6 Tuner) and Luster Dragon (Level 4), with their copies to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62514770, MZONE),
    (62514770, MZONE),
    (62514770, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 2, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel.get_target_cards(e).unwrap_or_default();
    let c = e.get_handler(duel);
    let ct = duel.destroy(g, REASON_EFFECT)?;
    if ct > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, ct);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
