// Gilti-Gearfried the Magical Steel Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 49161188;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 2];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Gaia The Fierce Knight and Queen's Knight (Warriors
// of different Attributes), with Spells in the Graveyard for the cost and an opponent's Mage Power to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (6368038, HAND),
    (25652259, HAND),
    (5318639, GRAVE),
    (5318639, GRAVE),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, atkcost);
    e3.set_condition(duel, atkcon);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DISABLE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, discon);
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, sump: u8, sg: Option<CardSet>) -> bool {
    c.is_race_full(duel, RACE_WARRIOR, Some(fc), sumtype, sump)
        && c.get_attribute(duel) != 0
        && sg.is_none_or(|sg| {
            !sg.is_exists(
                duel,
                |sc, duel| fusfilter(sc, duel, c.get_attribute(duel), fc, sumtype, sump),
                1,
                Some(c.into()),
            )
        })
}

fn fusfilter(c: CardId, duel: &Duel, attr: u32, fc: CardId, sumtype: u64, sump: u8) -> bool {
    c.is_attribute_full(duel, attr, Some(fc), sumtype, sump) && c.is_has_effect(duel, 511002961).is_none()
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.e.get_label(duel) != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    if g.len() == g.filter_count(duel, |mc, duel| mc.is_location(duel, LOCATION_MZONE), None)
        && let Some(e1) = e.get_label_object(duel)
    {
        e1.set_label(duel, i64::from(ID));
    }
}

fn costfil(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_remove_as_cost(duel)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfil, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, costfil, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel)
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_faceup(duel) && bc.is_relate_to_battle(duel))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let value = c.get_defense(duel).max(0);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) && value > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, (value + 1) / 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return false };
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    let tg = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_contains(c)) && duel.is_chain_disablable(a.ev as u8) && loc != LOCATION_DECK
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.negate_effect(a.ev as u8) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
