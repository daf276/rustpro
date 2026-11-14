// Steelswarm Origin
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 61888819;

pub const LISTED_SERIES: &[u16] = &[SET_LSWARM];

// 0x60 for your Extra Monster Zones, 0x60 << 16 for your opponent's.
const ALL_EMZ: u32 = 0x600060;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_LSWARM], Some(lc), sumtype, tp)
    }
}

// Steelswarm Sting to Link Summon with and to Special Summon from the Deck, a Synchro Monster for it to destroy, and the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (35618486, MZONE),
    (35618486, MZONE),
    (35618486, MZONE),
    (35618486, DECK),
    (35618486, DECK),
    (35618486, DECK),
    (60800381, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_FORCE_MZONE);
    e1.set_range(duel, LOCATION_EMZONE);
    e1.set_target_range(duel, LOCATION_EXTRA, LOCATION_EXTRA);
    e1.set_value(
        duel,
        Value::CardZones(|duel, e, _, _, rp, _| (e.get_handler(duel).get_linked_zone_full(duel, rp) | ALL_EMZ) as i32),
    );
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, indcon);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    c.register_effect(duel, e4);
    let e5 = e3.clone_effect(duel);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_DESTROYED);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e6.set_count_limit(duel, 1, 0);
    e6.set_condition(duel, spcon);
    e6.set_target(duel, sptg);
    e6.set_operation(duel, spop);
    c.register_effect(duel, e6);
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_linked_group(duel)
        .filter_count(duel, CardId::is_monster, None)
        > 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && c.is_previous_location(duel, LOCATION_MZONE)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, cfilter, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_LSWARM])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let ct = a.eg.unwrap_or_default().filter_count(duel, cfilter, None) as i32;
    let mut ft = ft.min(ct);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, ft as u8, None)?;
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}
