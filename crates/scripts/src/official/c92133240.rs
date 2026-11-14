// Dinomorphia Therizia
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92133240;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];
// Copies in hand, Dinomorphia Domain in the Deck to Set and in the GY to banish, Diplos in the GY to summon once Raigeki destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (26631975, DECK),
    (26631975, DECK),
    (26631975, GRAVE),
    (38628859, GRAVE),
    (38628859, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    register_destroyed_summon::<ID, 4, true>(duel, c);
}

// The Special Summon from the GY every Dinomorphia monster has when it is destroyed; the Main Deck ones banish a Trap for it
// and exclude their own name.
pub fn register_destroyed_summon<const CODE: u32, const LEVEL: u32, const MAIN_DECK: bool>(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(CODE, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (CODE, 1));
    e3.set_condition(duel, |_, a| a.r & (REASON_EFFECT + REASON_BATTLE) != 0);
    if MAIN_DECK {
        e3.set_cost(duel, spcost);
    }
    e3.set_target(duel, sptg::<CODE, LEVEL, MAIN_DECK>);
    e3.set_operation(duel, spop::<CODE, LEVEL, MAIN_DECK>);
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DINOMORPHIA]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let Some(tc) = duel
        .select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.sset(tp, tc)? > 0 && duel.get_lp(tp) <= 2000 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_remove_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spcostfilter, LOCATION_GRAVE)
}

fn spfilter<const CODE: u32, const LEVEL: u32, const MAIN_DECK: bool>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DINOMORPHIA])
        && c.is_level_below(duel, LEVEL)
        && (!MAIN_DECK || !c.is_code(duel, CODE))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg<const CODE: u32, const LEVEL: u32, const MAIN_DECK: bool>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter::<CODE, LEVEL, MAIN_DECK>(c, duel, a.e, a.tp),
        LOCATION_GRAVE,
    )
}

fn spop<const CODE: u32, const LEVEL: u32, const MAIN_DECK: bool>(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter::<CODE, LEVEL, MAIN_DECK>(c, duel, a.e, a.tp),
        LOCATION_GRAVE,
        POS_FACEUP,
    )
}
