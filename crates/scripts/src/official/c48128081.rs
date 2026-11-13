// Destined Duel
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 48128081;

// In the hand with Normal Monsters, and the opponent's face-down Spell (Lightning Storm) and monsters to attack and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (99171160, HAND),
    (99171160, HAND),
    (86498013, HAND | OPPONENT),
    (86498013, MZONE),
    (14532163, SZONE | OPPONENT),
    (14532163, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, a.tp)));
    e2.set_target(duel, |duel, a, chk, _| {
        let tp = a.tp;
        let g = duel.get_matching_group(CardId::is_facedown, tp, 0, LOCATION_ONFIELD, None);
        if chk == 0 {
            return Ok(!g.is_empty());
        }
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        let tp = a.tp;
        if duel.negate_attack()? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, CardId::is_facedown, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_END);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        let c = a.e.get_handler(duel);
        duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
        Ok(false)
    });
    e3.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        }
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_EITHER, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_ALL, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, sp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, sp, false, false, POS_FACEUP_ATTACK, sp, 0xff)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut sg = CardSet::default();
    let turn_player = duel.get_turn_player();
    for sp in [turn_player, 1 - turn_player] {
        if duel.get_location_count_full(sp, LOCATION_MZONE, sp, LOCATION_REASON_TOFIELD, 0xff) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, sp), sp, LOCATION_HAND, 0, 1, None)
            && duel.select_yes_no(sp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, sp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(sp, |c, duel| spfilter(c, duel, e, sp), sp, LOCATION_HAND, 0, 1, 1, None)?;
            if let Some(sc) = g.get_first() {
                sg.insert(sc);
                duel.special_summon_step(sc, 0, sp, sp, false, false, POS_FACEUP_ATTACK)?;
            }
        }
    }
    if duel.special_summon_complete()? > 0
        && duel.is_existing_matching_card(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(sg))
    {
        let posg = duel.get_matching_group(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, Some(sg));
        duel.break_effect()?;
        duel.change_position(posg, POS_FACEDOWN_DEFENSE)?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1a = duel.create_effect(e.get_handler(duel));
    e1a.set_description(duel, aux::stringid(ID, 4));
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1a.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1a.set_target_range(duel, 1, 0);
    e1a.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1a, tp);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    duel.register_effect(e1b, tp);
    Ok(())
}
