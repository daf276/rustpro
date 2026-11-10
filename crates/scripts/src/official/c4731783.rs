// A Bao A Qu, the Lightless Shadow
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 4731783;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp), 1, None)
    }
}

// Feral Imp and Ryu-Kishin, Fiend Normal Monsters, to Link Summon it from the Extra Deck; Fiends, a Dragon and a Spellcaster
// in the Graveyard for the Special Summon and the draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (41392891, MZONE),
    (15303296, MZONE),
    (41392891, HAND),
    (15303296, HAND),
    (15303296, GRAVE),
    (88819587, GRAVE),
    (34460851, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let b1 = duel.get_field_group_count(tp, LOCATION_ONFIELD, LOCATION_ONFIELD) > 0;
    let b2 = c.is_able_to_remove(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_DESTROY);
        let g = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.destroy(g, REASON_EFFECT)?;
        }
    } else if op == 2 {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e)
            && aux::remove_until::<aux::DefaultFieldReturnOp>(duel, c, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?.is_some()
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
                tp,
                LOCATION_GRAVE,
                0,
                1,
                1,
                None,
            )?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn race_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(CardId::is_monster, tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| c.get_race(duel) as i64)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = race_count(duel, tp);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, ct as u8, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = race_count(duel, tp);
    let ct = duel.draw(p, d as u32, REASON_EFFECT)?;
    if ct == 0 {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, ct as u8, ct as u8, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.break_effect()?;
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0 {
        duel.sort_deckbottom(tp, tp, g.len() as u16)?;
    }
    Ok(())
}
