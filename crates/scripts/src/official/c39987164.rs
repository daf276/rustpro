// Vylon Disigma
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 39987164;

// Twice in the Extra Deck, Xyz Summoned with Kojikocy (Level 4), with Elfnote Lucinas (FIRE Effect Monsters) on the opponent's side
// to equip and fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
// The destruction needs a battle with a monster of the equipped monster's Attribute, which random play reaches in about 3 of 64 scenarios.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition_mut(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attack_pos(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_able_to_change_controler(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    Ok(false)
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    c.equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)?;
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_attack_pos(duel)
    {
        equipop(duel, c, e, tp, tc)?;
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.get_flag_effect(duel, ID) > 0 && c.is_attribute(duel, att)
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let e = a.e;
    let c = e.get_handler(duel);
    let mut dt = duel.get_attacker();
    if dt == Some(c) {
        dt = duel.get_attack_target();
    }
    let Some(dt) = dt.filter(|dt| !dt.is_facedown(duel)) else {
        return false;
    };
    e.set_label_object_group(duel, Some(dt.into()));
    let att = dt.get_attribute(duel);
    c.get_equip_group(duel).is_exists(duel, |c, duel| desfilter(c, duel, att), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let dt = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, dt, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(dt) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let eqg = a.e.get_handler(duel).get_equip_group(duel);
    let att = dt.get_attribute(duel);
    if eqg.is_empty() || !eqg.is_exists(duel, |c, duel| desfilter(c, duel, att), 1, None) {
        return Ok(());
    }
    if dt.is_relate_to_battle(duel) {
        duel.destroy(dt, REASON_EFFECT)?;
    }
    Ok(())
}
