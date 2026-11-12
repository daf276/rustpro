// Hyper Psychic Riser
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 99115354;

// In the Extra Deck, Synchro Summoned with Tune Warrior (Level 3 Tuner) and Wood Clown (Level 3), destroyed by the opponent's Dark Hole with an EARTH Warrior Tuner and non-Tuner in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (74093656, MZONE),
    (17511156, MZONE),
    (74093656, GRAVE),
    (17511156, GRAVE),
    (53129443, HAND | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_target_card(duel, atktg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn atktg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_attack(duel) < e.get_handler(duel).get_attack(duel)
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    re.get_activate_location(duel) == LOCATION_MZONE
        && re.is_monster_effect(duel)
        && re.get_handler(duel).get_attack(duel) > e.get_handler(duel).get_attack(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.e.get_handler(duel).is_reason(duel, REASON_DESTROY)
}

fn thfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_monster(duel) && c.is_can_be_effect_target(duel, e) && c.is_able_to_hand(duel)
}

fn thcheck(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 1
            && sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1
            && sg.is_exists(duel, |c, duel| c.is_type(duel, TYPE_TUNER), 1, None)
            && sg.is_exists(duel, |c, duel| !c.is_type(duel, TYPE_TUNER), 1, None),
        false,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| thfilter(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| thcheck(duel, sg)));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| thcheck(duel, sg), tp, HINTMSG_ATOHAND)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
