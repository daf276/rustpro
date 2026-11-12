// Ultimate Dragonic Utopia Ray
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 75402014;
pub const XYZ_NUMBER: u32 = 39;

pub const LISTED_NAMES: &[u32] = &[56840427];

// On the field with ZW - Ultimate Shield in the Deck to equip when it is attacked by Sangan, with Xyz Import to attach Sangan, which it detaches to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (18865703, DECK),
    (18865703, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_target(duel, super::c68679595::eqtg::<{ LOCATION_DECK | LOCATION_HAND }>);
    e1.set_operation(duel, super::c68679595::eqop::<{ LOCATION_DECK | LOCATION_HAND }>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, eqcon);
    e2.set_target(duel, super::c68679595::eqtg::<{ LOCATION_DECK | LOCATION_HAND }>);
    e2.set_operation(duel, super::c68679595::eqop::<{ LOCATION_DECK | LOCATION_HAND }>);
    c.register_effect(duel, e2);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.contains(a.e.get_handler(duel)))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ZW]) && c.is_original_type(duel, TYPE_MONSTER)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_negatable(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_ONFIELD));
    }
    if chk == 0 {
        return Ok(c.get_equip_group(duel).is_exists(duel, cfilter, 1, None)
            && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let ct = c.get_equip_group(duel).filter(duel, cfilter, None).len() as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = e1.clone_effect(duel);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}
