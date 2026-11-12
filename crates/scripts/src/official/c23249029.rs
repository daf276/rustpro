// Cursed Copycat Noble Arms
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 23249029;

// In the hand and Deck, equipped to Sparkman (a Level 4 LIGHT Warrior) to summon Disguise from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (20721928, MZONE),
    (20721928, MZONE),
    (87750925, DECK),
    (87750925, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl EquipProcedure for Procedure {}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_EQUIP);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_value(duel, Value::Card(atkdefval));
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_target(duel)
            .is_some_and(|ec| ec.is_race(duel, RACE_WARRIOR))
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

pub(crate) fn atkdefvalfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && (c.is_faceup(duel) || c.get_equip_target(duel).is_some())
}

fn atkdefval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    200 * duel.get_matching_group_count(
        atkdefvalfilter,
        e.get_handler_player(duel),
        LOCATION_ONFIELD | LOCATION_GRAVE,
        0,
        None,
    ) as i32
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, attr: u32, lv: u32, code: u32, hc: CardId) -> bool {
    c.is_race(duel, RACE_WARRIOR)
        && c.is_attribute(duel, attr)
        && c.is_level(duel, &[lv])
        && !c.is_code(duel, code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && hc.check_equip_target(duel, c)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(ec) = c.get_equip_target(duel) else { return Ok(false) };
    if chk == 0 {
        let (attr, lv, code) = (ec.get_attribute(duel), ec.get_level(duel), ec.get_code(duel));
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |sc, duel| spfilter(sc, duel, e, tp, attr, lv, code, c),
                tp,
                LOCATION_DECK,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(ec.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_WARRIOR));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(ec) = c.get_equip_target(duel) else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let (attr, lv, code) = (ec.get_attribute(duel), ec.get_level(duel), ec.get_code(duel));
    let g = duel.select_matching_card(
        tp,
        |sc, duel| spfilter(sc, duel, e, tp, attr, lv, code, c),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.equip(tp, c, sc)?
        && ec.is_location(duel, LOCATION_MZONE)
    {
        duel.break_effect()?;
        duel.destroy(ec, REASON_EFFECT)?;
    }
    Ok(())
}
