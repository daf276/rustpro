// Mysterion the Dragon Crown
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 13735899;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)),
    ];
}

// On the field and in the Extra Deck with Polymerization, Thousand-Eyes Idol and Petit Dragon to Fusion Summon it, and Crystron
// Rosenix on both sides, whose Graveyard effect Special Summons a token, with Crystron Tuners its field effect summons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (24094653, HAND),
    (27125110, HAND),
    (75356564, HAND),
    (55326322, MZONE),
    (55326322, GRAVE),
    (93665266, DECK),
    (55326322, MZONE | OPPONENT),
    (55326322, GRAVE | OPPONENT),
    (20050865, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| duel.get_field_group_count(e.get_handler_player(duel), LOCATION_REMOVED, 0) as i32 * -100),
    );
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2a.set_code(duel, EVENT_CUSTOM + ID);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit(duel, 1, ID);
    e2a.set_condition(duel, |duel, _| !duel.is_phase(PHASE_DAMAGE));
    e2a.set_target(duel, rmtg);
    e2a.set_operation(duel, rmop);
    c.register_effect(duel, e2a);
    e2a.set_label_object_group(duel, Some(CardSet::default()));
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_label_object(duel, Some(e2a));
    e2b.set_operation(duel, regop);
    c.register_effect(duel, e2b);
}

fn tgfilter(c: CardId, duel: &Duel, rc: CardId) -> bool {
    c.is_location(duel, LOCATION_MZONE)
        && (rc == c || (c.is_faceup(duel) && rc.is_original_race(duel, c.get_original_race(duel))))
        && c.is_able_to_remove(duel)
}

fn tgfilter2(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.is_able_to_remove(duel) && c.is_can_be_effect_target(duel, e)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| tgfilter2(c, duel, e), None);
    // The Lua's chkc check calls s.tgfilter with a nil rc, which errors on a face-up card and fails on a face-down one.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tc = if g.len() == 1 {
        g
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        g.select(duel, tp, 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tc), 1, tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let race = tc.get_original_race(duel);
        let mut g = duel.get_matching_group(
            aux::faceup_filter(move |c, duel| c.is_original_race(duel, race)),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            Some(tc.into()),
        );
        g.insert(tc);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if eg.is_contains(c) {
        return Ok(());
    }
    unported!(duel, a.re.is_none());
    let Some(re) = a.re else { return Ok(()) };
    if !(re.is_activated(duel) && re.is_monster_effect(duel)) {
        return Ok(());
    }
    let rc = re.get_handler(duel);
    let tg = eg.filter(duel, |tc, duel| tgfilter(tc, duel, rc), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e2a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e2a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e2a.set_label_object_group(duel, Some(g));
    duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}
