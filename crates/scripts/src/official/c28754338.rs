// Legendary Atlantean Tridon
use super::c47826112::weaken_opponent_monsters;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 28754338;

pub const LISTED_NAMES: &[u32] = &[47826112];
// On the field with Atlantean Pikeman, a Sea Serpent Normal Monster, to Tribute and Poseidra, the Atlantean Dragon in the hand
// and the Deck, facing a monster to weaken.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26976414, MZONE),
    (47826112, HAND),
    (47826112, DECK),
    (15025844, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, ft: i32, tp: u8) -> bool {
    c.is_race(duel, RACE_SEASERPENT)
        && (ft > 0 || (c.get_sequence(duel) < 5 && c.is_controler(duel, tp)))
        && (c.is_faceup(duel) || c.is_controler(duel, tp))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 1);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let c = e.get_handler(duel);
    if c.get_sequence(duel) < 5 {
        ft += 1;
    }
    let filter = |sc, duel: &Duel| cfilter(sc, duel, ft, tp);
    if chk == 0 {
        return Ok(ft > -1 && c.is_releasable(duel) && duel.check_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let mut rg = duel.select_release_group_cost(tp, filter, (1, 1), false, None, Some(c.into()))?;
    rg.insert(c);
    duel.release(rg, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 47826112) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
        if e.get_label(duel) == 0 && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.break_effect()?;
        weaken_opponent_monsters(duel, e.get_handler(duel), tp, 300);
    }
    Ok(())
}
