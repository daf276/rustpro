// Enlilgirsu, the Orcust Mekk-Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 74820316;

pub const LISTED_SERIES: &[u16] = &[SET_ORCUST, SET_WORLD_LEGACY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

// In the Extra Deck for its Link Summon from Galatea and Girsu on the field, banished Girsu to add, and an opponent's monster to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (30741503, MZONE),
    (69811710, MZONE),
    (69811710, MZONE),
    (69811710, REMOVED),
    (69811710, REMOVED),
    (69811710, HAND),
    (3573512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_TODECK + CATEGORY_CONTROL);
    e1a.set_type(duel, EFFECT_TYPE_IGNITION);
    e1a.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, |duel, a| !babelcon(duel, a));
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1b.set_code(duel, EVENT_FREE_CHAIN);
    e1b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1b.set_condition(duel, babelcon);
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOGRAVE);
    e2a.set_type(duel, EFFECT_TYPE_IGNITION);
    e2a.set_range(duel, LOCATION_GRAVE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, |duel, a| !babelcon(duel, a) && tgcond(duel, a));
    e2a.set_cost(duel, Cost::self_banish);
    e2a.set_target(duel, tgtg);
    e2a.set_operation(duel, tgop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2b.set_code(duel, EVENT_FREE_CHAIN);
    e2b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2b.set_condition(duel, |duel, a| babelcon(duel, a) && tgcond(duel, a));
    c.register_effect(duel, e2b);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_ORCUST], Some(lc), sumtype, tp) && c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
}

fn babelcon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_some()
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ORCUST, SET_WORLD_LEGACY]) && c.is_able_to_hand(duel) && c.is_faceup(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_CONTROL, None, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
        && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, None)
        && duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_controler_can_be_changed),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let sc = duel
            .select_matching_card(
                tp,
                aux::faceup_filter(CardId::is_controler_can_be_changed),
                tp,
                0,
                LOCATION_MZONE,
                1,
                1,
                None,
            )?
            .get_first();
        if let Some(sc) = sc {
            duel.hint_selection(sc);
            duel.get_control(sc, tp)?;
        }
    }
    Ok(())
}

fn tgcond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_EMZONE) && c.get_turn_id(duel) == duel.get_turn_count()
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
