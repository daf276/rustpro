// Guoglim, Spear of the Ghoti
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 2405631;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FISH, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// A copy on the field and one in the Extra Deck, Synchro Summoned with Paces (a Level 2 Fish Tuner) and Amphibian Beast (Level 6),
// with Sangan on the other side to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (46037983, MZONE),
    (67371383, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

const REASON_SYNCHRO_MATERIAL: u32 = REASON_MATERIAL + REASON_SYNCHRO;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_controler(duel, 1 - tp) && tc.is_able_to_remove(duel)));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = a.e.get_handler(duel).get_battle_target(duel)
        && tc.is_relate_to_battle(duel)
        && tc.is_controler(duel, 1 - tp)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn mgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sync: CardId) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & REASON_SYNCHRO_MATERIAL == REASON_SYNCHRO_MATERIAL
        && c.get_reason_card(duel) == Some(sync)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = c.get_material(duel);
    let ct = mg.len();
    let sumtype = c.get_summon_type(duel);
    if duel.remove(c, POS_FACEUP, REASON_EFFECT)? > 0
        && sumtype == SUMMON_TYPE_SYNCHRO
        && ct > 0
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && ct as i32 <= duel.get_location_count(tp, LOCATION_MZONE)
        && mg.filter_count(duel, aux::necro_valley_filter(|mc, duel| mgfilter(mc, duel, e, tp, c)), None) == ct
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.special_summon(mg, 0, tp, tp, false, false, POS_FACEUP)? != ct as i32 {
            return Ok(());
        }
        for tc in mg {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3300);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
            e1.set_property(
                duel,
                EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_CLIENT_HINT,
            );
            e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
            e1.set_value(duel, LOCATION_REMOVED as i32);
            tc.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}
