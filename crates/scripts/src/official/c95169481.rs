// Diamond Dire Wolf
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 95169481;

// Twice in the Extra Deck, Xyz Summoned with Level 4 Beast Normal Monsters, with Sangan on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (7225792, MZONE),
    (5818798, MZONE),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && duel.is_existing_target(aux::TRUE, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel
        .select_target(tp, desfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g2 = duel
        .select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(g1))?
        .unwrap_or_default();
    let mut g = g1;
    g.merge(g2);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
