// Yubel
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 78371393;
const TERROR_INCARNATE: u32 = 4779091;

pub const LISTED_NAMES: &[u32] = &[TERROR_INCARNATE];

// Three on the field, to Tribute each other, with Terror Incarnate in the Deck to Special Summon, and the opponent's Sangans to
// attack them and Dark Hole to remove them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (TERROR_INCARNATE, DECK),
    (TERROR_INCARNATE, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// Being attacked in Attack Position is rare: its End Phase effect Tributes or destroys the copies before the first Battle Phase.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3a.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3a.set_operation(duel, regop);
    c.register_effect(duel, e3a);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_BATTLE_CONFIRM);
    e3.set_condition(duel, damcon);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_RELEASE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_DESTROYED);
    e5.set_condition(duel, |duel, a| a.re != a.e.get_label_object(duel));
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if Some(c) == duel.get_attack_target() && c.is_position(duel, POS_FACEUP_ATTACK) {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE, 0, 1);
    } else {
        c.reset_flag_effect(duel, ID);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.has_flag_effect(duel, ID) && Some(c) == duel.get_attack_target()
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    let atk = attacker.map_or(0, |ac| ac.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, atk);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    if let Some(ac) = attacker
        && ac.is_relate_to_battle(duel)
    {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        let atk = ac.get_attack(duel);
        duel.damage(p, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        exception: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.check_release_group_full(tp, CardId::is_releasable_by_effect, 1, release_options(c, tp)) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_RELEASE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let b1 = duel.check_release_group_full(tp, CardId::is_releasable_by_effect, 1, release_options(c, tp));
    let op = if b1 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (true, aux::stringid(ID, 4))])?
    } else {
        Some(2)
    };
    match op {
        Some(1) => {
            let g = duel.select_release_group_full(tp, CardId::is_releasable_by_effect, (1, 1), false, release_options(c, tp))?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection(g);
            duel.release(g, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.destroy(c, REASON_EFFECT)?;
        }
        _ => {}
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, TERROR_INCARNATE) && c.is_can_be_special_summoned(duel, e, 0, tp, true, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, true, true, POS_FACEUP)? > 0
    {
        sc.complete_procedure(duel);
    }
    Ok(())
}
