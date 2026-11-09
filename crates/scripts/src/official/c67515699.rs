// K9-66X "Jacks"
use super::c92248362::chainfilter;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 67515699;

// In the Extra Deck, Xyz Summoned with K9-17 Izuna and K9-00 Lupis, and on the field, against an opponent's Elfnote Lucina in the
// hand, Graveyard and field that activates there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (92248362, MZONE),
    (91025875, MZONE),
    (13597785, HAND | OPPONENT),
    (13597785, GRAVE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_DESTROY);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1a.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1a.set_target(duel, destg);
    e1a.set_operation(duel, desop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1b.set_code(duel, EVENT_CHAINING);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, descon);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, doubledamcon);
    e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && trig_loc & (LOCATION_HAND | LOCATION_GRAVE) > 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_monster(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn doubledamcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_set_card(duel, &[SET_K9])
        && c.is_xyz_monster(duel)
        && duel.get_custom_activity_count(ID, 1 - a.e.get_handler_player(duel), ACTIVITY_CHAIN) > 0
}
