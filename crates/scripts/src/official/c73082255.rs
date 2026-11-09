// The Zombie Vampire
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 73082255;

// In the Extra Deck, Xyz Summoned with two Dragon Core Hexers (Level 8).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (38955728, MZONE), (38955728, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_XYZ_LEVEL);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_range(duel, LOCATION_EXTRA);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| {
        c.is_level_above(duel, 1) && c.get_owner(duel) != e.get_handler_player(duel)
    });
    e1.set_value(
        duel,
        Value::XyzLevel(|duel, _, c, rc| if rc.is_code(duel, ID) { 8 } else { c.get_level(duel) as i32 }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::EffectPlayer(tgval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DECKDES + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, gytg);
    e3.set_operation(duel, gyop);
    c.register_effect(duel, e3);
}

fn tgval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel)
        && re.get_activate_location(duel) == LOCATION_MZONE
        && !rc.is_summon_location(duel, LOCATION_GRAVE)
        && rc.is_special_summoned(duel)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 4) && duel.is_player_can_discard_deck(1 - tp, 4));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, PLAYER_ALL, 4);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_EITHER, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_location(duel, LOCATION_GRAVE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let gy1 = duel.discard_deck(tp, 4, REASON_EFFECT)?;
    let mut og = duel.get_operated_group();
    let gy2 = duel.discard_deck(1 - tp, 4, REASON_EFFECT)?;
    if gy1 == 0 && gy2 == 0 {
        return Ok(());
    }
    og.merge(duel.get_operated_group());
    if !og.is_empty()
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && og.is_exists(duel, aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)), 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = og.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp), 1, 1, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
