// Invoked Dee Anima
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 80635735;

pub const LISTED_SERIES: &[u16] = &[SET_ALEISTER];

const CHOICE: u8 = 0;

pub const MATERIAL_SETCODE: &[u16] = &[SET_ALEISTER];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ALEISTER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_summon_location(duel, LOCATION_EXTRA) && c.is_location(duel, LOCATION_MZONE)),
    ];
}

// On the field and in the Extra Deck for a Fusion Summon with Polymerization (Aleister the Invoker and an Invoked monster from the
// Extra Deck), with Aleister monsters in the Deck, the opponent's Sangan sent to its Graveyard, and its Spells/Traps to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (12307878, EXTRA),
    (12307878, EXTRA),
    (86120751, MZONE),
    (86120751, HAND),
    (86120751, HAND),
    (86120751, DECK),
    (86120751, DECK),
    (43989315, MZONE),
    (24094653, HAND),
    (24094653, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && chain::is_triggering_location(duel, a.ev as u8, LOCATION_GRAVE) && duel.is_chain_disablable(a.ev as u8)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn exd_monsters(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(|c, duel| c.is_summon_location(duel, LOCATION_EXTRA), tp, 0, LOCATION_MZONE, None)
}

fn spell_traps(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (g1, g2) = (exd_monsters(duel, tp), spell_traps(duel, tp));
    let (b1, b2) = (!g1.is_empty(), !g2.is_empty());
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    let g = if choice == Some(1) { g1 } else { g2 };
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = match a.e.get_chain_data(duel, CHOICE) {
        Some(1) => exd_monsters(duel, tp),
        Some(2) => spell_traps(duel, tp),
        _ => return Ok(()),
    };
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ALEISTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.negate_effect(a.ev as u8);
    }
    Ok(())
}
