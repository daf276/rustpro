// Thor, Lord of the Aesir
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 30604579;

pub const LISTED_SERIES: &[u16] = &[SET_NORDIC_BEAST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_NORDIC_BEAST], Some(sc), sumtype, tp) || c.is_has_effect(duel, EFFECT_SYNSUB_NORDIC).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// A copy on the field and in the Extra Deck, Synchro Summoned with Guldfaxe of the Nordic Beasts (a Level 4 Tuner), Melffy Rabby (Level 2) and Noble
// Knight Artorigus (Level 4); Dark Hole in the opponent's hand destroys it, with Nordic Beast Gullinbursti (a Tuner) in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (41788781, MZONE),
    (41788781, MZONE),
    (20129614, MZONE),
    (92125819, MZONE),
    (65626958, GRAVE),
    (65626958, GRAVE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    c.register_effect(duel, e4);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let mut pos = c.get_previous_position(duel);
    if c.is_reason(duel, REASON_BATTLE) {
        pos = c.get_battle_position(duel);
    }
    if a.rp != tp
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && pos & POS_FACEUP != 0
    {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NORDIC_BEAST])
        && c.is_type(duel, TYPE_TUNER)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && (duel.get_location_count(tp, LOCATION_MZONE) > 0 || (c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| cfilter(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 1, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(800);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    Ok(false)
}
