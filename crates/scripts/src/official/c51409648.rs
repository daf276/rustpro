// Rindbrumm the Striking Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 51409648;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACES_BEAST_BWARRIOR_WINGB, Some(fc), sumtype, tp)),
    ];
}

// On the field and in the Graveyard with Fallen of Albaz, against the opponent's Tri-Brigade Arms Bucephalus II whose effect it negates.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_ALBAZ, GRAVE),
    (10019086, MZONE | OPPONENT),
    (10019086, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_monster_effect(duel) && re.is_active_type(duel, TYPE_EXTRA))
    });
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, sprmtg);
    e2.set_operation(duel, sprmop);
    c.register_effect(duel, e2);
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.negate_effect(a.ev as u8)
        && duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection_full(g, true);
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rmv_chk: bool, sp_chk: bool) -> bool {
    c.is_code(duel, CARD_ALBAZ)
        && ((sp_chk && c.is_able_to_remove(duel)) || (rmv_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn sprmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let rmv_chk = c.is_able_to_remove(duel);
    let sp_chk = c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, e, tp, rmv_chk, sp_chk));
    }
    let filter = |c, duel: &Duel| tgfilter(c, duel, e, tp, rmv_chk, sp_chk);
    if chk == 0 {
        return Ok((rmv_chk || sp_chk)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let mut g = duel
        .select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 0, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, g: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && g.is_exists(duel, CardId::is_able_to_remove, 1, Some(c.into()))
}

fn sprmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let g = CardSet::from_iter([c, tc]);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp, g), 1, 1, None)?;
    if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.remove(g.difference(sg), POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
