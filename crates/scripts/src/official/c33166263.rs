// Hyperinvoked Aeon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 33166263;

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::GroupFilter(matfilter);
    const MINC: i32 = 2;
    const MAXC: i32 = 99;
}

// In the Extra Deck, Fusion Summoned by one of three Polymerization with Extox Hydra (WATER) and Garura (DARK) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (24094653, HAND),
    (26050548, MZONE),
    (11765832, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, attrtg);
    e2.set_operation(duel, attrop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp)
        && sg.is_none_or(|sg| {
            sg.filter_count(duel, aux::TRUE, Some(c.into())) == 0
                || !sg.is_exists(
                    duel,
                    |sc, duel| sc.is_attribute_full(duel, c.get_attribute(duel), Some(fc), sumtype, tp),
                    1,
                    Some(c.into()),
                )
        })
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let material_count = e.get_handler(duel).get_material(duel).len();
    let location = LOCATION_ONFIELD | LOCATION_GRAVE;
    let g = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, location, location, None);
    if chk == 0 {
        return Ok(material_count > 0 && !g.is_empty());
    }
    e.set_label(duel, material_count as i64);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, PLAYER_EITHER, location as i32);
    if material_count >= 3 {
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 3, 1 - tp, LOCATION_EXTRA as i32);
    }
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let material_count = a.e.get_label(duel);
    let location = LOCATION_ONFIELD | LOCATION_GRAVE;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| c.is_able_to_remove(duel),
        tp,
        location,
        location,
        1,
        material_count as u8,
        None,
    )?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 && material_count >= 3 {
        let extrag = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_EXTRA, None);
        if !extrag.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.confirm_cards(tp, extrag)?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let sg = extrag.select(duel, tp, 1, 3, None)?;
            if !sg.is_empty() {
                duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
            }
            duel.shuffle_extra(1 - tp)?;
        }
    }
    Ok(())
}

fn attrtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(
        CardId::is_faceup,
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let attr = duel.announce_another_attribute(g, tp)?;
    e.set_label(duel, i64::from(attr));
    Ok(false)
}

fn attrop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c);
    let attr = e.get_label(duel);
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, exc.map(CardSet::from));
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e1.set_value(duel, attr as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
