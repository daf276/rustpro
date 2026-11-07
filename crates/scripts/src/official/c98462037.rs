// Sky Striker Ace - Azalea
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 98462037;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(lc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Link Summoned with Sky Striker Ace - Raye and Roze, with Spells in the GY to banish and to keep it on
// the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (26077387, MZONE),
    (37351133, MZONE),
    (98338152, GRAVE),
    (98338152, GRAVE),
    (98338152, GRAVE),
    (52340444, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::lnklimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, btcon);
    e3.set_cost(duel, btcost);
    e3.set_target(duel, bttg);
    e3.set_operation(duel, btop);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && c.is_relate_to_effect(duel, e)
        && duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) <= 3
    {
        duel.break_effect()?;
        duel.send_to_grave(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn btcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_controler(duel, 1 - a.tp))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_remove_as_cost(duel)
}

fn btcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn bttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn btop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
        && bc.is_controler(duel, 1 - a.tp)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
