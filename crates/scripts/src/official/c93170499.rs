// Jurrac Megalo
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 93170499;

pub const LISTED_SERIES: &[u16] = &[SET_JURRAC];
pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand to Special Summon with Uraby on the field, and discard together; the only Jurrac monster other than Megalo
// that could be sent to the Graveyard is not ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (1784619, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_race(duel, RACE_DINOSAUR)),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, drwtg);
    e2.set_operation(duel, drwop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_BATTLE | REASON_EFFECT));
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn is_jurrac(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_JURRAC])
}

fn drwtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(discardable, tp, LOCATION_HAND, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 2 && g.is_exists(duel, is_jurrac, 1, None) && duel.is_player_can_draw(tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drwop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(discardable, tp, LOCATION_HAND, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| (sg.is_exists(duel, is_jurrac, 1, None), false);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_DISCARD)?;
    if sg.len() == 2 && duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)? > 0 {
        duel.break_effect()?;
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    is_jurrac(c, duel) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK | LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK | LOCATION_EXTRA)
}
