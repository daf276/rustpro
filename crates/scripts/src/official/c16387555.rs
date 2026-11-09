// Kewl Tune Cue
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 16387555;

pub const LISTED_NAMES: &[u32] = &[ID];

struct TunerFromHand;

impl HandMaterial for TunerFromHand {
    const ID: u32 = ID;

    fn material_filter(c: CardId, duel: &Duel) -> bool {
        c.is_type(duel, TYPE_TUNER)
    }
}

// In the hand to Normal Summon and on the field with Kewl Tune Mix and Track Maker to Synchro Summon it, Tuners in the
// Deck and Graveyard to Special Summon, and a Deck for the opponent to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (16509007, MZONE),
    (16509007, DECK),
    (16509007, GRAVE),
    (42781164, EXTRA),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_hand_material_effect::<TunerFromHand>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e2.set_target(duel, exctg);
    e2.set_operation(duel, excop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_TUNER) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId) -> bool {
    if c.is_monster(duel) {
        !c.is_type(duel, TYPE_TUNER)
    } else {
        c.is_monster_card(duel) && !c.is_original_type(duel, TYPE_TUNER)
    }
}

fn exctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_DECK) >= 2 && duel.is_player_can_remove(tp, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_DECK as i32);
    Ok(false)
}

fn excop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, 0, LOCATION_DECK) == 0 {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    duel.confirm_decktop(1 - tp, 2)?;
    let g = duel.get_decktop_group(1 - tp, 2);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let sg = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    duel.confirm_decktop(1 - tp, 1)?;
    if duel.get_field_group_count(tp, 0, LOCATION_DECK) <= 1 {
        return Ok(());
    }
    if duel.select_option(tp, &[aux::stringid(ID, 3), aux::stringid(ID, 4)])? == 0 {
        return Ok(());
    }
    if let Some(tc) = g.difference(sg).get_first() {
        duel.move_sequence(tc, 1)?;
    }
    Ok(())
}
