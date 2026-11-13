// GranSolfachord Coolia
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 84521924;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp), 1, None)
    }
}

// Copies in the Extra Deck, Link Summoned from Pendulum Monsters (Flash Knight), with FaSolfachord Fancia (Scale 5) in the Pendulum Zone to Special Summon and
// DoSolfachord Cutia (Scale 8) in the Deck to send to the Extra Deck, and the opponent's Ash Blossom to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (17390179, MZONE),
    (17390179, MZONE),
    (17390179, MZONE),
    (54100561, PZONE),
    (55226153, DECK),
    (14558127, MZONE | OPPONENT),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_DISEFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Chain(effval));
    c.register_effect(duel, e2);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2a.set_code(duel, EVENT_CHAINING);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_label_object(duel, Some(e2));
    e2a.set_operation(duel, checkop);
    c.register_effect(duel, e2a);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_NEGATE + CATEGORY_TOEXTRA);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let player = e.get_handler_player(duel);
    duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_PENDULUM)),
        player,
        LOCATION_EXTRA,
        0,
        None,
    ) as i32
        * 100
}

// The Lua keeps a table (fieldid of the effect to fieldid of this card) in e2's label object, replaced at the start of each chain;
// here a per-chain epoch in the script state stands in for the table, with one entry per effect.
const EPOCH: u32 = u32::MAX;

fn entry_key(e2: EffectId, fieldid: u32) -> u64 {
    u64::from(ID) << 48 | (e2.index() as u64 & 0xffff) << 32 | u64::from(fieldid)
}

fn effval(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let Some(te) = duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let epoch = duel.get_script_state(entry_key(e, EPOCH));
    epoch > 0 && duel.get_script_state(entry_key(e, te.get_field_id(duel))) == epoch
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(e2) = a.e.get_label_object(duel) else { return Ok(()) };
    if duel.get_current_chain() == 1 {
        duel.set_script_state(entry_key(e2, EPOCH), duel.get_script_state(entry_key(e2, EPOCH)) + 1);
    }
    if let Some(re) = a.re
        && re.is_active_type(duel, TYPE_PENDULUM)
        && c.get_linked_group(duel).is_contains(re.get_handler(duel))
    {
        duel.set_script_state(entry_key(e2, re.get_field_id(duel)), duel.get_script_state(entry_key(e2, EPOCH)));
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && a.rp == 1 - a.tp && duel.is_chain_negatable(a.ev as u8)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD])
        && c.is_odd_scale(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
        return Ok(
            duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) > 0
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_PZONE, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_PZONE as i32);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tefilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM) && c.is_even_scale(duel) && !c.is_forbidden(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let zone = c.get_linked_zone_full(duel, tp);
    if duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_PZONE, 0, 1, 1, None)?;
    if sg.is_empty() || duel.special_summon_full(sg, 0, tp, tp, false, false, POS_FACEUP, zone)? == 0 || !duel.negate_activation(a.ev as u8)
    {
        return Ok(());
    }
    if let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    let g = duel.get_matching_group(tefilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    let pg = g.select(duel, tp, 1, 1, None)?;
    if !pg.is_empty() {
        duel.break_effect()?;
        duel.send_to_extra_p(pg, tp, REASON_EFFECT)?;
    }
    Ok(())
}
