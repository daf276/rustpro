// Virtual World Shell - Jaja
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 90664857;

// On the field with Xyz Import to attach Sangan, two Level 3 monsters of one Type and Attribute (Eldeen) in the Graveyard, and the
// opponent's attackers to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (6367785, GRAVE),
    (6367785, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, indtg);
    e1.set_operation(duel, indop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn indtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3000);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let same = sg.get_class_count(duel, |c, duel| c.get_original_race(duel) as i64) == 1
        && sg.get_class_count(duel, |c, duel| i64::from(c.get_original_attribute(duel))) == 1;
    (same, false)
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(CardId::is_monster, a.tp, LOCATION_GRAVE, 0, None);
    aux::select_unselect_group_check(duel, g, 2, 2, &rescon)
}

fn battle_opponent(duel: &Duel, e: EffectId) -> Option<CardId> {
    let tc = duel.get_attacker();
    if tc == Some(e.get_handler(duel)) {
        duel.get_attack_target()
    } else {
        tc
    }
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let tc = battle_opponent(duel, e);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_relate_to_battle(duel) && tc.is_able_to_remove(duel) && tc.is_controler(duel, 1 - tp)));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, tc.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = battle_opponent(duel, e)
        && tc.is_relate_to_battle(duel)
        && tc.is_controler(duel, 1 - tp)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
