// El Shaddoll Construct
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 20366274;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SHADDOLL];

// In the Extra Deck, Fusion Summoned by Polymerization with Shaddoll Squamata and Mokey Mokey (LIGHT), with Shaddoll monsters
// in the Deck to send, and sent to the GY by Dark Hole drawn later with Shaddoll Fusion there to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (30328508, HAND),
    (27288416, HAND),
    (30328508, DECK),
    (77723643, DECK),
    (53129443, DECK),
    (53129443, DECK),
    (44394295, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c94977269::add_procedure::<ATTRIBUTE_LIGHT>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_BATTLE_START);
    e4.set_condition(duel, descon);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    super::c94977269::register_to_hand(duel, c, aux::stringid(ID, 2));
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_special_summoned(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
