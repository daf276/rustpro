// Pitknight Filly
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 25919316;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from Link Devotees (Effect Monsters, Top arrow, so pointing at its Extra Monster Zone) and
// Sangans, destroyed by Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (48068378, MZONE),
    (48068378, MZONE),
    (48068378, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, doublecon);
    e1.set_target(duel, doubletg);
    e1.set_operation(duel, doubleop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn doubleconfilter(c: CardId, duel: &Duel, oc: CardId) -> bool {
    c.get_linked_group(duel).is_contains(oc)
}

fn doublecon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_able_to_enter_bp()
        && duel.is_existing_matching_card(
            |tc, duel| doubleconfilter(tc, duel, c),
            a.tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn doublefilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, 1500)
}

fn doubletg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && doublefilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(doublefilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    duel.select_target(tp, doublefilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn doubleop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
        e2.set_condition(duel, |duel, a| {
            let handler = a.e.get_handler(duel);
            handler
                .get_battle_target(duel)
                .is_some_and(|bc| bc.is_controler(duel, 1 - a.e.get_handler_player(duel)))
        });
        e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_turn_id(duel) == duel.get_turn_count() - 1
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}
