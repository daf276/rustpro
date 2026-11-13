// The Arrival Cyberse @Ignister
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 11738489;

// Link Summoned from Cyberse monsters of five Attributes (Leotron, Bururu, Link Apple, Wizard, Link Disciple), with an opponent's monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (47226949, MZONE),
    (42429678, MZONE),
    (7925734, MZONE),
    (3723262, MZONE),
    (32995276, MZONE),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(TOKEN_IGNISTER),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| u64::from(c.get_attribute_full(duel, Some(lc), sumtype, tp)))
    }
}

pub const LISTED_NAMES: &[u32] = &[ID, TOKEN_IGNISTER];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_link_summoned(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_value(duel, c.get_material_count(duel) as i32 * 1000);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.get_owner(duel) != e.get_owner(duel)
}

fn desfilter(c: CardId, duel: &Duel, hc: CardId, tp: u8) -> bool {
    let zone = hc.get_linked_zone_full(duel, tp) & !ZONES_EMZ;
    duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_TOFIELD, zone) > 0
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_IGNISTER, TYPES_TOKEN, 0, 0, 1, RACE_CYBERSE, ATTRIBUTE_DARK)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |tc, duel| desfilter(tc, duel, c, tp),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        ) && duel.is_player_can_special_summon_monster(tp, TOKEN_IGNISTER, token_stats()));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp) & !ZONES_EMZ;
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0
        && duel.is_player_can_special_summon_monster(tp, TOKEN_IGNISTER, token_stats())
        && let Some(token) = duel.create_token(tp, TOKEN_IGNISTER)
    {
        duel.special_summon_full(token, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}
