// Domain of the True Monarchs
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 84171830;

// In the hand to activate, with Dark Armed Dragon and Kashtira Ogre (2800 ATK, 1000 DEF) in the hand to reveal.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (65192027, HAND), (65192027, HAND), (94392192, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, discon);
    e2.set_target_card(duel, splimit);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_LEAVE_FIELD_P);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| {
            eg.is_exists(
                duel,
                |c, duel| c.is_tribute_summoned(duel) && c.is_reason(duel, REASON_FUSION),
                1,
                None,
            )
        })
    });
    e3.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).register_flag_effect(duel, ID, 0, 0, 0);
        Ok(())
    });
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_UPDATE_ATTACK);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_condition(duel, atkcon);
    e4.set_target_card(duel, |duel, _, c| duel.get_attacker() == Some(c) && c.is_tribute_summoned(duel));
    e4.set_value(duel, 800);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_target(duel, lvtg);
    e5.set_operation(duel, lvop);
    c.register_effect(duel, e5);
    let e6 = aux::create_continuous_lizard_check(duel, c, LOCATION_FZONE, |_, _, _| true);
    e6.set_target_range(duel, 0xff, 0xff);
    e6.set_condition(duel, discon);
    c.register_effect(duel, e6);
}

fn splimit(duel: &Duel, e: EffectId, c: CardId) -> bool {
    // The Lua resets the flag inside this target, which a target cannot do here.
    unported!(duel, e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    c.is_location(duel, LOCATION_EXTRA)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_field_group_count(tp, LOCATION_EXTRA, 0) == 0
        && duel.is_existing_matching_card(CardId::is_tribute_summoned, tp, LOCATION_MZONE, 0, 1, None)
        && !duel.is_existing_matching_card(CardId::is_tribute_summoned, tp, 0, LOCATION_MZONE, 1, None)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.is_phase(PHASE_DAMAGE_CAL) && duel.get_attack_target().is_some_and(|d| d.is_controler(duel, 1 - tp))
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_attack(duel) == 2800 && c.get_defense(duel) == 1000
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -2);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
