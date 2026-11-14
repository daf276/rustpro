// Amorphage Lechery
use crate::prelude::*;
use crate::testing::{Check, HAND, PZONE};
use engine::constants::MSG_CARD_SELECTED;

const ID: u32 = 70917315;

pub const LISTED_SERIES: &[u16] = &[SET_AMORPHAGE];
// In the Pendulum Zone and in the hand, for its Standby Phase maintenance cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, HAND)];
// The maintenance cost is a continuous effect: its selection hint on the card is counted.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_CARD_SELECTED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_FLIP);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 1);
    e3.set_target_spsummon(duel, |duel, e, c, _, _, _, _, _| {
        let handler = e.get_handler(duel);
        c.is_location(duel, LOCATION_EXTRA)
            && !c.is_set_card(duel, &[SET_AMORPHAGE])
            && (handler.is_pendulum_summoned(duel) || handler.get_flag_effect(duel, ID) != 0)
    });
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_range(duel, LOCATION_PZONE);
    e4.set_target_range(duel, 1, 1);
    e4.set_condition(duel, |duel, a| {
        let amorphage = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_AMORPHAGE]));
        duel.is_existing_matching_card(amorphage, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
    });
    e4.set_value(duel, Value::EffectPlayer(limval));
    c.register_effect(duel, e4);
    let e5 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| {
        !c.is_original_set_card(duel, &[SET_AMORPHAGE])
    });
    e5.set_target_range(duel, 0xff, 0xff);
    c.register_effect(duel, e5);
}

fn limval(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_spell_effect(duel) && !rc.is_set_card(duel, &[SET_AMORPHAGE]) && !rc.is_immune_to_effect(duel, e)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint_selection_full(c, true);
    let options = ReleaseOptions {
        exception: Some(c),
        ..ReleaseOptions::new(tp)
    };
    if duel.check_release_group_full(tp, CardId::is_releasable_by_effect, 1, options) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        let g = duel.select_release_group_full(tp, CardId::is_releasable_by_effect, (1, 1), false, options)?;
        duel.release(g, REASON_COST)?;
    } else {
        duel.destroy(c, REASON_COST)?;
    }
    Ok(())
}
