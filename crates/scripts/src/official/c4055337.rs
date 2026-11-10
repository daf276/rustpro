// Orcust Knightmare
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 4055337;
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Graveyard with Girsu (a DARK Machine) in the Deck to send, and monsters on both sides to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (69811710, DECK),
    (69811710, DECK),
    (69811710, MZONE),
    (3573512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.is_link_monster(duel).into()));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_TOGRAVE);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_condition(duel, |duel, a| !babelcon(duel, a));
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMING_END_PHASE);
    e3.set_condition(duel, |duel, a| babelcon(duel, a) && aux::stat_change_damage_step_condition(duel, a));
    c.register_effect(duel, e3);
}

fn babelcon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_some()
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_DARK) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(gc) = g.get_first() {
        let lv = gc.get_level(duel);
        if duel.send_to_grave(gc, REASON_EFFECT)? != 0
            && gc.is_location(duel, LOCATION_GRAVE)
            && let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
        {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            e1.set_value(duel, (lv * 100) as i32);
            tc.register_effect(duel, e1);
        }
    }
    let e2 = duel.create_effect(e.get_handler(duel));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_attribute(duel, ATTRIBUTE_DARK));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, e.get_handler(duel), 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}
