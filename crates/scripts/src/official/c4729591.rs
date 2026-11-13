// Appliancer Kappa Scale
use super::c3507053::{Procedure, register_cannot_be_link_material};
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4729591;

// Link Summoned from an Appliancer monster, with Appliancer Link monsters in the Extra Deck and the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78225596, EXTRA),
    (78225596, EXTRA),
    (78447174, MZONE),
    (78447174, MZONE),
    (78447174, MZONE),
    (81769387, MZONE),
    (77102944, MZONE),
    (78447174, HAND),
    (78447174, DECK),
    (3507053, EXTRA),
    (30118200, EXTRA),
    (13597785, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (78447174, GRAVE),
    (30118200, GRAVE),
];

pub const LISTED_NAMES: &[u32] = &[ID];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    register_cannot_be_link_material(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_label(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| !spcon(duel, a));
    e3.set_label(duel, 0);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, colinked: bool) -> bool {
    c.is_set_card(duel, &[SET_APPLIANCER])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
        && !c.is_code(duel, ID)
        && ((c.is_type(duel, TYPE_LINK) && colinked) || (c.is_level_below(duel, 4) && !colinked))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let colinked = e.get_label(duel) == 1;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, colinked), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let colinked = e.get_label(duel) == 1;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, colinked), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
