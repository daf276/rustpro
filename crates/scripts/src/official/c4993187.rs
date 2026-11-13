// W:P Fancy Ball
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 4993187;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Copies and Zenna's Deceiving Doll Maidens in the Extra Deck, Sangan as Link Material on the field, and the opponent's Sangan on
// the field, sent to the Graveyard by Raigeki, whose trigger the Link Summoned copy negates. The opponent's Linkuriboh is the
// material the second effect offers, for Code Talker with Code Generator in the hand, whose value the extra material check calls.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (7594154, EXTRA),
    (7594154, EXTRA),
    (53413628, EXTRA),
    (30114823, HAND),
    (41999284, MZONE | OPPONENT),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(53413628),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_main_phase_full(Some(1 - a.tp)));
    e2.set_target(duel, linktg);
    e2.set_operation(duel, linkop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if !a.e.get_handler(duel).is_status(duel, STATUS_SPSUMMON_TURN) {
        return false;
    }
    let ev = a.ev as u8;
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && trig_loc & (LOCATION_MZONE | LOCATION_GRAVE) > 0
        && duel.is_chain_disablable(ev)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel));
    }
    aux::remove_until::<aux::DefaultFieldReturnOp>(duel, c, 0, REASON_COST, PHASE_END, ID, a.e, a.tp)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    if relation {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(rc.into()), 1, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8) && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn register_extra_material(duel: &mut Duel, c: CardId) -> EffectId {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_extra_material_operation(duel, extracon);
    e1.set_value(duel, Value::ExtraMaterial(extraval));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    e1
}

fn linksummonable(c: CardId, duel: &Duel) -> bool {
    c.is_link_summonable(duel, None, None)
}

fn linktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let e1 = register_extra_material(duel, a.e.get_handler(duel));
        let res = duel.is_existing_matching_card(linksummonable, tp, LOCATION_EXTRA, 0, 1, None);
        e1.reset(duel);
        return Ok(res);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_chain_limit(
        |duel, re, _, _, _| !(re.is_monster_effect(duel) && re.get_handler(duel).is_link_monster(duel)),
        0,
    );
    Ok(false)
}

fn linkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = c.is_relate_to_effect(duel, e).then(|| register_extra_material(duel, c));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, linksummonable, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.link_summon(tp, sc, None, None)?;
        if let Some(e1) = e1 {
            let eff_code = if duel.get_current_chain() == 1 {
                EVENT_SPSUMMON
            } else {
                EVENT_SPSUMMON_SUCCESS
            };
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e2.set_code(duel, eff_code);
            e2.set_label_object(duel, Some(e1));
            e2.set_operation(duel, |duel, a| {
                if let Some(e1) = a.e.get_label_object(duel) {
                    e1.reset(duel);
                }
                a.e.reset(duel);
                Ok(())
            });
            duel.register_effect(e2, tp);
        }
    } else if let Some(e1) = e1 {
        e1.reset(duel);
    }
    Ok(())
}

fn extracon(duel: &mut Duel, _: CardId, e: EffectId, tp: u8, sg: CardSet, _: CardSet, lc: CardId, _: CardSet, _: i32) -> bool {
    if !sg.is_contains(e.get_handler(duel)) {
        return false;
    }
    let g = duel.get_matching_group(|c, duel| c.is_link_below(duel, 2), tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return true;
    }
    let mut max_count = 1;
    let mut must_include = CardSet::default();
    for eff in duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL) {
        if eff.get_owner(duel).is_code(duel, ID) {
            continue;
        }
        let Value::ExtraMaterial(value) = eff.get_value(duel) else {
            unported!(duel, true);
            continue;
        };
        if !value(
            duel,
            eff,
            ExtraMaterialCall::Get {
                summon_type: SUMMON_TYPE_LINK,
                tp,
                sc: lc,
            },
        )
        .is_empty()
        {
            let handler = eff.get_handler(duel);
            must_include.insert(handler);
            if !sg.intersection(must_include).is_empty() || lc == handler {
                max_count += 1;
            }
        }
    }
    let count = sg.intersection(g).len();
    count > 0 && count <= max_count
}

fn extraval(duel: &mut Duel, _: EffectId, call: ExtraMaterialCall) -> CardSet {
    match call {
        ExtraMaterialCall::Get { summon_type, tp, .. } if summon_type == SUMMON_TYPE_LINK => {
            duel.get_matching_group(|c, duel| c.is_link_below(duel, 2), tp, 0, LOCATION_MZONE, None)
        }
        ExtraMaterialCall::Use { summon_type, sg, tp, .. } => {
            if summon_type & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK && !sg.is_empty() {
                duel.hint(HINT_CARD, tp, u64::from(ID));
            }
            CardSet::default()
        }
        _ => CardSet::default(),
    }
}
