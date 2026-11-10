// Barricadeborg Blocker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 13117073;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == sg.len()
    }
}

// In the Extra Deck, Link Summoned from two different Normal Monsters, with Mausoleum of White in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (11091375, MZONE),
    (43096270, MZONE),
    (24382602, GRAVE),
    (24382602, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, regcost);
    e1.set_target(duel, regtg);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_ONFIELD, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_faceup(duel) && c.is_spell(duel));
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
}

fn regcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(thfilter, a.tp, LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_operation(duel, thop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && c.is_spell(duel) && c.is_type(duel, TYPE_FIELD + TYPE_CONTINUOUS)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, PLAYER_ALL, u64::from(ID));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
