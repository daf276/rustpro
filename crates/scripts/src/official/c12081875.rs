// Thunder Dragon Thunderstormech
use engine::Operation;
use engine::Target;
use engine::constants::MSG_SELECT_EFFECTYN;

use super::c4965193::{runfn, runfn_args};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 12081875;
const THUNDER_DRAGON: u32 = 31786629;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_THUNDER, Some(lc), sumtype, tp)
    }
}

// Thunder Kids to Link Summon with and to banish, Thunder Dragon (a discard effect) in the Graveyard and the Deck, and the
// opponent's Raigeki and Dark Hole to destroy Thunder monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (15510988, MZONE),
    (15510988, MZONE),
    (15510988, MZONE),
    (15510988, MZONE),
    (15510988, MZONE),
    (15510988, MZONE),
    (THUNDER_DRAGON, GRAVE),
    (THUNDER_DRAGON, GRAVE),
    (THUNDER_DRAGON, DECK),
    (THUNDER_DRAGON, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, applytg);
    e1.set_operation(duel, applyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
}

fn applyfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_THUNDER_DRAGON])
        && c.is_monster(duel)
        && c.is_able_to_deck(duel)
        && c.is_faceup(duel)
        && c.get_own_effects(duel)
            .into_iter()
            .any(|eff| eff.has_self_discard_cost(duel) && runfn(duel, eff, tp))
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| applyfilter(c, duel, tp),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(
        tp,
        |c, duel| applyfilter(c, duel, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    let effs: Vec<EffectId> = tc
        .get_own_effects(duel)
        .into_iter()
        .filter(|eff| eff.has_self_discard_cost(duel))
        .collect();
    let options: Vec<(bool, u64)> = effs.iter().map(|&eff| (runfn(duel, eff, tp), eff.get_description(duel))).collect();
    let op = if options.len() == 1 {
        Some(1)
    } else {
        duel.select_effect(tp, &options)?
    };
    let Some(te) = op.and_then(|op| effs.get(op - 1).copied()) else {
        return Ok(false);
    };
    duel.hint(HINT_OPSELECTED, 1 - tp, te.get_description(duel));
    duel.clear_target_card();
    tc.create_effect_relation(duel, e);
    e.set_label(duel, te.get_label(duel));
    e.set_label_object(duel, te.get_label_object(duel));
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(f) => {
            f(duel, &runfn_args(e, tp), 1, None)?;
            te.set_label(duel, e.get_label(duel));
            te.set_label_object(duel, e.get_label_object(duel));
            duel.clear_operation_info(0);
        }
        _ => unported!(duel, true),
    }
    e.set_label_object(duel, Some(te));
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tc.into()), 1, tp, 0);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    let tc = te.get_handler(duel);
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let op = te.get_operation(duel);
    unported!(
        duel,
        op.is_some_and(|op| !matches!(op, Operation::Event(_) | Operation::Promptless(_)))
    );
    if let Some(Operation::Event(f) | Operation::Promptless(f)) = op {
        e.set_label(duel, te.get_label(duel));
        e.set_label_object(duel, te.get_label_object(duel));
        f(duel, a)?;
    }
    if tc.is_able_to_deck(duel) {
        let opt = if duel.get_field_group_count(tp, LOCATION_DECK, 0) > 0 {
            duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
        } else {
            0
        };
        duel.break_effect()?;
        duel.send_to_deck(tc, PLAYER_NONE, opt, REASON_EFFECT)?;
    }
    e.set_label(duel, 0);
    e.set_label_object(duel, None);
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_THUNDER)
        && c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn repcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a
            .eg
            .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None))
            && duel.is_existing_matching_card(repcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 3, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, repcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 3, 3, None)?;
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(true)
}
