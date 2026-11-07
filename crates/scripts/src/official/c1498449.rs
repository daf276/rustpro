// Dracotail Faimena
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 1498449;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON | RACE_SPELLCASTER)
    }
}

struct TargetParams;

impl FusionSummonParams for TargetParams {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        Params::fusfilter(c, duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, Some((fcheck, e.get_handler(duel))))
    }
}

// Two copies in hand, one to discard and one as material with a Dracotail monster for Dracotail Arthalion in the Extra Deck, and a
// Dracotail Spell/Trap in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (33760966, EXTRA),
    (75003700, MZONE),
    (5431722, DECK),
    (5431722, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, fustg);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, matcon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn fcheck(_: &Duel, exc: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    !sg.contains(exc)
}

fn fustg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Fusion::new_summon_eff_closure(duel, a.e, false);
    if chk == 0 {
        return Fusion::summon_eff_tg::<TargetParams>(duel, a, 0, None);
    }
    Fusion::summon_eff_tg::<Params>(duel, a, 1, None)?;
    Ok(false)
}

// The condition of the Dracotail monsters' effect when sent to the Graveyard as Fusion Material.
pub fn matcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r & REASON_FUSION == REASON_FUSION
}

pub fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRACOTAIL]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

pub fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

pub fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
