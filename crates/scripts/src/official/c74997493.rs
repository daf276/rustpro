// Saryuja Skull Dread
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 74997493;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property(duel, sg, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
    }
}

// Copies in the Extra Deck, to be Link Summoned from four different Level 4 or lower Cyberse Normal Monsters, with more in
// the hand and Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
    (24154052, HAND),
    (32295838, DECK),
    (36211150, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let count = c.get_material_count(duel);
    let hint = |duel: &mut Duel, n: u64| {
        c.register_flag_effect_full(
            duel,
            0,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, n),
        );
    };
    if count >= 2 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
        e1.set_property(duel, EFFECT_FLAG_DELAY);
        e1.set_code(duel, EVENT_SUMMON_SUCCESS);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_condition(duel, |duel, a| {
            aux::zptgroupcon(duel, a.eg.unwrap_or_default(), CardId::is_faceup, a.e.get_handler(duel))
        });
        e1.set_operation(duel, atkop);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        c.register_effect(duel, e2);
        if count == 2 {
            hint(duel, 3);
        }
    }
    if count >= 3 {
        let e3 = duel.create_effect(c);
        e3.set_description(duel, aux::stringid(ID, 1));
        e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e3.set_type(duel, EFFECT_TYPE_IGNITION);
        e3.set_range(duel, LOCATION_MZONE);
        e3.set_count_limit(duel, 1, 0);
        e3.set_target(duel, sptg);
        e3.set_operation(duel, spop);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e3);
        hint(duel, if count == 3 { 4 } else { 5 });
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = aux::zptgroup(duel, a.eg.unwrap_or_default(), CardId::is_faceup, c);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 300);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_link_summoned(duel) && c.get_material_count(duel) == 4
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 4));
    }
    duel.set_target_player(tp);
    duel.set_target_param(4);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 4);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 3, tp, 0);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == 4 {
        let g = duel.get_matching_group(CardId::is_able_to_deck, p, LOCATION_HAND, 0, None);
        if g.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, p, HINTMSG_TODECK);
        let sg = g.select(duel, p, 3, 3, None)?;
        duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
        duel.sort_deckbottom(p, p, 3)?;
    }
    Ok(())
}
