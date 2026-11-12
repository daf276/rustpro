// Trickstar Black Catbat
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 94626871;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Trickstar Nightshades, with more on the field to be destroyed in its linked zones.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (86825114, MZONE), (86825114, MZONE), (86825114, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, damcon);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let seq = c.get_previous_sequence(duel) + if c.is_previous_controler(duel, tp) { 0 } else { 16 };
    c.is_reason(duel, REASON_DESTROY)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && zone >> seq & 1 != 0
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp, zone), 1, None))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    duel.damage(1 - a.tp, 200, REASON_EFFECT)?;
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.ep != a.tp && a.r & REASON_BATTLE == 0 && re.is_monster_effect(duel) && re.get_handler(duel).is_set_card(duel, &[SET_TRICKSTAR])
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_linked_group_count(duel) as i32;
    if ct <= 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -ct * 200);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
