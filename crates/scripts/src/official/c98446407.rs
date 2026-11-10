// Hand of Nephthys
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 98446407;

pub const LISTED_NAMES: &[u32] = &[61441708];

// On the field, with Sacred Phoenix of Nephthys in the Deck to summon and another monster to release.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (61441708, DECK), (61441708, DECK), (ID, MZONE)];
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

fn mzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.get_sequence(duel) < 5
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if c.get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > -1
            && c.is_releasable(duel)
            && duel.check_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into()))
            && (ft > 0 || duel.check_release_group_cost(tp, |c, duel| mzfilter(c, duel, tp), (1, 1), false, None, Some(c.into()))));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let mut rg = if ft > 0 {
        duel.select_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into()))?
    } else {
        duel.select_release_group_cost(tp, |c, duel| mzfilter(c, duel, tp), (1, 1), false, None, Some(c.into()))?
    };
    rg.insert(c);
    duel.release(rg, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 61441708) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
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
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
