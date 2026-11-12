// Ragnaraika Selene Snapper
use super::c42307760::{Procedure, selfspop, selfsptg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 75352507;

// Link Summoned from Insect and Plant Normal Monsters, with monsters in the Graveyard to banish and Spells/Traps to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (58192742, MZONE),
    (58192742, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (58192742, GRAVE),
    (14181608, GRAVE),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop::<ID, 2>);
    c.register_effect(duel, e2);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, sg.len(), None),
        false,
    )
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(
        |c, duel| c.is_monster(duel) && c.is_able_to_remove_as_cost(duel),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if chk == 0 {
        return Ok(!rg.is_empty());
    }
    let g = aux::select_unselect_group(duel, rg, 1, 2, &|duel, sg, _, _| rescon(duel, sg, tp), tp, HINTMSG_REMOVE)?;
    e.set_label(duel, g.len() as i64);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
