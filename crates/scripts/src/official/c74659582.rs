// Sleipnir the Runick Mane
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 74659582;
const TOKEN: u32 = ID + 1;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_RUNICK], Some(fc), sumtype, tp)); 2];
}

// Copies on the field and one to Special Summon by Runick Tip, the opponent's Sangan to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (31562086, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(TOKEN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, rmcon);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, tkcon);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    (duel.is_turn_player(a.tp) && duel.is_main_phase()) || (duel.is_turn_player(1 - a.tp) && duel.is_battle_phase())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel
        .select_target(tp, rmfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(c.is_relate_to_effect(duel, a.e) && tc.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    let g = CardSet::from_iter([c, tc]);
    aux::remove_until::<aux::DefaultFieldReturnOp>(duel, g, 0, REASON_EFFECT, PHASE_END, ID, a.e, a.tp)?;
    Ok(())
}

fn tkcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| {
        eg.is_exists(
            duel,
            |c, duel| c.is_controler(duel, 1 - a.tp) && c.is_previous_location(duel, LOCATION_DECK),
            1,
            None,
        )
    })
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_RUNICK, TYPES_TOKEN, 1500, 1500, 4, RACE_FAIRY, ATTRIBUTE_LIGHT)
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    duel.is_player_can_special_summon_monster_full(tp, TOKEN, token_stats(), POS_FACEUP_ATTACK, tp, 0)
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 || !can_summon_token(duel, tp) {
        return Ok(());
    }
    if let Some(token) = duel.create_token(tp, TOKEN) {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}
