// Update Jammer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 88093706;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_above(duel, 2) && c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// Cyberse Normal Monsters (Digitron, Bitron, Leotron) to Link Summon and battle with and as Link Material of Link Spider, and Sangan
// and Elfnote Lucina on the opponent's side to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (98978921, EXTRA),
    (98978921, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
    (47226949, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, matcon);
    e2.set_operation(duel, matop);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let (Some(at), Some(d)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    let ours = if at.is_controler(duel, 1 - a.tp) { d } else { at };
    ours.is_faceup(duel) && ours.is_race(duel, RACE_CYBERSE)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let at = duel.get_attacker();
    let d = duel.get_attack_target();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e1.set_target_card(duel, |duel, e, c| c != e.get_owner(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop);
    e2.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_BATTLE_DESTROYED);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    e4.set_condition(duel, damcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    duel.register_effect(e4, tp);
    for tc in [at, d].into_iter().flatten() {
        if tc.is_relate_to_battle(duel) {
            fix_stats(duel, e, c, tc);
        }
    }
    Ok(())
}

fn fix_stats(duel: &mut Duel, e: EffectId, c: CardId, tc: CardId) {
    let (mut atk, mut def) = (tc.get_text_attack(duel), tc.get_text_defense(duel));
    if tc.is_immune_to_effect(duel, e) {
        (atk, def) = (tc.get_base_attack(duel), tc.get_base_defense(duel));
    }
    for (code, value) in [(EFFECT_SET_ATTACK_FINAL, atk.max(0)), (EFFECT_SET_DEFENSE_FINAL, def.max(0))] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, code);
        e1.set_value(duel, value);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        tc.register_effect_full(duel, e1, true);
    }
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    let loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let ce = duel.get_chain_info::<Option<EffectId>>(ev, CHAININFO_TRIGGERING_EFFECT);
    if loc & LOCATION_ONFIELD == LOCATION_ONFIELD && ce.is_some_and(|ce| ce.get_owner(duel) != a.e.get_owner(duel)) {
        duel.negate_effect(ev);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .get_first()
        .is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE) && tc.is_monster(duel) && !tc.is_previous_controler(duel, a.tp))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(1000);
    Ok(false)
}

fn matcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE)
        && a.r & REASON_LINK == REASON_LINK
        && a.e
            .get_handler(duel)
            .get_reason_card(duel)
            .is_some_and(|sc| sc.is_link_monster(duel))
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(sc) = c.get_reason_card(duel)
        && sc.is_faceup(duel)
        && sc.is_on_field(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}
