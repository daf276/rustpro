// Virtual World Phoenix - Fanfan
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 27069566;

// On the field with Xyz Import to attach Sangan, Virtual World Mai-Hime - Lulu in the Deck to Special Summon, and the opponent's
// Raigeki to destroy it, with Sangan on the opponent's field and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (49088914, DECK),
    (49088914, DECK),
    (49088914, DECK),
    (12580477, HAND | OPPONENT),
    (86483512, MZONE),
    (86483512, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(aux::faceup_filter(CardId::is_able_to_remove), tp, 0, LOCATION_ONFIELD, 1, None)
                && duel.is_existing_target(CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g1 = duel.select_target(
        tp,
        aux::faceup_filter(CardId::is_able_to_remove),
        tp,
        0,
        LOCATION_ONFIELD,
        1,
        1,
        None,
    )?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g2 = duel.select_target(tp, CardId::is_able_to_remove, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    let mut g = g1.unwrap_or_default();
    g.merge(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && (c.is_reason(duel, REASON_EFFECT)
            || (c.is_reason(duel, REASON_BATTLE) && duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - tp))))
        && c.is_xyz_summoned(duel)
        && a.rp == 1 - tp
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VIRTUAL_WORLD])
        && c.is_monster(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let same = sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1
        && sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 1;
    (same, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
            return Ok(false);
        }
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_SPSUMMON)?;
    if sg.len() == 2 {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
