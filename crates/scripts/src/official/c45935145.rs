// Wollow, Founder of the Drudge Dragons
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 45935145;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// Two Level 6 Normal Monsters to Xyz Summon it, Xyz Import with Dragon Rebellion and Sangan to attach, and Normal Monsters
// in the opponent's Graveyard to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (70781052, GRAVE | OPPONENT),
    (88819587, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| 100 * duel.get_field_group_count(e.get_handler_player(duel), 0, LOCATION_GRAVE) as i32),
    );
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[efftgcost, effdetachcost]));
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, detach_1: bool, detach_2: bool) -> bool {
    (detach_1 && c.is_able_to_deck(duel)) || (detach_2 && setfilter(c, duel, e, tp))
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if c.is_spell_trap(duel) {
        return c.is_ssetable(duel);
    }
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP | POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn efftgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let detach_1 = c.check_remove_overlay_card(duel, tp, 1, REASON_COST);
    let detach_2 = c.check_remove_overlay_card(duel, tp, 2, REASON_COST);
    let filter = |c: CardId, duel: &Duel| tgfilter(c, duel, e, tp, detach_1, detach_2);
    if chk == 0 {
        return Ok((detach_1 || detach_2) && duel.is_existing_target(filter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    Ok(false)
}

fn tccheck<const OP: u8>(duel: &Duel, a: &Args) -> bool {
    let Some(tc) = duel.get_first_target() else { return true };
    (OP == 1 && tc.is_able_to_deck(duel)) || setfilter(tc, duel, a.e, a.tp)
}

fn effdetachcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let choices: [CostChoice; 2] = [
        (Cost::detach_from_self::<1, 1>, aux::stringid(ID, 1), tccheck::<1>),
        (Cost::detach_from_self::<2, 2>, aux::stringid(ID, 2), tccheck::<2>),
    ];
    Cost::choice(duel, a, chk, &choices)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    match Cost::get_choice(duel, e) {
        Some(1) => {
            e.set_category(duel, CATEGORY_TODECK);
            duel.set_operation_info(0, CATEGORY_TODECK, tc.map(Into::into), 1, tp, 0);
        }
        Some(2) => {
            let categ = CATEGORY_SET | CATEGORY_LEAVE_GRAVE;
            if tc.is_some_and(|tc| tc.is_monster(duel)) {
                e.set_category(duel, CATEGORY_SPECIAL_SUMMON | categ);
                duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, tc.map(Into::into), 1, tp, 0);
            } else {
                e.set_category(duel, categ);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    match Cost::get_choice(duel, e) {
        Some(1) => {
            duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
        Some(2) => {
            if tc.is_monster(duel)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP | POS_FACEDOWN_DEFENSE)? > 0
                && tc.is_facedown(duel)
            {
                duel.confirm_cards(1 - tp, tc)?;
            } else if tc.is_ssetable(duel) {
                duel.sset(tp, tc)?;
            }
        }
        _ => {}
    }
    Ok(())
}
