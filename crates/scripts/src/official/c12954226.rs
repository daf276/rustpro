// Circle of the Fairies
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 12954226;

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Summoned Skull (Level 6), against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_INSECT | RACE_PLANT));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, damcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn insect_or_plant(race: u64) -> bool {
    race & (RACE_INSECT | RACE_PLANT) != 0
}

fn damcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let rc = c.get_reason_card(duel);
    let Some(rc) = rc.filter(|_| c.is_location(duel, LOCATION_GRAVE)) else {
        return false;
    };
    if c.is_previous_controler(duel, tp) {
        insect_or_plant(c.get_previous_race_on_field(duel))
    } else if rc.is_relate_to_battle(duel) {
        rc.is_controler(duel, tp) && rc.is_race(duel, RACE_INSECT | RACE_PLANT)
    } else {
        rc.is_previous_controler(duel, tp) && insect_or_plant(rc.get_previous_race_on_field(duel))
    }
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| damcfilter(c, duel, tp), 1, None))
}

fn damtgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_can_be_effect_target(duel, e) && c.get_previous_attack_on_field(duel) > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && damtgfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| damtgfilter(c, duel, e), 1, None));
    }
    let mut tg = eg;
    if eg.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        tg = eg.filter_select(duel, tp, |c, duel| damtgfilter(c, duel, e), 1, 1, None)?;
    }
    duel.set_target_card(tg);
    let value = tg.get_first().map_or(0, |tc| (tc.get_previous_attack_on_field(duel) + 1) / 2);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, value);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, value);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let value = duel.damage(1 - tp, ((tc.get_previous_attack_on_field(duel) + 1) / 2) as u32, REASON_EFFECT)?;
    if value > 0 {
        duel.break_effect()?;
        duel.recover(tp, value, REASON_EFFECT)?;
    }
    Ok(())
}
