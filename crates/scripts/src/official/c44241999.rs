// Goblin Biker Troika Griare
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 44241999;

// In the Extra Deck, Xyz Summoned from Level 3 Goblins (Goblin Biker Dugg Charger, Scrap Goblin), and on the field with Xyz Import attaching
// Sangan, with monsters in the opponent's hand to Summon and Elfnote Lucina on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (24079759, MZONE),
    (24079759, MZONE),
    (83135907, MZONE),
    (83135907, MZONE),
    (24079759, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GOBLIN]));
    e1.set_condition(duel, |duel, _| duel.get_overlay_count(0, 1, 1) >= 3);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, poscond);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    e4.set_target(duel, negatktg);
    e4.set_operation(duel, negatkop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_change_position(duel) && c.is_location(duel, LOCATION_MZONE) && c.is_can_be_effect_target(duel, e)
}

fn poscond(duel: &Duel, a: &Args) -> bool {
    let (e, eg) = (a.e, a.eg.unwrap_or_default());
    eg.is_exists(duel, |c, duel| cfilter(c, duel, e), 1, None) && !eg.is_contains(e.get_handler(duel))
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp, eg) = (a.e, a.tp, a.eg.unwrap_or_default());
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && cfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| cfilter(c, duel, e), 1, None) && duel.check_remove_overlay_card(tp, 1, 1, 1, REASON_EFFECT));
    }
    let tc = if eg.len() == 1 {
        eg
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        eg.filter_select(duel, tp, |c, duel| cfilter(c, duel, e), 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(tc), 1, 0, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    if duel.remove_overlay_card(tp, 1, 1, 1, 1, REASON_EFFECT)? > 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

fn negatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_remove_overlay_card(a.tp, 1, 1, 1, REASON_EFFECT))
}

fn negatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.remove_overlay_card(a.tp, 1, 1, 1, 1, REASON_EFFECT)? > 0 {
        duel.negate_attack()?;
    }
    Ok(())
}
