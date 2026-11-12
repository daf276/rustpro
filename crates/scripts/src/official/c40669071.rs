// Excode Talker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 40669071;

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, lztg);
    e1.set_operation(duel, lzop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, tgtg);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_property(duel, 0);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, 500);
    c.register_effect(duel, e3);
}

fn lzfilter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) > 4
}

fn lztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct1 = duel.get_matching_group_count(lzfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let ct2 = duel.get_location_count(tp, LOCATION_MZONE) + duel.get_location_count(1 - tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ct1 > 0 && ct2 >= ct1 as i32);
    }
    let dis = duel
        .select_disable_field(tp, ct1 as u8, LOCATION_MZONE as u16, LOCATION_MZONE as u16, 0)?
        .unwrap_or(0);
    duel.hint(HINT_ZONE, tp, u64::from(dis));
    a.e.set_label(duel, i64::from(dis));
    Ok(false)
}

fn lzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, label) = (a.e.get_handler(duel), a.e.get_label(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_DISABLE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_returning_operation(duel, |duel, a| Ok(a.e.get_label(duel) as u32));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_label(duel, label);
    c.register_effect(duel, e1);
    Ok(())
}

fn tgtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_linked_group(duel).contains(c)
}
