// Ritual Beast Ulti-Gaiapelio
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 56655675;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST, SET_RITUAL_BEAST_ULTI, SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST];
pub const MATERIAL_SETCODE: &[u16] = &[SET_RITUAL_BEAST, SET_RITUAL_BEAST_TAMER, SET_SPIRITUAL_BEAST, SET_RITUAL_BEAST_ULTI];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_RITUAL_BEAST_ULTI], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_RITUAL_BEAST_TAMER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SPIRITUAL_BEAST], Some(fc), sumtype, tp)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::Int(0));
    const CONDITION: Option<fn(&Duel, u8) -> bool> = Some(|_, _| true);
    const SUMTYPE: Option<u32> = Some(1);

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, LOCATION_MZONE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// In the Extra Deck, contact Summoned with Ritual Beast Ulti-Kimunfalcos, Ritual Beast Tamer Elder and Spiritual Beast Rampengu
// on the field, with Ritual Beast monsters in hand to banish and the opponent's Raigeki to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (58811192, MZONE),
    (14513016, MZONE),
    (88123329, MZONE),
    (14513016, HAND),
    (88123329, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        re != a.e
            && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
            && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
            && duel.is_chain_negatable(a.ev as u8)
    })
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_able_to_remove_as_cost(duel)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, filter, LOCATION_HAND)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
