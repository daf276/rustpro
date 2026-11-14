// Volcanic Emperor
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 46412900;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_BLAZE_ACCELERATOR, SET_VOLCANIC];

// In hand with three Volcanic Rats in the Graveyard to banish, and Volcanic Traps in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (33112041, GRAVE),
    (33112041, GRAVE),
    (33112041, GRAVE),
    (58143766, DECK),
    (84138874, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
    });
    e1.set_target(duel, damsettg);
    e1.set_operation(duel, damsetop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None))
    });
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_monster(duel) && c.is_race(duel, RACE_PYRO)) || c.is_set_card(duel, &[SET_BLAZE_ACCELERATOR]))
        && c.is_faceup(duel)
        && c.is_able_to_remove_as_cost(duel)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        duel.get_mzone_count(tp, Some(sg)) > 0
            && ((sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_BLAZE_ACCELERATOR]), 1, None) && sg.len() == 1)
                || sg.filter_count(duel, |c, duel| c.is_race(duel, RACE_PYRO), None) == 3),
        false,
    )
}

fn cost_group(duel: &Duel, c: CardId, tp: u8) -> CardSet {
    duel.get_matching_group(spcostfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, Some(c.into()))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = cost_group(duel, c, tp);
    !rg.is_empty() && aux::select_unselect_group_check(duel, rg, 1, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let rg = cost_group(duel, c, tp);
    let check: aux::GroupCheck = &|duel, sg, _, _| rescon(duel, sg, tp);
    let g = aux::select_unselect_group_full(duel, rg, 1, 3, check, tp, HINTMSG_REMOVE, Some(check), true)?;
    if !g.is_empty() {
        e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn banished_pyros(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_PYRO)),
        tp,
        LOCATION_REMOVED,
        LOCATION_REMOVED,
        None,
    )
}

fn damsettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = banished_pyros(duel, tp);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 500);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VOLCANIC]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn damsetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = banished_pyros(duel, tp);
    if g.is_empty() {
        return Ok(());
    }
    if duel.damage(1 - tp, g.len() as u32 * 500, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.sset(tp, sg)?;
        }
    }
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    Ok(())
}
