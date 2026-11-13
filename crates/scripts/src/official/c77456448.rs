// Spell Shattering Sword
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 77456448;
const CHOICE: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand and Set, with Light and Darkness Ritual in the hand and Graveyard to show, Toon World and Elfnote Lucina on the
// opponent's field to destroy or negate, and the opponent's Mystical Space Typhoon to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, HAND),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, GRAVE),
    (CARD_TOON_WORLD, SZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E | TIMING_DAMAGE_STEP,
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e2.set_target(duel, |duel, a, chk, _| {
        let tp = a.tp;
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_HAND | LOCATION_ONFIELD, 1, None));
        }
        duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
        Ok(false)
    });
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    (!c.is_attack(duel, &[0]) || c.is_negatable_monster(duel)) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let opp_spells = duel.get_matching_group(aux::faceup_filter(CardId::is_spell), tp, 0, LOCATION_ONFIELD, None);
    let option_1 = !opp_spells.is_empty() && !duel.is_damage_step();
    let option_2 = duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None)
        && aux::stat_change_damage_step_condition(duel, a)
        && duel.is_existing_matching_card(
            |c, duel| c.is_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        );
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 2)), (option_2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_DESTROY);
        e.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(opp_spells), opp_spells.len() as u8, tp, 0);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let g = duel.select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            let opp_spells = duel.get_matching_group(aux::faceup_filter(CardId::is_spell), tp, 0, LOCATION_ONFIELD, None);
            if !opp_spells.is_empty() {
                duel.destroy(opp_spells, REASON_EFFECT)?;
            }
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let filter = |c: CardId, duel: &Duel| c.is_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL);
            let Some(sc) = duel
                .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first()
            else {
                return Ok(());
            };
            if sc.is_location(duel, LOCATION_HAND) {
                duel.confirm_cards(1 - tp, sc)?;
                duel.shuffle_hand(tp)?;
            } else {
                duel.hint_selection(sc);
            }
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, e)
                && tc.is_faceup(duel)
            {
                let c = e.get_handler(duel);
                duel.break_effect()?;
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
                e1.set_value(duel, 0);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
                tc.negate_effects(duel, c, 0, false);
            }
        }
        _ => {}
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let hand_group = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_HAND, None);
    let field_group = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    let (option_1, option_2) = (!hand_group.is_empty(), !field_group.is_empty());
    if !(option_1 || option_2) {
        return Ok(());
    }
    let selected = if option_1 && option_2 {
        duel.select_effect(tp, &[(option_1, aux::stringid(ID, 4)), (option_2, aux::stringid(ID, 5))])?
    } else {
        None
    };
    let choice = selected.unwrap_or(if option_1 { 1 } else { 2 });
    let mut desg = CardSet::default();
    if choice == 1 {
        desg = hand_group.random_select(duel, tp, 1);
    } else if choice == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        desg = field_group.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(desg);
    }
    if !desg.is_empty() {
        duel.destroy(desg, REASON_EFFECT)?;
    }
    Ok(())
}
