// Odd-Eyes Pendulum Dragon of the Four Heavenly Dragons
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 75787708;

// Copies in the Pendulum Zone and on the field with Dark Hole in the opponent's hand, and Pendulum Monsters in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (55226153, DECK),
    (55226153, DECK),
    (17390179, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY | CATEGORY_TOHAND | CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_pendulum_summoned(duel));
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_pendulum_summoned(duel));
    e3.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOHAND | CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, thcon2);
    e4.set_target(duel, thtg2);
    e4.set_operation(duel, thop2);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_pendulum_monster(duel) && c.is_attack_below(duel, 1500) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(actlimit));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_label_object(duel, Some(e1));
    e0.set_operation(duel, checkop);
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e0, tp);
    Ok(())
}

fn actlimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_location(duel, LOCATION_MZONE) && re.is_monster_effect(duel)
}

fn psfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_pendulum_summoned(duel)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(ef) = e.get_label_object(duel)
        && a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| psfilter(c, duel, tp), 1, None))
    {
        ef.reset(duel);
        e.reset(duel);
    }
    Ok(())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let pg = duel.get_field_group(e.get_handler_player(duel), LOCATION_PZONE, 0);
    if pg.len() != 2 {
        return 0;
    }
    let mut cards = pg.iter();
    let (Some(pc1), Some(pc2)) = (cards.next(), cards.next()) else {
        return 0;
    };
    (pc1.get_scale(duel).abs_diff(pc2.get_scale(duel)) * 300) as i32
}

fn thcon2(duel: &Duel, a: &Args) -> bool {
    a.r & (REASON_BATTLE | REASON_EFFECT) > 0 && a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    let label = i64::from(a.rp == 1 - tp);
    e.set_label(duel, label);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    if label == 1 {
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    }
    Ok(false)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? != 0
        && e.get_label(duel) == 1
        && c.is_location(duel, LOCATION_HAND)
    {
        duel.shuffle_hand(tp)?;
        if duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.break_effect()?;
            duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
