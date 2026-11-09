// Allied Code Talker @Ignister
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 39138610;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Linked to Firewall Dragons and Elfnote Lucina, which it can release, against the activations of the opponent's Lucina; a copy
// Link Summoned and sent to the Graveyard is its own 2300 ATK Cyberse monster to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (5043010, MZONE),
    (5043010, MZONE),
    (5043010, MZONE),
    (13597785, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
    });
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && c.is_attack(duel, &[2300])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_linked_zone_full(duel, tp);
    let mut ft = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && zone > 0 && ft > 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp, zone),
            tp,
            LOCATION_GRAVE,
            0,
            ft as u8,
            ft as u8,
            None,
        )?;
        if !g.is_empty() {
            let ct = duel.special_summon_full(g, 0, tp, tp, false, false, POS_FACEUP, zone)?;
            if ct > 0 {
                c.update_attack(duel, ct * 500, None, None, None);
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn negcostfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_link_monster(duel) && lg.is_contains(c) && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    if chk == 0 {
        return Ok(!lg.is_empty() && duel.check_release_group_cost(tp, |c, duel| negcostfilter(c, duel, lg), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| negcostfilter(c, duel, lg), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if relation {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(rc.into()), 1, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, 0, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
