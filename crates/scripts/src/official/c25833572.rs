// Gate Guardian
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 25833572;

pub const LISTED_NAMES: &[u32] = &[CARD_SANGA_OF_THE_THUNDER, CARD_KAZEJIN, CARD_SUIJIN];

// In the hand, with Sanga, Kazejin and Suijin on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_SANGA_OF_THE_THUNDER, MZONE),
    (CARD_KAZEJIN, MZONE),
    (CARD_SUIJIN, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

const CODES: &[u32] = &[CARD_SANGA_OF_THE_THUNDER, CARD_KAZEJIN, CARD_SUIJIN];

pub fn chk(c: CardId, duel: &Duel, sg: CardSet, g: CardSet, codes: &[u32]) -> bool {
    let Some((&code, rest)) = codes.split_first() else { return false };
    if !c.is_code(duel, code) {
        return false;
    }
    let mut taken = g;
    taken.insert(c);
    rest.is_empty() || sg.is_exists(duel, |tc, duel| chk(tc, duel, sg, taken, rest), 1, Some(taken))
}

pub fn rescon(duel: &Duel, sg: CardSet, tp: u8, codes: &[u32]) -> (bool, bool) {
    (
        duel.get_mzone_count(tp, Some(sg)) >= 1 && sg.is_exists(duel, |c, duel| chk(c, duel, sg, CardSet::default(), codes), 1, None),
        false,
    )
}

// The cards among the release group that have one of the codes, and whether each code has at least one.
pub fn by_codes(duel: &Duel, rg: CardSet, codes: &[u32]) -> (CardSet, bool) {
    let mut g = CardSet::default();
    let mut all = true;
    for &code in codes {
        let found = rg.filter(duel, |c, duel| c.is_code(duel, code), None);
        all &= !found.is_empty();
        g.merge(found);
    }
    (g, all)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let (g, all) = by_codes(duel, duel.get_release_group(tp), CODES);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && all
        && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp, CODES))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (g, _) = by_codes(duel, duel.get_release_group(tp), CODES);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp, CODES);
    let g = aux::select_unselect_group_full(duel, g, 3, 3, &rescon, tp, HINTMSG_RELEASE, Some(&rescon), true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST)?;
    Ok(())
}
