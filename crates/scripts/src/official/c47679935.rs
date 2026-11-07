// Magical Meltdown
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 47679935;
const ALEISTER_THE_INVOKER: u32 = 86120751;
const ALBION_THE_BRANDED_DRAGON: u32 = 87746184;

pub const LISTED_NAMES: &[u32] = &[ALEISTER_THE_INVOKER];

// Aleister the Invoker to search; Polymerization with Fallen of Albaz and Petit Angel to Fusion Summon Albion the Branded Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, LOCATION_FZONE as u16),
    (ALEISTER_THE_INVOKER, DECK),
    (24094653, HAND),
    (68468459, HAND),
    (38142739, HAND),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(ALBION_THE_BRANDED_DRAGON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_INACTIVATE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_value(duel, Value::Chain(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_condition(duel, limcon);
    e3.set_operation(duel, limop);
    c.register_effect(duel, e3);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_code(duel, EVENT_CHAIN_END);
    e5.set_operation(duel, limop2);
    c.register_effect(duel, e5);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ALEISTER_THE_INVOKER) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let p = e.get_handler_player(duel);
    let tp: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT)
        .is_some_and(|te| p == tp && te.is_has_category(duel, CATEGORY_FUSION_SUMMON))
}

fn limcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| c.is_summon_player(duel, a.tp) && c.is_fusion_summoned(duel),
            1,
            None,
        )
    })
}

fn chainlm(_: &Duel, _: EffectId, rp: u8, tp: u8, _: i64) -> bool {
    tp == rp
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match duel.get_current_chain() {
        0 => duel.set_chain_limit_till_chain_end(chainlm, 0),
        1 => {
            let c = e.get_handler(duel);
            c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_CHAINING);
            e1.set_operation(duel, resetop);
            duel.register_effect(e1, tp);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EVENT_BREAK_EFFECT);
            e2.set_reset(duel, RESET_CHAIN);
            duel.register_effect(e2, tp);
        }
        _ => {}
    }
    Ok(())
}

fn resetop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).reset_flag_effect(duel, ID);
    a.e.reset(duel);
    Ok(())
}

fn limop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_flag_effect(duel, ID) != 0 {
        duel.set_chain_limit_till_chain_end(chainlm, 0);
    }
    c.reset_flag_effect(duel, ID);
    Ok(())
}
