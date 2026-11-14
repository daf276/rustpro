// Altergeist Kidolga
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 76685519;

pub const LISTED_SERIES: &[u16] = &[SET_ALTERGEIST];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_ALTERGEIST], Some(lc), sumtype, tp)
    }
}

// Copies attack each other's trigger (it points only to its left, so few monsters), with Mighty Guard in the opponent's Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (62327910, GRAVE | OPPONENT),
    (62327910, GRAVE | OPPONENT),
    (12977245, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.and_then(|eg| eg.get_first()).is_some_and(|tc| {
        a.ep == 1 - a.tp && tc.is_controler(duel, a.tp) && tc.is_set_card(duel, &[SET_ALTERGEIST]) && tc != a.e.get_handler(duel)
    })
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp) & ZONES_MMZ;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && spfilter(chkc, duel, e, tp, zone));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, zone), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, zone), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_linked_zone_full(duel, tp) & ZONES_MMZ;
    if !c.is_relate_to_effect(duel, e) || zone == 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)? > 0
    {
        if c.get_attack_announced_count(duel) > 0 {
            tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
        tc.create_relation(duel, c, RESET_EVENT | RESETS_STANDARD);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) == 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        // The Lua sets this on e1 (already registered), which drops its client hint; e2 keeps no property.
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition(duel, |duel, a| {
            let handler = a.e.get_handler(duel);
            a.e.get_label_object_group(duel)
                .and_then(|g| g.get_first())
                .is_some_and(|tc| tc.is_relate_to_card(duel, handler))
        });
        e2.set_operation(duel, |duel, a| {
            if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
                tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
            }
            Ok(())
        });
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ALTERGEIST]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
