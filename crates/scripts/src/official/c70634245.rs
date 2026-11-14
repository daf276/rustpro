// Legendary Lord Six Samurai - Enishi
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT, REMOVED};

const ID: u32 = 70634245;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Anarchist Monk of the Six Samurai (a Level 3 Tuner) and Chamberlain of the Six Samurai (Level 3), with more
// Chamberlains in the Graveyard to banish and banished to summon, monsters of the opponent's to return, and Dark Hole to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (80570228, MZONE),
    (80570228, MZONE),
    (44430454, MZONE),
    (44430454, MZONE),
    (44430454, GRAVE),
    (44430454, GRAVE),
    (44430454, REMOVED),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thcostfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let maxtc = duel.get_target_count(CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, thcostfilter, tp, LOCATION_GRAVE, 0, 1, maxtc as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, ct, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED, POS_FACEUP)
}
