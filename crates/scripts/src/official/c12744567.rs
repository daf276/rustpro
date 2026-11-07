// Number C101: Silent Honor DARK
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 12744567;
const SILENT_HONOR_ARK: u32 = 48739166;

pub const LISTED_NAMES: &[u32] = &[SILENT_HONOR_ARK];

// In the Extra Deck, Xyz Summoned with three Level 5 monsters, with the opponent's monsters to attach, Silent Honor ARK in
// the Graveyard to be Special Summoned, and Dark Hole and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (35565537, MZONE),
    (86088138, MZONE),
    (90876561, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (SILENT_HONOR_ARK, GRAVE),
    (53129443, HAND),
    (12580477, HAND),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_TOKEN) && c.is_able_to_change_controler(duel) && c.is_special_summoned(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY) && c.is_previous_location(duel, LOCATION_MZONE) && c.get_overlay_count(duel) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(|tc, duel| tc.is_code(duel, SILENT_HONOR_ARK), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    let rec = c.get_base_attack(duel);
    duel.set_target_player(tp);
    duel.set_target_param(rec as u32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, rec);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_existing_matching_card(|tc, duel| tc.is_code(duel, SILENT_HONOR_ARK), tp, LOCATION_GRAVE, 0, 1, None) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        duel.break_effect()?;
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
        duel.recover(p, d, REASON_EFFECT)?;
    }
    Ok(())
}
