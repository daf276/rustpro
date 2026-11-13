// Vorticular Drumgon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 739444;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, scard: CardId, sumtype: u64, tp: u8) -> bool {
        matfilter(c, duel, scard, sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with three DARK Dragons (Black Stone of Legend, Black Metal Dragon).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (66574418, MZONE),
    (66574418, MZONE),
    (66574418, MZONE),
    (93969023, MZONE),
    (93969023, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DISABLE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e2.set_returning_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, scard: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 0);
    Ok(())
}

fn disop(duel: &mut Duel, a: &Args) -> Script<u32> {
    Ok(a.e.get_handler(duel).get_linked_zone_full(duel, a.tp) & 0xff0000ff)
}
