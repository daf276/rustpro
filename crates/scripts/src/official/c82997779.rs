// Archfiend's Usurpation
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 82997779;

pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_EXTRA);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_ARCHFIEND])
    }
}

// In the hand, with Archfiend's Ghastly Glitch (an Archfiend Trap) in the Deck and Graveyard to Set. No Archfiend Ritual
// Monster is ported, so its Ritual Summon never resolves.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (5168381, DECK), (5168381, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARCHFIEND]) && c.is_trap(duel) && c.is_ssetable_full(duel, true, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let b1 = ((e.get_handler(duel).is_location(duel, LOCATION_SZONE) && ft > 0) || ft > 1)
        && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None);
    let b2 = Ritual::target::<Params, RITPROC_GREATER>(duel, a, 0, None)?;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SET);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
        Ritual::target::<Params, RITPROC_GREATER>(duel, a, 1, None)?;
        duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_EXTRA) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let filter = aux::necro_valley_filter(setfilter);
            let sc = duel
                .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first();
            if let Some(sc) = sc
                && duel.sset(tp, sc)? > 0
            {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_description(duel, aux::stringid(ID, 3));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e1);
            }
        }
        2 => Ritual::operation::<Params, RITPROC_GREATER>(duel, a)?,
        _ => {}
    }
    Ok(())
}
