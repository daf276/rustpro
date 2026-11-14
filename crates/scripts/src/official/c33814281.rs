// Dark Contract with Patent License
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 33814281;

pub const LISTED_SERIES: &[u16] = &[SET_DDD, SET_DD];
// Copies in the hand to replace each other, D/D/D monsters of each Extra Deck type on the field to match the opponent's summons,
// and D/D monsters in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (9024198, MZONE),
    (3758046, MZONE),
    (70576413, MZONE),
    (74583607, MZONE),
    (72291412, GRAVE),
    (45206713, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 2))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
    e2.set_label_object_group(duel, Some(CardSet::default()));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_label_object(duel, Some(e2));
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

const EXTRA_TYPES: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

fn damcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let ty = c.get_type(duel) & EXTRA_TYPES;
    c.is_summon_player(duel, 1 - tp)
        && ty != 0
        && duel.is_existing_matching_card(|c, duel| damcfilter2(c, duel, ty), tp, LOCATION_MZONE, 0, 1, None)
}

fn damcfilter2(c: CardId, duel: &Duel, ty: u32) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DDD]) && c.is_type(duel, ty)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| damcfilter(c, duel, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e2) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e2.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e2.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    if chk == 0 {
        return Ok(!g.is_empty() && duel.get_flag_effect(tp, ID) == 0);
    }
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::damage(duel, a)?;
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let ty = g.into_iter().fold(0, |ty, tc| ty | tc.get_type(duel)) & EXTRA_TYPES;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_target_spsummon(duel, sumlimit);
    e1.set_label(duel, ty.into());
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn sumlimit(duel: &Duel, e: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    let typ = e.get_label(duel) as u32;
    if c.is_monster(duel) {
        c.is_type(duel, typ)
    } else {
        c.is_original_type(duel, typ)
    }
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_DD])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_EXTRA))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_EXTRA | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_EXTRA | LOCATION_GRAVE)
}
