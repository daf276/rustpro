// Fydraulis Harmonia
use crate::prelude::*;

const ID: u32 = 70088809;
// The fields of its chain data table.
const REVEAL_COUNT: u8 = 0;
const REVEALED_SYNCHROS: u8 = 1;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, effcon);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && chain::is_triggering_location(duel, a.ev as u8, LOCATION_MZONE)
}

fn revealfilter(c: CardId, duel: &Duel) -> bool {
    c.is_synchro_monster(duel) && !c.is_public(duel)
}

fn rescon(duel: &Duel, tp: u8, sg: CardSet) -> bool {
    let able = |sg: CardSet| sg.is_exists(duel, |c, duel| c.is_able_to_grave(duel), 1, None);
    let b1 = sg.len() == 1;
    let b2 = sg.len() == 3 && able(sg);
    let b3 = sg.len() == 5 && able(sg) && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
    b1 || b2 || b3
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(revealfilter, tp, LOCATION_EXTRA, 0, None);
    if chk == 0 {
        return Ok(!c.is_public(duel) && !g.is_empty());
    }
    let max_reveal_count = g.len().min(5);
    let check = |duel: &Duel, sg, _, _| (rescon(duel, tp, sg), false);
    let sg = aux::select_unselect_group_full(duel, g, 1, max_reveal_count, &check, tp, HINTMSG_CONFIRM, Some(&check), false)?;
    duel.confirm_cards(1 - tp, c)?;
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_extra(tp)?;
    for sc in sg {
        sc.create_effect_relation(duel, e);
    }
    e.set_chain_data(duel, REVEAL_COUNT, sg.len() as i64 + 1);
    e.set_chain_data_group(duel, REVEALED_SYNCHROS, sg);
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let reveal_count = e.get_chain_data(duel, REVEAL_COUNT).unwrap_or(0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    if reveal_count >= 4 {
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_EXTRA as i32);
    }
    if reveal_count == 6 {
        let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let reveal_count = e.get_chain_data(duel, REVEAL_COUNT).unwrap_or(0);
    let mut break_chk = false;
    if reveal_count >= 2 && c.is_relate_to_effect(duel, e) {
        break_chk = true;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    if reveal_count >= 4 {
        let revealed = e.get_chain_data_group(duel, REVEALED_SYNCHROS).unwrap_or_default();
        let revealed = revealed.filter(duel, |sc, duel| sc.is_relate_to_effect(duel, e), None);
        e.set_chain_data_group(duel, REVEALED_SYNCHROS, revealed);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = revealed.filter_select(duel, tp, |sc, duel| sc.is_able_to_grave(duel), 1, 1, None)?;
        if !g.is_empty() {
            if break_chk {
                duel.break_effect()?;
            }
            break_chk = true;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    if reveal_count == 6 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            if break_chk {
                duel.break_effect()?;
            }
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    let reset_count = if duel.is_turn_player(tp) { 2 } else { 1 };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, reset_count);
    duel.register_effect(e1, tp);
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && rc.is_on_field(duel) && rc.is_summon_location(duel, LOCATION_EXTRA) && !rc.is_synchro_monster(duel)
}
