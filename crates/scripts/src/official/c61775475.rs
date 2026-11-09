// Arms of Genex Return Zero
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 61775475;
// s.attr_list[tp], the attributes banished so far this turn, at this key plus tp.
const ATTRIBUTES_USED: u64 = (ID as u64) << 16;

// In the Extra Deck, Synchro Summoned with Genex Controller (DARK Tuner) and Misairuzame and Slime Toad, with Genex Controller in the
// Graveyard to shuffle, and Elfnote Lucina and Sangan (destroyed by Raigeki) on the opponent's side. Banishing a monster of the activating monster's
// attribute is the cost of the negation.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (68505803, MZONE),
    (33178416, MZONE),
    (68638985, MZONE),
    (68505803, GRAVE),
    (13597785, MZONE | OPPONENT),
    (14558127, GRAVE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, negcost);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| aux::add_values_reset(duel, reset));
}

fn reset(duel: &mut Duel) {
    duel.set_script_state(ATTRIBUTES_USED, 0);
    duel.set_script_state(ATTRIBUTES_USED | 1, 0);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn negcostfilter(c: CardId, duel: &Duel, tp: u8, att: u32) -> bool {
    let used = duel.get_script_state(ATTRIBUTES_USED | u64::from(tp)) as u32;
    c.get_attribute(duel) & att & !used > 0 && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let att = a.re.map_or(0, |re| re.get_handler(duel).get_attribute(duel));
    let filter = |c: CardId, duel: &Duel| negcostfilter(c, duel, tp, att);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.first() {
        let key = ATTRIBUTES_USED | u64::from(tp);
        let used = duel.get_script_state(key) | i64::from(tc.get_attribute(duel));
        duel.set_script_state(key, used);
    }
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GENEX]) && c.is_able_to_deck(duel) && c.is_can_be_effect_target(duel, e)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tdfilter(c, duel, e), tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let rescon = aux::dpcheck(|c, duel| i64::from(c.get_attribute(duel)));
    let tg = aux::select_unselect_group(duel, g, 1, 6, &rescon, tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_STZONE as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tg) = duel.get_target_cards(a.e).filter(|tg| !tg.is_empty()) else {
        return Ok(());
    };
    if duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let ct = tg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if ct == 0 {
        return Ok(());
    }
    let g = duel.get_field_group(tp, LOCATION_STZONE, LOCATION_STZONE);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = g.select(duel, tp, 1, ct as u8, None)?;
    if !dg.is_empty() {
        duel.hint_selection(dg);
        duel.break_effect()?;
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
