// Dragunity Lord - Georgius
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 70522875;

// On the field, with Dragunity Knight - Luin in the Extra Deck and the GY to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12496261, EXTRA), (12496261, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        tunermatcheck(duel, g, lc, sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_group(duel)
            .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_DRAGUNITY]), 1, None)
    });
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && re.get_handler(duel).is_location(duel, LOCATION_GRAVE)),
    );
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, ID);
    e2a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2a.set_target(duel, sptg);
    e2a.set_operation(duel, spop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_condition(duel, |duel, a| {
        let eg = a.eg.unwrap_or_default();
        !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None)
    });
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_spell_trap_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
    });
    e3.set_cost(duel, discost);
    e3.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
        Ok(false)
    });
    e3.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn tunermatcheck(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
    g.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_TUNER, Some(lc), sumtype, tp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !(c.is_level_above(duel, 5) && c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        mmz_chk
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, mmz_chk),
            tp,
            LOCATION_EXTRA | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, mmz_chk));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_card(duel) && c.is_able_to_grave_as_cost(duel)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, discostfilter, LOCATION_ONFIELD)
}
