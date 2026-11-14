// Mermail Abysstrite
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 59170782;

pub const LISTED_SERIES: &[u16] = &[SET_MERMAIL];
// In the Extra Deck, Xyz Summoned with Level 3 Normal Monsters, with Mermail Abysspikes on the field and in the Graveyard for the
// opponent's Elfnote Lucina to attack, Mage Power to target, and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13039848, MZONE),
    (40826495, MZONE),
    (52121290, MZONE),
    (62193699, MZONE),
    (58471134, MZONE),
    (58471134, GRAVE),
    (13597785, MZONE | OPPONENT),
    (83746708, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, tgcon1);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, tgop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition_mut(duel, tgcon2);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, tgop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn tgcon1(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return false };
    if a.rp == tp || !re.is_spell_trap_effect(duel) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(ev, CHAININFO_TARGET_CARDS) else {
        return false;
    };
    let Some(tc) = g.get_first().filter(|_| g.len() == 1) else {
        return false;
    };
    let c = a.e.get_handler(duel);
    if tc == c
        || tc.is_controler(duel, 1 - tp)
        || tc.is_facedown(duel)
        || !tc.is_location(duel, LOCATION_MZONE)
        || !tc.is_set_card(duel, &[SET_MERMAIL])
    {
        return false;
    }
    duel.check_mut(|duel| duel.check_chain_target(ev, c))
}

fn tgop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.check_chain_target(a.ev as u8, c) {
        duel.change_target_card(a.ev as u8, c.into());
    }
    Ok(())
}

fn tgcon2(duel: &mut Duel, a: &Args) -> bool {
    if a.tp == duel.get_turn_player() {
        return false;
    }
    let (Some(at), Some(attacker)) = (duel.get_attack_target(), a.eg.and_then(|eg| eg.get_first())) else {
        return false;
    };
    if at.is_faceup(duel) && at.is_set_card(duel, &[SET_MERMAIL]) {
        let (ag, _) = attacker.get_attackable_target(duel);
        return ag.is_contains(a.e.get_handler(duel));
    }
    false
}

fn tgop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e)
        && let Some(attacker) = duel.get_attacker()
        && !attacker.is_immune_to_effect(duel, a.e)
    {
        duel.change_attack_target(Some(c), false);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MERMAIL]) && c.get_code(duel) != ID && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
