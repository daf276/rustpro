// Junk Berserker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 59771339;

pub const LISTED_SERIES: &[u16] = &[SET_JUNK];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[CARD_JUNK_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[CARD_JUNK_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_SYNCHRON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
// On the field, or Synchro Summoned with Junk Synchron and Overdrive (a Level 4 Normal Monster), with a Junk Synchron in the GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (CARD_JUNK_SYNCHRON, MZONE),
    (CARD_JUNK_SYNCHRON, GRAVE),
    (2311603, MZONE),
    (2311603, MZONE),
    (2311603, MZONE | OPPONENT),
    (2311603, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, atkcost);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn atkcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_JUNK])
        && c.is_monster(duel)
        && c.get_attack(duel) > 0
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atkcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, atkcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        e.set_label(duel, sc.get_attack(duel).into());
        duel.remove(sc, POS_FACEUP, REASON_COST)?;
    }
    Ok(false)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_controler(duel, 1 - tp)
    {
        let amount = -(e.get_label(duel) as i32);
        tc.update_attack(duel, amount, None, Some(e.get_handler(duel)), None);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel)) && duel.get_attack_target().is_some_and(|bc| bc.is_defense_pos(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = duel.get_attack_target();
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(bc) = duel.get_attack_target()
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
