// Duel Link Dragon, the Duel Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 60025883;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_TOOL];

const DUEL_DRAGON_TOKEN: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[DUEL_DRAGON_TOKEN];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_SYNCHRO, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned with a Synchro Monster, or on the field, with Level 7 and 8 Dragon Synchro Monsters in the Extra Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (67556500, MZONE),
    (24154052, MZONE),
    (9012916, EXTRA),
    (3294539, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(DUEL_DRAGON_TOKEN),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_label(duel, 0);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, tkcost);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, tgcon);
    e2.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
}

fn token_stats(tc: CardId, duel: &Duel) -> MonsterStats {
    MonsterStats::new(
        0,
        TYPES_TOKEN,
        tc.get_attack(duel),
        tc.get_defense(duel),
        tc.get_level(duel),
        tc.get_race(duel),
        tc.get_attribute(duel),
    )
}

fn tkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, 100);
    Ok(chk == 0)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    (c.is_set_card(duel, &[SET_POWER_TOOL]) || ((c.get_level(duel) == 7 || c.get_level(duel) == 8) && c.is_race(duel, RACE_DRAGON)))
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.get_level(duel) > 0
        && c.is_able_to_remove_as_cost(duel)
        && duel.check_mut(|duel| duel.is_player_can_special_summon_monster(tp, DUEL_DRAGON_TOKEN, token_stats(c, duel)))
        && duel.get_mzone_count_full(tp, None, tp, LOCATION_REASON_TOFIELD, zone) > 0
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_free_linked_zone(duel) & ZONES_MMZ;
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(zone > 0 && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp, zone), tp, LOCATION_EXTRA, 0, 1, None));
    }
    e.set_label(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp, zone), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label_object_group(duel, Some(g));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_free_linked_zone(duel) & ZONES_MMZ;
    if zone == 0 {
        return Ok(());
    }
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if duel.is_player_can_special_summon_monster(tp, DUEL_DRAGON_TOKEN, token_stats(tc, duel))
        && duel.get_mzone_count_full(tp, None, tp, LOCATION_REASON_TOFIELD, zone) > 0
        && let Some(token) = duel.create_token(tp, DUEL_DRAGON_TOKEN)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_value(duel, tc.get_attack(duel));
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        token.register_effect(duel, e1);
        for (code, value) in [
            (EFFECT_SET_BASE_DEFENSE, tc.get_defense(duel)),
            (EFFECT_CHANGE_LEVEL, tc.get_level(duel) as i32),
            (EFFECT_CHANGE_RACE, tc.get_race(duel) as i32),
            (EFFECT_CHANGE_ATTRIBUTE, tc.get_attribute(duel) as i32),
        ] {
            let ex = e1.clone_effect(duel);
            ex.set_code(duel, code);
            ex.set_value(duel, value);
            token.register_effect(duel, ex);
        }
        duel.special_summon_full(token, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, DUEL_DRAGON_TOKEN)),
        a.e.get_handler_player(duel),
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}
