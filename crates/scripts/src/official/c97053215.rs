// G Golem Stubborn Menhir
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 97053215;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(lc), sumtype, tp)
    }
}

// Link Summoned from EARTH Normal Monsters (Doron, Flame Viper), destroyed by Raigeki from the opponent and revived with Monster
// Reborn, with EARTH monsters in the Graveyard to recover.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (756652, MZONE),
    (2830619, MZONE),
    (756652, GRAVE),
    (2830619, GRAVE),
    (83764718, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_GRAVE));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_summonable_card(duel)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel, e, tp, ft));
    }
    let tgfilter = |c, duel: &Duel| filter(c, duel, e, tp, ft);
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    let g = duel.select_target(tp, tgfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let check = |c: CardId, duel: &Duel| {
        c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && duel.get_location_count(tp, LOCATION_MZONE) > 0
    };
    aux::to_hand_or_else(duel, tc, tp, check, |duel| spsummon(duel, e, tp, tc), aux::stringid(ID, 1))
}

fn spsummon(duel: &mut Duel, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e2, true);
        let e3 = duel.create_effect(c);
        e3.set_description(duel, 3300);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e3.set_value(duel, LOCATION_REMOVED as i32);
        tc.register_effect_full(duel, e3, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
