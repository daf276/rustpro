// Hieratic Seal of the Heavenly Spheres
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 24361622;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp)
    }
}

// Link Summoned in the Extra Deck with two Lesser Dragon (Level 4 Dragon Normal Monsters), which it can also Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (55444629, MZONE), (55444629, MZONE), (55444629, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, thcon);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_HAND | LOCATION_DECK }>,
    );
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_sequence(duel) > 4 && duel.is_turn_player(1 - a.tp)
}

fn thcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_releasable(duel)
        && duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_able_to_hand),
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(c.into()),
        )
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| thcfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), true, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), true, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(CardId::is_able_to_hand),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else {
        return Ok(());
    };
    if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    Ok(())
}
