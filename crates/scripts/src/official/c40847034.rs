// Kewl Tune Playlist
use engine::Operation;
use engine::Target;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 40847034;

pub const LISTED_SERIES: &[u16] = &[SET_KEWL_TUNE];

// The fields of e:GetChainData().
const TARGET_CARD: u8 = 0;
const TARGET_EFFECT: u8 = 1;

// In hand, with Kewl Tune Mix, Reco and Track Maker in the GY and on the field (their Synchro material effects destroy
// or return the opponent's Sangan).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (16509007, GRAVE),
    (89392810, GRAVE),
    (42781164, GRAVE),
    (16509007, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 2, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

// An EVENT_BE_MATERIAL effect whose target passes, called as the Lua does with an empty eg and `e` as re.
fn is_available(duel: &Duel, eff: EffectId, e: EffectId, tp: u8) -> bool {
    if eff.get_code(duel) != EVENT_BE_MATERIAL {
        return false;
    }
    match eff.get_target(duel) {
        Target::None => true,
        Target::Action(tg) => {
            let a = Args {
                e: eff,
                tp,
                eg: Some(CardSet::default()),
                ep: PLAYER_NONE,
                re: Some(e),
                r: REASON_EFFECT,
                rp: PLAYER_NONE,
                ..Args::default()
            };
            duel.check_mut(|duel| tg(duel, &a, 0, None)).unwrap_or(false)
        }
        _ => {
            unported!(duel, true);
            false
        }
    }
}

fn applyfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_KEWL_TUNE]) && c.is_monster(duel) && c.is_faceup(duel)) {
        return false;
    }
    c.is_able_to_hand(duel) || c.get_own_effects(duel).into_iter().any(|eff| is_available(duel, eff, e, tp))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let filter = |c, duel: &Duel| applyfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let tc = duel
        .select_target(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?
        .and_then(|g| g.get_first());
    let Some(tc) = tc else { return Ok(false) };
    let available_effs: Vec<EffectId> = tc
        .get_own_effects(duel)
        .into_iter()
        .filter(|&eff| is_available(duel, eff, e, tp))
        .collect();
    let eff = if available_effs.len() > 1 {
        let descriptions: Vec<u64> = available_effs.iter().map(|eff| eff.get_description(duel)).collect();
        let op = duel.select_option(tp, &descriptions)?;
        usize::try_from(op).ok().and_then(|op| available_effs.get(op).copied())
    } else {
        available_effs.first().copied()
    };
    if let Some(eff) = eff {
        duel.hint(HINT_OPSELECTED, 1 - tp, eff.get_description(duel));
        duel.clear_target_card();
        tc.create_effect_relation(duel, e);
        e.set_label(duel, eff.get_label(duel));
        e.set_label_object(duel, eff.get_label_object(duel));
        match eff.get_target(duel) {
            Target::None => {}
            Target::Action(tg) => {
                tg(duel, a, 1, None)?;
                duel.clear_operation_info(0);
            }
            _ => unported!(duel, true),
        }
    }
    e.set_chain_data_group(duel, TARGET_CARD, tc.into());
    if let Some(eff) = eff {
        e.set_chain_data_effect(duel, TARGET_EFFECT, eff);
    }
    e.set_category(duel, CATEGORY_TOHAND);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tc.into()), 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = e.get_chain_data_group(duel, TARGET_CARD).and_then(|g| g.get_first());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        let mut break_chk = false;
        if let Some(te) = e.get_chain_data_effect(duel, TARGET_EFFECT) {
            let op = te.get_operation(duel);
            unported!(
                duel,
                op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
            );
            if tc.is_faceup(duel)
                && let Some(Operation::Event(op) | Operation::Promptless(op)) = op
            {
                op(duel, a)?;
                break_chk = true;
            }
        }
        if break_chk {
            duel.break_effect()?;
        }
        if duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0 {
            duel.shuffle_hand(tp)?;
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    super::c14442329::register_tuner_limit(duel, e, tp, aux::stringid(ID, 1));
    Ok(())
}
