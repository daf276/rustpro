// Dragunity Knight - Romulus
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 11969228;

// Link Summoned from two Dragon or Winged Beast monsters, with Guardragon Pisty to Link Summon into its zone, Dragon Ravine to search and monsters in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13143275, EXTRA),
    (13143275, EXTRA),
    (11091375, MZONE),
    (20315854, MZONE),
    (88819587, MZONE),
    (3170832, MZONE),
    (62265044, DECK),
    (62265044, DECK),
    (88819587, HAND),
    (88819587, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

const DRAGON_RAVINE: u32 = 62265044;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON | RACE_WINGEDBEAST, Some(lc), sumtype, tp)
            && !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[DRAGON_RAVINE];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_spell_trap(duel)) || (c.is_code(duel, DRAGON_RAVINE) && c.is_able_to_hand(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| c.is_race(duel, RACE_DRAGON) && c.is_previous_location(duel, LOCATION_EXTRA) && lg.is_contains(c),
        1,
        None,
    )
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON | RACE_WINGEDBEAST)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        let hc = e.get_handler(duel);
        let e1 = duel.create_effect(hc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(hc);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect_full(duel, e2, true);
        let e3 = duel.create_effect(hc);
        e3.set_description(duel, 3312);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        e3.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_value(duel, 1);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect_full(duel, e3, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
