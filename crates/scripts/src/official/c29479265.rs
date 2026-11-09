// Unchained Abomination
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 29479265;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp), 1, None)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck and on the field, with Unchained Twins - Rakea and Aruha on the field to destroy cards.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (53624265, MZONE), (26236560, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon2);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_condition(duel, |_, _| true);
    e3.set_count_limit(duel, 1, (ID, 2));
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_EFFECT) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, cfilter, 1, None)) && !a.re.is_some_and(|re| re.get_handler(duel).is_code(duel, ID))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon2(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| c.is_reason(duel, REASON_BATTLE),
            1,
            Some(a.e.get_handler(duel).into()),
        )
    })
}
