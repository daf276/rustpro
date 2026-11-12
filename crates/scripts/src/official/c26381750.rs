// Soaring Eagle Above the Searing Land
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE};

const ID: u32 = 26381750;

// On the field as the Level 1 Tuner of a Synchro Summon of Red Dragon Archfiend with Zoa, a Level 7 Normal Monster, and four
// Laval monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (24311372, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (2407147, GRAVE),
    (15169262, GRAVE),
    (45439263, GRAVE),
    (8041569, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn laval_classes(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(|c, duel| c.is_set_card(duel, &[SET_LAVAL]), tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO && laval_classes(duel, a.tp) >= 3
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if laval_classes(duel, tp) < 3 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
