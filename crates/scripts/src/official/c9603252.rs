// Witchcrafter Vice-Madame
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 9603252;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_WITCHCRAFTER], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Fusion Summoned by Polymerization with two Witchcrafter Genni (Witchcrafter Spellcasters); Witchcrafter Creation in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (64756282, HAND),
    (64756282, HAND),
    (64756282, DECK),
    (57916305, GRAVE),
    (57916305, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_condition(duel, condition);
    e1.set_target(duel, tg1);
    e1.set_operation(duel, op1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_count_limit(duel, 1, (ID, 2));
    e2.set_target(duel, tg2);
    e2.set_operation(duel, op2);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_count_limit(duel, 1, (ID, 3));
    e3.set_target(duel, tg3);
    e3.set_operation(duel, op3);
    c.register_effect(duel, e3);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let race: u64 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_RACE);
    a.re.is_some_and(|re| {
        re.is_spell_effect(duel) || (!re.is_active_type(duel, TYPE_FUSION) && re.is_monster_effect(duel) && race & RACE_SPELLCASTER > 0)
    })
}

fn tg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_level_below(duel, 6) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn tg3(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn op1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = g.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(dg);
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}

fn op2(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
        POS_FACEUP,
    )
}

fn op3(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
