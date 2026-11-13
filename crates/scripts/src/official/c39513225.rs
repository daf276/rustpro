// Seventh Barian's
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 39513225;

// In the hand, with Thousand Umbral Horror and V Umbral Horror in the Deck to add, and Number 39: Utopia the Envoy of Light
// (Rank 4) Xyz Summoned with Mystical Elf and Flying Kamakiri #2 for the End Phase damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (2932235, DECK),
    (2932235, DECK),
    (39053882, DECK),
    (76504386, EXTRA),
    (76504386, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (15025844, MZONE),
    (3134241, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_DAMAGE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1a.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1a.set_range(duel, LOCATION_FZONE);
    e1a.set_target_range(duel, 1, 1);
    e1a.set_target_spsummon(duel, |duel, e, c, sump, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA)
            && !c.is_set_card(duel, &[SET_NUMBER])
            && e.get_handler(duel).get_flag_effect(duel, ID + u32::from(sump)) >= 2
    });
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1b.set_range(duel, LOCATION_FZONE);
    e1b.set_operation(duel, countop);
    c.register_effect(duel, e1b);
    let e1c = duel.create_effect(c);
    e1c.set_type(duel, EFFECT_TYPE_FIELD);
    e1c.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1c.set_code(duel, EFFECT_LEFT_SPSUMMON_COUNT);
    e1c.set_range(duel, LOCATION_FZONE);
    e1c.set_target_range(duel, 1, 1);
    e1c.set_value(
        duel,
        Value::SpSummonCount(|duel, e, _, tp| (2 - e.get_handler(duel).get_flag_effect(duel, ID + u32::from(tp)) as i32).max(0)),
    );
    c.register_effect(duel, e1c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

fn countop(duel: &mut Duel, a: &Args) -> Script<()> {
    for sc in a.eg.unwrap_or_default() {
        if sc.is_summon_location(duel, LOCATION_EXTRA) && (!sc.is_set_card(duel, &[SET_NUMBER]) || sc.is_facedown(duel)) {
            let summon_player = sc.get_summon_player(duel);
            a.e.get_handler(duel)
                .register_flag_effect(duel, ID + u32::from(summon_player), RESETS_STANDARD_PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_UMBRAL_HORROR]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    if duel.get_field_group_count(tp, LOCATION_HAND, 0) > 0 {
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn damage_amount(duel: &Duel, tp: u8) -> i32 {
    400 * duel.get_matching_group_count(aux::faceup_filter(CardId::is_xyz_monster), tp, LOCATION_MZONE, LOCATION_MZONE, None) as i32
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let dam = damage_amount(duel, a.tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let dam = damage_amount(duel, tp);
    if dam > 0 {
        duel.damage_full(tp, dam as u32, REASON_EFFECT, true, None)?;
        duel.damage_full(1 - tp, dam as u32, REASON_EFFECT, true, None)?;
        duel.rd_complete()?;
    }
    Ok(())
}
