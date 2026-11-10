// Astrograph Sorcerer
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 76794549;

const DRAGON_SETCODES: [u16; 4] = [SET_PENDULUM_DRAGON, SET_XYZ_DRAGON, SET_SYNCHRO_DRAGON, SET_FUSION_DRAGON];

pub const LISTED_NAMES: &[u32] = &[94415058, CARD_ZARC];
// Copies in the Pendulum Zone, on the field, in the hand and in the Deck, Stargazer Magician in the Deck to place, and the
// opponent's Raigeki to destroy. No ported card is a Pendulum Dragon, so the four-Dragon cost of the Z-ARC effect is reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, DECK),
    (94415058, DECK),
    (94415058, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 5))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rptg);
    e1.set_operation(duel, rpop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 3));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default().is_exists(duel, |c, duel| spcfilter(c, duel, tp), 1, None)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 5));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, hncost);
    e3.set_target(duel, hntg);
    e3.set_operation(duel, hnop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROYED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) {
            tc.register_flag_effect(
                duel,
                ID,
                RESET_EVENT
                    | RESET_TURN_SET
                    | RESET_TEMP_REMOVE
                    | RESET_TOHAND
                    | RESET_TODECK
                    | RESET_LEAVE
                    | RESET_TOFIELD
                    | RESET_PHASE
                    | PHASE_END,
                0,
                1,
            );
        } else if tc.is_location(duel, LOCATION_EXTRA) {
            tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn rpfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 94415058)
        && (!c.is_forbidden(duel)
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn rptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rpfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(e.get_handler(duel).into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn rpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.destroy(c, REASON_EFFECT)? <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 6));
    let g = duel.select_matching_card(
        tp,
        |c, duel| rpfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else {
        return Ok(());
    };
    let op = if duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    };
    if op == 0 {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    } else {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && c.is_previous_controler(duel, tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.get_flag_effect(duel, ID) != 0
        && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && duel.is_existing_matching_card(|tc, duel| thfilter2(tc, duel, c.get_code(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter2(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let zones = LOCATION_GRAVE | LOCATION_REMOVED | LOCATION_EXTRA;
    let g = duel.get_matching_group(|c, duel| thfilter1(c, duel, tp), tp, zones, zones, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 4))? {
        return Ok(());
    }
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 7));
    let cg = g.select(duel, tp, 1, 1, None)?;
    duel.hint_selection(cg);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let code = cg.get_first().map_or(0, |tc| tc.get_code(duel));
    let sg = duel.select_matching_card(tp, |c, duel| thfilter2(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, sg)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &DRAGON_SETCODES)
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && (!c.is_location(duel, LOCATION_MZONE) || c.is_faceup(duel))
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, true))
}

fn rescon(duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> (bool, bool) {
    let setcodes = DRAGON_SETCODES.map(u32::from);
    let checkfunc = aux::property_table_filter(|c, duel| c.get_set_card(duel).into_iter().map(u32::from).collect(), &setcodes);
    if !aux::check_different_property(duel, sg, &checkfunc) {
        return (false, true);
    }
    (
        duel.is_existing_matching_card(|c, duel| hnfilter(c, duel, e, tp, Some(sg)), tp, LOCATION_EXTRA, 0, 1, None),
        false,
    )
}

fn hnfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: Option<CardSet>) -> bool {
    c.is_code(duel, CARD_ZARC)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
        && duel.get_location_count_from_ex(tp, tp, sg.map(|sg| sg.union(e.get_handler(duel).into())), c) > 0
}

fn hncost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mg = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE, 0, Some(c.into()));
    let check = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| rescon(duel, e, tp, sg);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && aux::select_unselect_group_check(duel, mg, 4, 4, &check));
    }
    let sg = aux::select_unselect_group_full(duel, mg, 4, 4, &check, tp, HINTMSG_REMOVE, Some(&check), false)?;
    duel.remove(sg.union(c.into()), POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn hntg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn hnop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| hnfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(g, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}
