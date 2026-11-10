// Lunalight Silver Hound
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 35763582;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_PERFUME_DANCER: u32 = 81196066;
const DARK_HOLE: u32 = 53129443;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Lunalight Black Sheep in the Deck to Special Summon once the opponent's Dark Hole destroys it, and in the
// Graveyard with Lunalight Perfume Dancer to negate that Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (DARK_HOLE, HAND | OPPONENT),
    (ID, GRAVE),
    (LUNALIGHT_PERFUME_DANCER, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        sc.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT + RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_controler(duel, a.e.get_owner_player(duel)));
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_LUNALIGHT])
        });
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        sc.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.re.is_some_and(|re| re.is_spell_trap_effect(duel)) && trig_loc & LOCATION_ONFIELD > 0 && duel.is_chain_negatable(a.ev as u8)
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_type(duel, TYPE_FUSION) && c.is_able_to_remove_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(
            c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(negcostfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, negcostfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
