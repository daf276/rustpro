// Levia-Dragon - Daedalus
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 37721209;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI];

// On the field with Umi in the hand to activate, and the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (CARD_UMI, HAND), (CARD_UMI, HAND), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descostfilter(c: CardId, duel: &Duel, hc: CardId) -> bool {
    c.is_code(duel, CARD_UMI)
        && c.is_faceup(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(
            aux::TRUE,
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(CardSet::from_iter([c, hc])),
        )
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|tc, duel| descostfilter(tc, duel, c), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |tc, duel| descostfilter(tc, duel, c), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, a.e).then(|| c.into());
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
