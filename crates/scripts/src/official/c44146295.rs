// Mirrorjade the Iceblade Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 44146295;
const POLYMERIZATION: u32 = 24094653;
const TORRENTIAL_TRIBUTE: u32 = 53582587;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_type_full(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK, Some(fc), sumtype, tp)
        }),
    ];
}

// In the Extra Deck, Fusion Summoned with Polymerization from Albaz and an Extra Deck monster, with Albion the Branded Dragon in
// the Extra Deck as the cost, and Torrential Tributes and Sangan on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (87746184, EXTRA),
    (POLYMERIZATION, HAND),
    (POLYMERIZATION, HAND),
    (CARD_ALBAZ, MZONE),
    (59822133, MZONE),
    (TORRENTIAL_TRIBUTE, SZONE | OPPONENT),
    (TORRENTIAL_TRIBUTE, SZONE | OPPONENT),
    (TORRENTIAL_TRIBUTE, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, rmcost);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION) && c.is_able_to_grave_as_cost(duel) && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, cfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(
            duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None) && c.get_flag_effect(duel, ID) == 0,
        );
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        let reset = if duel.is_turn_player(tp) {
            RESET_OPPO_TURN
        } else {
            RESET_SELF_TURN
        };
        c.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END | reset,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel) && a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(aux::TRUE, a.tp, 0, LOCATION_MZONE, 1, None)
    });
    e1.set_operation(duel, desop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let sg = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
