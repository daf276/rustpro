// Gorgonic Umbral Horror
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 76548531;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER, SET_UMBRAL_HORROR];
// In the hand and on the field with only Umbral Horror monsters, with non-Rock Umbral Horror monsters in the Deck to add, and Number 104:
// Masquerade Vain (3 Level 4 materials) in the Extra Deck to be Xyz Summoned with V Umbral Horror and this card counting as two.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (39053882, MZONE),
    (39053882, MZONE),
    (2932235, DECK),
    (2932235, DECK),
    (39053882, DECK),
    (16908882, EXTRA),
    (16908882, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_set_card(duel, &[SET_NUMBER]) && c.is_xyz_monster(duel))
    });
    c.register_effect(duel, e1);
    aux::add_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| {
        !(c.is_original_type(duel, TYPE_XYZ) && c.is_original_set_card(duel, &[SET_NUMBER]))
    });
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DOUBLE_XYZ_MATERIAL);
    e3.set_value(duel, 1);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_xyz_material_operation(duel, |duel, _, xyz, _, minxyzct| {
        minxyzct >= 3 && xyz.is_set_card(duel, &[SET_NUMBER])
    });
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    g.is_empty() || g.len() == g.filter_count(duel, aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_UMBRAL_HORROR])), None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_UMBRAL_HORROR]) && c.is_monster(duel) && !c.is_race(duel, RACE_ROCK) && c.is_able_to_hand(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sc = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sc.is_empty() && duel.send_to_hand(sc, tp, REASON_EFFECT)? > 0 {
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    Ok(())
}
