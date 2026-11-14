// Borrelcode Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 67288539;

pub const LISTED_SERIES: &[u16] = &[SET_TOPOLOGIC];
// In the Extra Deck, Link Summoned from three Rokket effect monsters to battle a Sangan, and in the Graveyard to banish
// Borreload Riot Dragon (DARK, 3000 ATK) for Topologic Trisbaena.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (25554552, MZONE),
    (25554552, MZONE),
    (25554552, MZONE),
    (68464358, MZONE),
    (7987191, MZONE),
    (72529749, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_condition(duel, incon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_cost(duel, Cost::self_banish);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn indfilter(c: CardId, duel: &Duel, cc: CardId) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_LINK) && c.get_linked_group(duel).contains(cc)
}

fn incon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(|tc, duel| indfilter(tc, duel, c), 0, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_flag_effect(duel, ID) > 0 && c.is_link_summoned(duel) && c.get_battle_target(duel).is_some()
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if c.get_material(duel).len() == 3 {
        c.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT | RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE),
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
    }
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_able_to_remove(duel)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_attack_above(duel, 3000)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c),
            tp,
            LOCATION_EXTRA | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rmc: CardId) -> bool {
    c.is_set_card(duel, &[SET_TOPOLOGIC])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && ((c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, Some(rmc.into()), c) > 0)
            || (c.is_location(duel, LOCATION_GRAVE) && duel.get_mzone_count(tp, Some(rmc.into())) > 0))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 0, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = duel.select_matching_card(
        tp,
        |c, duel| rmfilter(c, duel, e, tp),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    if let Some(rc) = rg.get_first()
        && duel.remove(rg, POS_FACEUP, REASON_EFFECT)? > 0
        && rc.is_location(duel, LOCATION_REMOVED)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, rc));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
