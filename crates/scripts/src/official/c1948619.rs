// Xtra HERO Wonder Driver
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 1948619;

pub const LISTED_SERIES: &[u16] = &[SET_HERO, SET_FUSION, SET_CHANGE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_HERO], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck for its Link Summon, then on the field with HERO monsters to Summon and Polymerization and Mask
// Change in the GY to Set, and the opponent's Raigeki and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (40044918, MZONE),
    (40044918, MZONE),
    (40044918, HAND),
    (10808715, HAND),
    (CARD_POLYMERIZATION, GRAVE),
    (21143940, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, setcon);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn setcfilter(c: CardId, duel: &Duel, tp: u8, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_set_card(duel, &[SET_HERO]) && lg.is_contains(c)
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let (tp, lg) = (a.tp, a.e.get_handler(duel).get_linked_group(duel));
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| setcfilter(c, duel, tp, lg), 1, None))
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_spell(duel) && c.is_set_card(duel, &[SET_FUSION])) || (c.is_type(duel, TYPE_QUICKPLAY) && c.is_set_card(duel, &[SET_CHANGE])))
        && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_ssetable(duel)
    {
        duel.sset(a.tp, tc)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - a.tp && c.is_reason(duel, REASON_DESTROY) && c.is_previous_controler(duel, a.tp))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_HERO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
