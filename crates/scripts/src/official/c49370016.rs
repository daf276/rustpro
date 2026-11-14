// P.U.N.K. JAM Extreme Session
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 49370016;

pub const LISTED_SERIES: &[u16] = &[SET_PUNK];

// In the hand to activate, with Ze Amin in the Graveyard to banish, in the hand to summon and on the field to pay Life Points
// searching Foxy Tune, and Wa Gon on the field to pay them too.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (19535693, GRAVE),
    (19535693, GRAVE),
    (19535693, HAND),
    (19535693, MZONE),
    (19535693, MZONE),
    (50642380, MZONE),
    (55920742, DECK),
    (55920742, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_PAY_LPCOST);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 2, ID);
    e3.set_condition(duel, drcon);
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spcostfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
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
    a.re.is_some_and(|re| {
        let rc = re.get_handler(duel);
        re.is_monster_effect(duel)
            && rc.is_controler(duel, a.tp)
            && rc.is_location(duel, LOCATION_MZONE)
            && rc.is_race(duel, RACE_PSYCHIC)
            && rc.is_faceup(duel)
    })
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::draw(duel, a)
}
