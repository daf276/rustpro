// Necrovalley
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 47355498;

// In the hand to activate, with Gravekeeper's Spy on the field, and the opponent's Monster Reborn and Gravekeeper's Spy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (24317029, MZONE),
    (24317029, GRAVE),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (24317029, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_FZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1a.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GRAVEKEEPERS]));
    e1a.set_value(duel, 500);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_code(duel, EFFECT_CANNOT_REMOVE);
    e2a.set_range(duel, LOCATION_FZONE);
    e2a.set_target_range(duel, LOCATION_GRAVE, 0);
    e2a.set_condition(duel, |duel, a| {
        let controler = a.e.get_handler(duel).get_controler(duel);
        duel.is_player_affected_by_effect(controler, EFFECT_NECRO_VALLEY_IM).is_none()
    });
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_target_range(duel, 0, LOCATION_GRAVE);
    e2b.set_condition(duel, |duel, a| {
        let controler = a.e.get_handler(duel).get_controler(duel);
        duel.is_player_affected_by_effect(1 - controler, EFFECT_NECRO_VALLEY_IM).is_none()
    });
    c.register_effect(duel, e2b);
    let e3a = e2a.clone_effect(duel);
    e3a.set_code(duel, EFFECT_NECRO_VALLEY);
    c.register_effect(duel, e3a);
    let e3b = e2b.clone_effect(duel);
    e3b.set_code(duel, EFFECT_NECRO_VALLEY);
    c.register_effect(duel, e3b);
    let e3c = e3a.clone_effect(duel);
    e3c.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3c.set_target_range(duel, 1, 0);
    c.register_effect(duel, e3c);
    let e3d = e3b.clone_effect(duel);
    e3d.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3d.set_target_range(duel, 0, 1);
    c.register_effect(duel, e3d);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAIN_SOLVING);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e5.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_target_range(duel, 1, 1);
    e5.set_target_spsummon(duel, cannotsptg);
    c.register_effect(duel, e5);
}

fn discheckfilter(c: CardId, duel: &Duel, not_im0: bool, not_im1: bool, re: EffectId) -> bool {
    let not_im = if c.is_controler(duel, 0) { not_im0 } else { not_im1 };
    not_im && c.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_some() && c.is_relate_to_effect(duel, re)
}

fn discheck(duel: &Duel, ev: u8, category: u64, re: EffectId, not_im0: bool, not_im1: bool) -> bool {
    let Some(info) = duel.get_operation_info(ev, category) else {
        return false;
    };
    if (category == CATEGORY_LEAVE_GRAVE || info.op_param == LOCATION_GRAVE as i32) && info.op_count > 0 && info.op_cards.is_none() {
        match info.op_player {
            0 => return not_im0,
            1 => return not_im1,
            PLAYER_ALL => return not_im0 || not_im1,
            PLAYER_EITHER => return not_im0 && not_im1,
            _ => {}
        }
    }
    info.op_cards
        .is_some_and(|tg| !tg.is_empty() && tg.is_exists(duel, |c, duel| discheckfilter(c, duel, not_im0, not_im1, re), 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    let Some(re) = a.re else { return Ok(()) };
    if !duel.is_chain_disablable(ev) || re.get_handler(duel).is_has_effect(duel, EFFECT_NECRO_VALLEY_IM).is_some() {
        return Ok(());
    }
    let not_im0 = duel.is_player_affected_by_effect(0, EFFECT_NECRO_VALLEY_IM).is_none();
    let not_im1 = duel.is_player_affected_by_effect(1, EFFECT_NECRO_VALLEY_IM).is_none();
    let categories = [
        CATEGORY_SPECIAL_SUMMON,
        CATEGORY_REMOVE,
        CATEGORY_TOHAND,
        CATEGORY_TODECK,
        CATEGORY_TOEXTRA,
        CATEGORY_EQUIP,
        CATEGORY_SET,
        CATEGORY_LEAVE_GRAVE,
    ];
    if categories
        .into_iter()
        .any(|category| discheck(duel, ev, category, re, not_im0, not_im1))
    {
        duel.negate_effect(ev);
    }
    Ok(())
}

fn cannotsptg(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, sumeff: Option<EffectId>) -> bool {
    let Some(sumeff) = sumeff else { return false };
    c.is_location(duel, LOCATION_GRAVE)
        && !sumeff.is_activated(duel)
        && !sumeff.is_has_property(duel, EFFECT_FLAG_CANNOT_DISABLE)
        && duel
            .is_player_affected_by_effect(c.get_controler(duel), EFFECT_NECRO_VALLEY_IM)
            .is_none()
        && c.is_has_effect(duel, EFFECT_NECRO_VALLEY_IM).is_none()
        && sumeff.get_handler(duel).is_has_effect(duel, EFFECT_NECRO_VALLEY_IM).is_none()
}
