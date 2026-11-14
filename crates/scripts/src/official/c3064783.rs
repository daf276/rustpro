// Shining Umbral Horror
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 3064783;

pub const LISTED_SERIES: &[u16] = &[SET_UMBRAL_HORROR, SET_MASQUERADE, SET_NUMBER];
// In the hand and on the field, with V Umbral Horror in the hand to Normal Summon and Umbral Horror monsters in the Deck to draw, and
// Number 104: Masquerade Vain (3 Level 4 materials) in the Extra Deck with Mystical Elf to be Xyz Summoned with it counting as two.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (39053882, HAND),
    (39053882, DECK),
    (2932235, DECK),
    (2932235, DECK),
    (16908882, EXTRA),
    (16908882, EXTRA),
    (15025844, MZONE),
    (15025844, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DRAW);
    e1a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_range(duel, LOCATION_HAND);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, spcon);
    e1a.set_target(duel, sptg);
    e1a.set_operation(duel, spop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DOUBLE_XYZ_MATERIAL);
    e2.set_value(duel, 1);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_xyz_material_operation(duel, |duel, _, xyz, _, minxyzct| {
        minxyzct >= 3 && xyz.is_set_card(duel, &[SET_NUMBER])
    });
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_UMBRAL_HORROR]) || (c.is_set_card(duel, &[SET_MASQUERADE]) && c.is_xyz_monster(duel)))
        && c.is_summon_player(duel, tp)
        && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.draw(tp, 1, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        let drawn_card = duel.get_operated_group().get_first();
        unported!(duel, drawn_card.is_none());
        let Some(drawn_card) = drawn_card else { return Ok(()) };
        if drawn_card.is_set_card(duel, &[SET_UMBRAL_HORROR]) && drawn_card.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
            if !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
                return duel.shuffle_hand(tp);
            }
            duel.break_effect()?;
            duel.special_summon(drawn_card, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
