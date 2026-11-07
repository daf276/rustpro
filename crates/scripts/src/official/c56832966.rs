// Number S39: Utopia the Lightning
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 56832966;

// In the Extra Deck, Xyz Summoned over Number 39: Utopia, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (84013237, MZONE),
    (84013237, MZONE),
    (84013237, MZONE),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, tp, xyzc| {
        c.is_rank(duel, &[4])
            && c.is_set_card_full(duel, &[SET_UTOPIA], Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && c.is_faceup(duel)
    });
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.get_attacker() == Some(c) || duel.get_attack_target() == Some(c)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, atkcon);
    // Cost.SoftOncePerBattle is passed without its flag, so it only returns a function and limits nothing.
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn atkconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_UTOPIA]) && c.is_type(duel, TYPE_XYZ)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel).is_some_and(|bc| bc.is_controler(duel, 1 - a.tp))
        && c.get_overlay_group(duel).is_exists(duel, atkconfilter, 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 5000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL);
        c.register_effect(duel, e1);
    }
    Ok(())
}
