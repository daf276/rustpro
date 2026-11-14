// Vidolium the Unstable Power Patron of Unity
use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 70488851;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON];

const TYPES_FSX: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ;

// Special Summoned by returning a Fusion, a Synchro and an Xyz Monster: Albion, Chaos Angel, Gauntlet Launcher.
pub const SUPPORT: &[(u32, u16)] = &[(87746184, 0), (22850702, 0), (15561463, 0)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 3))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure(duel, c);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA | LOCATION_GRAVE);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_DECK);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_value(
        duel,
        Value::Effect(|duel, _, te| te.is_monster_effect(duel) && !te.is_card_setcode(duel, SET_POWER_PATRON)),
    );
    c.register_effect(duel, e4);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPES_FSX) && c.is_able_to_extra_as_cost(duel)
}

fn tdrescon(duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> (bool, bool) {
    let handler = e.get_handler(duel);
    let types = sg.iter().fold(0, |types, c| types | c.get_type(duel));
    let res = types & TYPES_FSX == TYPES_FSX
        && ((handler.is_location(duel, LOCATION_GRAVE) && duel.get_mzone_count(tp, Some(sg)) > 0)
            || duel.get_location_count_from_ex(tp, tp, Some(sg), handler) > 0);
    (res, false)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let e = a.e;
    let tp = e.get_handler_player(duel);
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if g.len() >= 3 && lacks_type(duel, g) {
        return false;
    }
    aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| tdrescon(duel, e, tp, sg))
}

// Not in the Lua: tdrescon passes no three cards without all three types, and the search would only read types that run nothing.
fn lacks_type(duel: &Duel, g: CardSet) -> bool {
    let codes = [EFFECT_ADD_TYPE, EFFECT_REMOVE_TYPE, EFFECT_CHANGE_TYPE];
    g.iter().all(|c| codes.iter().all(|&code| !duel.lookup_may_run(c, code)))
        && g.iter().fold(0, |types, c| types | c.get_type(duel)) & TYPES_FSX != TYPES_FSX
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let rescon = |duel: &Duel, sg, _, _| tdrescon(duel, e, tp, sg);
    let rg = aux::select_unselect_group_full(duel, g, 3, 3, &rescon, tp, HINTMSG_TODECK, None, true)?;
    if rg.len() != 3 {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(rg));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    duel.hint_selection(g);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let b1 = duel.get_mzone_count(tp, None) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (true, aux::stringid(ID, 5))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    let c = e.get_handler(duel);
    if op == Some(1) {
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DESTROY);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let op = e.get_label(duel);
    if op == 1 {
        if c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? > 0 && duel.get_mzone_count(tp, None) > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    } else if op == 2 {
        if c.is_relate_to_effect(duel, e) {
            duel.destroy(c, REASON_EFFECT)?;
        }
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 6)));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_original_level(duel, 12)
        });
        e1.set_value(duel, 5000);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove_full(duel, None, POS_FACEDOWN, REASON_EFFECT)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(rmfilter, a.tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(rmfilter, a.tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    if !g.is_empty() {
        duel.remove(g, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
