// Dyna Mondo
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 73898890;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_RITUAL, Some(lc), sumtype, tp), 1, None)
    }
}

// Copies in the Extra Deck, Relinquished, a Ritual Monster, and Normal Monsters to Link Summon with, Ritual Monsters in the
// Graveyard, and Elfnote Lucina on the opponent's side to return to the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (64631466, MZONE),
    (64631466, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (64631466, GRAVE),
    (30208479, GRAVE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_be_effect_target(duel, e) && c.is_able_to_deck(duel) && (c.is_location(duel, LOCATION_ONFIELD) || c.is_ritual_monster(duel))
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) == 1,
        false,
    )
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let tdg = duel.get_matching_group(
        |c, duel| tgfilter(c, duel, e),
        tp,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        LOCATION_ONFIELD,
        None,
    );
    if chk == 0 {
        return Ok(tdg.len() > 1 && aux::select_unselect_group_check(duel, tdg, 2, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let tg = aux::select_unselect_group(duel, tdg, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 2, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.len() == 2 {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rc: CardId) -> bool {
    c.is_ritual_monster(duel)
        && (c.is_able_to_hand(duel)
            || (duel.get_mzone_count(tp, Some(rc.into())) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thspfilter(chkc, duel, e, tp, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c2, duel| thspfilter(c2, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c2, duel| thspfilter(c2, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        |tc, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 2),
    )
}
