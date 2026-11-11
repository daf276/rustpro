// Teller of Fairy Tails
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4026187;

pub const LISTED_NAMES: &[u32] = &[CARD_PRINCE_OF_FAIRIES];
pub const MATERIAL_SETCODE: &[u16] = &[SET_FAIRY_TAIL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned with Polymerization from two Fairy Tail - Luna, facing an Elfnote Lucina to negate, and
// Matchgiru in the Graveyard to banish. The destruction needs a "Fairy Prince" among the materials, which only an opponent's
// monster renamed earlier can be.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (86937530, HAND),
    (86937530, HAND),
    (19144622, GRAVE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_FAIRY_TAIL]));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 0));
    e2a.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.e.get_label(duel) == 1
    });
    e2a.set_target(duel, destg);
    e2a.set_operation(duel, desop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_SINGLE);
    e2b.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2b.set_label_object(duel, Some(e2a));
    e2b.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, negcon);
    e3.set_cost(duel, negcost);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp) && c.is_set_card_full(duel, &[SET_FAIRY_TAIL], Some(fc), sumtype, tp)
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let found = c
        .get_material(duel)
        .is_exists(duel, |m, duel| m.is_code(duel, CARD_PRINCE_OF_FAIRIES), 1, None);
    if let Some(e2a) = e.get_label_object(duel) {
        e2a.set_label(duel, i64::from(found));
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 500);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if !g.is_empty() {
        let dam = duel.destroy(g, REASON_EFFECT)? as u32 * 500;
        if dam > 0 {
            duel.damage(1 - tp, dam, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FAIRY_TAIL]) && c.is_able_to_remove_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, negcostfilter, LOCATION_HAND | LOCATION_GRAVE)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
        }
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
