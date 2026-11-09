// Blue-Eyes Twin Burst Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 2129638;
pub const MATERIAL_SETCODE: &[u16] = &[SET_BLUE_EYES];

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_BLUEEYES_W_DRAGON); 2];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));
    const CANNOT_BE_LIZARD: bool = false;

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_MZONE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// In the Extra Deck, Fusion Summoned from two Blue-Eyes White Dragons on the field, copies to attack, and opponent's copies and Blue-Eyes Ultimate Dragon, which survive its attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (23995346, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 0));
    e7.set_category(duel, CATEGORY_REMOVE);
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_DAMAGE_STEP_END);
    e7.set_condition_mut(duel, rmcon);
    e7.set_target(duel, rmtg);
    e7.set_operation(duel, rmop);
    c.register_effect(duel, e7);
}

fn splimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    a.e.set_label_object_group(duel, bc.map(Into::into));
    bc.is_some_and(|bc| {
        duel.get_attacker() == Some(c) && c.is_status(duel, STATUS_OPPO_BATTLE) && bc.is_on_field(duel) && bc.is_relate_to_battle(duel)
    })
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = label_card(duel, a.e);
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(bc.is_able_to_remove(duel));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let bc = label_card(duel, a.e);
    unported!(duel, bc.is_none());
    if let Some(bc) = bc
        && bc.is_relate_to_battle(duel)
    {
        duel.remove(bc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
