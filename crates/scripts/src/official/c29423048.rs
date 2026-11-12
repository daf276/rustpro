// Infernal Flame Banshee
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 29423048;

// Twice in the Extra Deck, Xyz Summoned with Level 4 Pyro Normal Monsters, with Pyros in the Deck, and the opponent's Bottomless
// Trap Holes and Dimensional Prisons to banish it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (71407486, MZONE),
    (5464695, MZONE),
    (5388481, MZONE),
    (78861134, MZONE),
    (78861134, DECK),
    (91731841, DECK),
    (29401950, SZONE | OPPONENT),
    (29401950, SZONE | OPPONENT),
    (29401950, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, thtgtg);
    e1.set_operation(duel, thtgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PYRO) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let send = |duel: &mut Duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop);
    aux::to_hand_or_else(duel, tc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| c.is_faceup(duel) && c.is_race(duel, RACE_PYRO),
                tp,
                LOCATION_MZONE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        let ct = duel.get_matching_group_count(|c, duel| c.is_faceup(duel) && c.is_monster(duel), tp, LOCATION_REMOVED, 0, None);
        if ct > 0 {
            c.update_attack(duel, ct as i32 * 100, Some(RESETS_STANDARD_PHASE_END), None, None);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
