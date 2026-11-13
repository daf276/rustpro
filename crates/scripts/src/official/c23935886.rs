// Draco Masters of the Tenyi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 23935886;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned from Monk of the Tenyi (Link-1) and Mystery Shell Dragon, with non-Effect Monsters in the Graveyard
// and the opponent's Effect Monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32519092, MZONE),
    (32519092, MZONE),
    (18108166, MZONE),
    (18108166, MZONE),
    (18108166, GRAVE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.is_type(duel, TYPE_EFFECT).into()));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(desfilter, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE)) && c.is_non_effect_monster(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_matching_card(desfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if ct > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, desfilter, tp, 0, LOCATION_MZONE, 1, ct as u8, None)?;
        if !g.is_empty() {
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
