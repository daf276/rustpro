// Supreme King Z-ARC
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 13331639;

pub const LISTED_SERIES: &[u16] = &[SET_SUPREME_KING_DRAGON];

fn fusfilter<const CARD_TYPE: u32>(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp) && c.is_type_full(duel, CARD_TYPE, Some(fc), sumtype, tp)
}

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(fusfilter::<TYPE_FUSION>),
        FusionMaterial::Filter(fusfilter::<TYPE_SYNCHRO>),
        FusionMaterial::Filter(fusfilter::<TYPE_XYZ>),
        FusionMaterial::Filter(fusfilter::<TYPE_PENDULUM>),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization from a copy on the field (a Dragon Fusion Monster), Stardust Dragon, Dark Rebellion Xyz Dragon and Supreme King
// Dragon Darkwurm; in the Pendulum Zone for the opponent's Pot of Greed to be punished; on the field to battle Sangan and to be destroyed by Dark Hole;
// Supreme King Dragons in the Deck to summon. Random play never attacks with it into a monster it destroys, so that effect is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, PZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND),
    (24094653, HAND),
    (44508094, MZONE),
    (16195942, MZONE),
    (69610326, MZONE),
    (69610326, DECK),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_value(duel, Value::EffectPlayer(limval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_HAND);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.get_current_phase() != PHASE_DRAW);
    e3.set_target(duel, ddtg);
    e3.set_operation(duel, ddop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e5);
    let e6 = e5.clone_effect(duel);
    e6.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e6.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp != e.get_handler_player(duel)));
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 2));
    e7.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_BATTLE_DESTROYING);
    e7.set_condition(duel, aux::bdocon);
    e7.set_target(duel, sptg);
    e7.set_operation(duel, spop);
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_description(duel, aux::stringid(ID, 3));
    e8.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e8.set_code(duel, EVENT_DESTROYED);
    e8.set_property(duel, EFFECT_FLAG_DELAY);
    e8.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
    });
    e8.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e8.set_operation(duel, penop);
    c.register_effect(duel, e8);
}

fn limval(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    rc.is_location(duel, LOCATION_MZONE) && re.is_monster_effect(duel) && rc.is_type(duel, TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ)
}

fn ddfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_previous_location(duel, LOCATION_DECK)
}

fn ddtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| ddfilter(c, duel, tp), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn ddop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| ddfilter(c, duel, tp), None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8) -> bool {
    if c.is_location(duel, LOCATION_DECK) && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, rp, None, c) <= 0 {
        return false;
    }
    c.is_set_card(duel, &[SET_SUPREME_KING_DRAGON]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    if chk == 0 {
        // The Lua tests the undefined global `loc`, which is nil and so differs from 0.
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, rp),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, rp),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.check_pendulum_zones(tp) && c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
