// Drytron Meteonis Draconids
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 69815951;

pub const LISTED_NAMES: &[u32] = &[22398665];
// Ritual Summoned by Meteonis Drytron from Drytron monsters, with more in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (22398665, HAND),
    (22398665, HAND),
    (97148796, HAND),
    (96026108, MZONE),
    (60037599, MZONE),
    (97148796, GRAVE),
    (96026108, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(tgval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_condition(duel, atkcon);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_special_summoned(duel))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, gytg);
    e3.set_operation(duel, gyop);
    c.register_effect(duel, e3);
}

fn tgval(duel: &Duel, e: EffectId, re: EffectId, rp: u8) -> bool {
    aux::tgoval(duel, e, re, rp) && re.is_monster_effect(duel)
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    if g.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) <= 2 {
        let reset = RESET_EVENT | RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE);
        c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, aux::stringid(ID, 1));
    }
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_ritual_summoned(duel) && c.get_flag_effect(duel, ID) != 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && c.get_attack(duel) > 0 && aux::sp_elim_filter(c, duel, true, false)
}

fn rescon(ct: usize) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let atk = sg.get_sum(duel, |c, duel| i64::from(c.get_attack(duel)));
        if ct == 1 {
            (atk == 2000, atk > 2000)
        } else {
            (atk == 2000 || atk == 4000, atk > 4000)
        }
    }
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_grave(duel)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    // The Lua calls tgfilter without a card, an error.
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(cfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let targetable = |c: CardId, duel: &Duel| tgfilter(c, duel) && c.is_can_be_effect_target(duel, e);
    let ct = duel.get_matching_group_count(targetable, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(ct > 0 && aux::select_unselect_group_check(duel, g, 1, g.len(), &rescon(ct)));
    }
    let rg = aux::select_unselect_group_full(duel, g, 1, g.len(), &rescon(ct), tp, HINTMSG_REMOVE, Some(&rescon(ct)), false)?;
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    let gyc = (duel.get_operated_group().get_sum(duel, |c, duel| i64::from(c.get_attack(duel))) / 2000) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sg = duel.select_target(tp, tgfilter, tp, 0, LOCATION_ONFIELD, gyc, gyc, None)?;
    let count = sg.map_or(0, |sg| sg.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, sg, count, 0, 0);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = duel.get_target_cards(a.e)
        && !g.is_empty()
    {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
