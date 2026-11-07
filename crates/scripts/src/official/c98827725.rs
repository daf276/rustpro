// The Phantom Knights of Shade Brigandine
use crate::prelude::*;
use crate::testing::{DECK, SZONE};

const ID: u32 = 98827725;

// Set, to be activated; a second copy in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(CardId::is_trap, a.e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
    });
    c.register_effect(duel, e2);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(
        SET_THE_PHANTOM_KNIGHTS,
        TYPE_MONSTER + TYPE_NORMAL,
        0,
        300,
        4,
        RACE_WARRIOR,
        ATTRIBUTE_DARK,
    );
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if c.is_relate_to_effect(duel, a.e) && can_summon(duel, tp) {
        c.add_monster_attribute(duel, TYPE_NORMAL);
        duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP_DEFENSE)?;
        c.add_monster_attribute_complete(duel);
        duel.special_summon_complete()?;
    }
    Ok(())
}
