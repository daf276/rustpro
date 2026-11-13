// Black Skull Dragon the Archfiend Dragon of Unity
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 97818130;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand and on the field, with Light and Darkness Ritual (a Ritual Spell that mentions itself) to discard and to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, HAND),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, HAND),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, DECK),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(
            duel,
            a,
            chk,
            |c, duel| c.is_quick_play_spell(duel) || c.is_ritual_spell(duel),
            false,
            (1, 1),
        )
    });
    e1.set_target(duel, |duel, a, chk, _| {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        if chk == 0 {
            return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
                && c.is_can_be_special_summoned_full(duel, e, 0, tp, true, false, POS_FACEUP_DEFENSE, tp, 0xff));
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
        Ok(false)
    });
    e1.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, a.tp, a.tp, true, false, POS_FACEUP_DEFENSE)? > 0 {
            c.complete_procedure(duel);
        }
        Ok(())
    });
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_operation(duel, |duel, a| {
        let opp = 1 - a.tp;
        duel.set_lp(opp, duel.get_lp(opp) - 800);
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
            c.update_attack(duel, 800, None, None, None);
        }
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let location = LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, a.tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SET, None, 1, a.tp, location as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(setfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
