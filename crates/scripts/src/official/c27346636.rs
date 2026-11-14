// Gladiator Beast Heraklinos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 27346636;
const LAQUARI: u32 = 78868776;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GLADIATOR];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(LAQUARI),
        FusionMaterial::Filter(gladiator),
        FusionMaterial::Filter(gladiator),
    ];
}

fn gladiator(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_GLADIATOR], Some(fc), sumtype, tp)
}

pub struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(
            |c, duel| c.is_monster(duel) && c.is_able_to_deck_or_extra_as_cost(duel),
            tp,
            LOCATION_ONFIELD,
            0,
            None,
        )
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        duel.confirm_cards(1 - tp, g)?;
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    e.get_handler(duel).get_location(duel) != LOCATION_EXTRA
}

// Contact Summoned from Laquari and Gladiator Beast Hoplomus on the field, with cards to discard and Spells to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (LAQUARI, MZONE),
    (4253484, MZONE),
    (4253484, MZONE),
    (90582719, HAND),
    (90582719, HAND),
    (53129443, HAND | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_faceup(duel) || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
