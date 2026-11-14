// Graceful & Skull Dice
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 76630812;

pub const ROLL_DICE: bool = true;
pub const LISTED_NAMES: &[u32] = &[CARD_DARK_TIME_WIZARD];

// In the hand and the Graveyard, with Panthers to buff, the opponent's Sangan to weaken and its Level 4 Normal Monsters to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (77482666, MZONE),
    (77482666, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (1784619, HAND | OPPONENT),
    (1784619, HAND | OPPONENT),
    (1784619, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DICE + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_DICE + CATEGORY_DESTROY);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2a.set_code(duel, EVENT_CUSTOM + ID);
    e2a.set_range(duel, LOCATION_GRAVE);
    e2a.set_cost(duel, Cost::self_banish);
    e2a.set_target(duel, destg);
    e2a.set_operation(duel, desop);
    e2a.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2b.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2b.set_range(duel, LOCATION_GRAVE);
    e2b.set_label_object(duel, Some(e2a));
    e2b.set_operation(duel, regop);
    c.register_effect(duel, e2b);
    let e2c = e2b.clone_effect(duel);
    e2c.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2c);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, a.tp, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let total_atkdef = duel.toss_dice(tp, 2)?.iter().map(|&die| i32::from(die)).sum::<i32>() * 200;
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, _, c| c.lists_code(duel, CARD_DARK_TIME_WIZARD));
    e1a.set_value(duel, total_atkdef);
    e1a.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1a, tp);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    duel.register_effect(e1b, tp);
    aux::register_client_hint(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 3)));
    let e2a = e1a.clone_effect(duel);
    e2a.set_target_range(duel, 0, LOCATION_MZONE);
    e2a.set_target_card(duel, |_, _, _| true);
    e2a.set_value(duel, -total_atkdef);
    duel.register_effect(e2a, tp);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    duel.register_effect(e2b, tp);
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, opp: u8) -> bool {
    c.is_summon_player(duel, opp) && c.is_can_be_effect_target(duel, e) && c.is_location(duel, LOCATION_MZONE)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_damage_step() {
        return Ok(());
    }
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| desfilter(c, duel, e, 1 - tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e2a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e2a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e2a.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let opp = 1 - tp;
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| desfilter(c, duel, e, opp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && desfilter(chkc, duel, e, opp));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        g.select(duel, tp, 1, 1, None)?
    } else {
        g
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(tc), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && duel.toss_dice(tp, 2)?.iter().map(|&die| u32::from(die)).sum::<u32>() >= 6
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
