// Quadborrel Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 29296344;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
// In the Extra Deck, Link Summoned from two Rokket monsters, destroying an opponent's Link Monster (Linguriboh) to Special Summon
// Rokket monsters from the Graveyard, with Sangan in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (25554552, MZONE),
    (68464358, MZONE),
    (24842059, MZONE | OPPONENT),
    (24842059, MZONE | OPPONENT),
    (25554552, GRAVE),
    (68464358, GRAVE),
    (67127799, GRAVE),
    (26202165, HAND),
    (26202165, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(68464358)];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_ROKKET], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ge1 = duel.create_effect(c);
    ge1.set_description(duel, aux::stringid(ID, 2));
    ge1.set_type(duel, EFFECT_TYPE_FIELD);
    ge1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    ge1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    ge1.set_target_range(duel, 1, 0);
    ge1.set_target_card(duel, |duel, _, c| c.is_link_below(duel, 2) && c.is_location(duel, LOCATION_EXTRA));
    ge1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_link_below(duel, 2));
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && tc.is_link_monster(duel)
        && tc.get_link(duel) > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        let mut ft = duel.get_location_count(tp, LOCATION_MZONE) as usize;
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        ft = ft.min(tc.get_link(duel) as usize);
        let g = duel.get_matching_group(&filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, None);
        ft = ft.min(g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))));
        let sg = aux::select_unselect_group(duel, g, 1, ft, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
