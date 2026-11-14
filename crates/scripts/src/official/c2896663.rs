// ZW - Dragonic Halberd
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 2896663;

pub const LISTED_SERIES: &[u16] = &[SET_ZEXAL, SET_ZW, SET_UTOPIA];
// On the field to equip to Number 39: Utopia, with ZW - Ultimate Shield beside it and in the hand to equip and Special Summon, Zexal
// Construction in the Deck to search and the opponent's Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (84013237, MZONE),
    (18865703, MZONE),
    (18865703, HAND),
    (18865703, HAND),
    (62623659, DECK),
    (62623659, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3207);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_category(duel, CATEGORY_EQUIP);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
    aux::add_zw_equip_limit(
        duel,
        c,
        None,
        |duel, tc, _, tp| filter(tc, duel) && tc.is_controler(duel, tp),
        equipop,
        e3,
    );
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_BATTLE_DESTROYING);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, spcon);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ZEXAL]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_UTOPIA])
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    match duel.get_first_target() {
        Some(tc)
            if duel.get_location_count(tp, LOCATION_SZONE) > 0
                && tc.is_relate_to_effect(duel, e)
                && !tc.is_facedown(duel)
                && tc.get_controler(duel) == tp =>
        {
            equipop(duel, c, e, tp, tc)
        }
        _ => {
            duel.send_to_grave(c, REASON_EFFECT)?;
            Ok(())
        }
    }
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !aux::equip_and_limit_register(duel, c, e, tp, tc)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 3000);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(ec) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    ec.get_battle_target(duel).is_some_and(|bc| {
        Some(ec) == a.e.get_handler(duel).get_equip_target(duel) && ec.is_status(duel, STATUS_OPPO_BATTLE) && bc.is_monster(duel)
    })
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ZW]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(eqt) = e.get_handler(duel).get_equip_target(duel) else {
        return Ok(false);
    };
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && eqt
                .get_equip_group(duel)
                .is_exists(duel, |c, duel| spfilter(c, duel, e, tp), 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_SZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(eqt) = e.get_handler(duel).get_equip_target(duel) else {
        return Ok(());
    };
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if ft < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let spg = eqt
        .get_equip_group(duel)
        .filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp), 1, ft as u8, None)?;
    if !spg.is_empty() {
        duel.special_summon(spg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
