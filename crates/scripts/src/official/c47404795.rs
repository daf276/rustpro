// Abyss Actor - Super Producer
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 47404795;
const FANTASTIC_THEATER: u32 = 77297908;

pub const LISTED_NAMES: &[u32] = &[FANTASTIC_THEATER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp), 1, None)
    }
}

// Link Summoned from Fiend Normal Monsters, with Abyss Playhouse - Fantastic Theater and Abyss Actor - Extras in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (3557275, MZONE),
    (12057781, MZONE),
    (53830602, MZONE),
    (99171160, MZONE),
    (FANTASTIC_THEATER, DECK),
    (FANTASTIC_THEATER, DECK),
    (88412339, DECK),
    (88412339, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn fldfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, FANTASTIC_THEATER) && c.is_field_spell(duel) && !c.is_forbidden(duel)
}

fn pendfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ABYSS_ACTOR]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || duel.destroy(tc, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g1 = duel.get_matching_group(fldfilter, tp, LOCATION_DECK, 0, None);
    let g2 = duel.get_matching_group(pendfilter, tp, LOCATION_DECK, 0, None);
    let b1 = !g1.is_empty();
    let b2 = !g2.is_empty() && duel.check_pendulum_zones(tp);
    if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 1))?) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    if op == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, fldfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.place_field_spell(tc, tp)?;
        }
    } else if op == Some(2) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, pendfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(tc) = g.get_first() {
            duel.break_effect()?;
            duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
        }
    }
    Ok(())
}
