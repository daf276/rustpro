// Dogmatika Fleurdelis, the Thunderbolt
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 73355772;

// In the hand with Despian Quaeritis Fusion Summoned from the Extra Deck by Polymerization with Aluber and Mystical Elf, and
// Dogmatika Punishment in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (72272462, EXTRA),
    (24094653, HAND),
    (62962630, HAND),
    (15025844, HAND),
    (82956214, DECK),
    (82956214, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_HAND));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase()
        && duel.is_existing_matching_card(
            |c, duel| c.is_summon_location(duel, LOCATION_EXTRA),
            a.tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOGMATIKA]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn actcondfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EXTRA) && c.is_attack_above(duel, 2500) && c.is_faceup(duel)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.sset(tp, sc)? > 0
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_label(duel, i64::from(tp));
        e1.set_condition(duel, |duel, a| {
            duel.is_existing_matching_card(actcondfilter, a.e.get_label(duel) as u8, 0, LOCATION_MZONE, 1, None)
        });
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}
