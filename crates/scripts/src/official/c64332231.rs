// Hieratic Sun Dragon Overlord of Heliopolis
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 64332231;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

// Sangan on both sides to release and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_ex(tp, CardId::is_releasable_by_effect, 1, None)
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct1 = duel.get_matching_group_count(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let rg = duel.select_release_group_ex(tp, CardId::is_releasable_by_effect, (1, ct1 as u8), None)?;
    let ct2 = duel.release(rg, REASON_EFFECT)?;
    if ct2 == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, ct2 as u8, ct2 as u8, None)?;
    duel.hint_selection(dg);
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
