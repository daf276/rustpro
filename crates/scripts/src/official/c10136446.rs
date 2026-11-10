// Darklord Eveningstar
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 10136446;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Darklord Gulgolet and Ixchel. No Darklord Spell or Trap is
// ported, so the Set and the copy of a Graveyard card have nothing to find.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (84031359, HAND),
    (84031359, MZONE),
    (52840267, HAND),
    (52840267, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::pay_lp::<1000>);
    e3.set_target(duel, copytg);
    e3.set_operation(duel, copyop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, sump: u8) -> bool {
    c.is_level_above(duel, 6)
        && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, sump)
        && c.is_race_full(duel, RACE_FAIRY, Some(fc), sumtype, sump)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn rescon(duel: &Duel, sg: CardSet, stzone_chk: bool) -> (bool, bool) {
    (
        sg.len() == 2
            && (stzone_chk || sg.is_exists(duel, CardId::is_field_spell, 1, None))
            && sg.is_exists(duel, CardId::is_spell, 1, None)
            && sg.is_exists(duel, CardId::is_trap, 1, None),
        false,
    )
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk != 0 {
        return Ok(false);
    }
    let tp = a.tp;
    let stzone_chk = duel.get_location_count(tp, LOCATION_SZONE) >= 2;
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, stzone_chk)))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let stzone_chk = duel.get_location_count(tp, LOCATION_SZONE) >= 2;
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, stzone_chk), tp, HINTMSG_SET)?;
    if sg.len() == 2 {
        duel.sset(tp, sg)?;
    }
    Ok(())
}

fn copyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_spell_trap(duel)
        && c.is_able_to_deck(duel)
        && c.check_activate_effect(duel, false, true).is_some()
}

fn copytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && copyfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(copyfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, copyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    Ok(false)
}

fn copyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(te) = tc.check_activate_effect(duel, false, true) else {
        return Ok(());
    };
    let args = Args {
        e: te,
        tp,
        eg: Some(CardSet::default()),
        ep: PLAYER_NONE,
        ev: 0,
        re: Some(e),
        r: REASON_EFFECT,
        rp: PLAYER_NONE,
        ..Args::default()
    };
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(tg) => {
            tg(duel, &args, 1, None)?;
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    duel.break_effect()?;
    tc.create_effect_relation(duel, te);
    duel.break_effect()?;
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    for etc in g.iter() {
        etc.create_effect_relation(duel, te);
    }
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, &args)?;
    }
    tc.release_effect_relation(duel, te);
    for etc in g.iter() {
        etc.release_effect_relation(duel, te);
    }
    duel.break_effect()?;
    duel.send_to_deck(te.get_handler(duel), PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
