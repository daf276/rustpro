// Trickstar Bella Madonna
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 41302052;

pub const LISTED_SERIES: &[u16] = &[SET_TRICKSTAR];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// On the field with Trickstar monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (77307161, GRAVE), (14365823, GRAVE), (298846, GRAVE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_handler(duel) && te.is_activated(duel) && imcon(duel, e)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn imcon(duel: &Duel, e: EffectId) -> bool {
    let c = e.get_handler(duel);
    c.is_link_summoned(duel) && c.get_linked_group(duel).filter_count(duel, CardId::is_monster, None) == 0
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_link_monster(duel) && c.get_linked_group(duel).filter_count(duel, CardId::is_monster, None) == 0
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && c.is_monster(duel)
}

fn damount(duel: &Duel, tp: u8) -> i32 {
    duel.get_matching_group(damfilter, tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32
        * 200
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(damfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let dam = damount(duel, tp);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = damount(duel, a.tp);
    duel.damage(p, dam as u32, REASON_EFFECT)?;
    Ok(())
}
