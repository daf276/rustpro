// Raidraptor - Fiend Eagle
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 52323874;

pub const LISTED_SERIES: &[u16] = &[SET_RAIDRAPTOR];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_RAIDRAPTOR], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// On the field with Sangan attached by Xyz Import, and the opponent's Monster Reborn to Special Summon monsters to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_base_attack(duel) > 0 && c.is_special_summoned(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let tc = g.and_then(|g| g.get_first());
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(false) };
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, tc.get_base_attack(duel));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let atk = tc.get_base_attack(duel).max(0);
        duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}
