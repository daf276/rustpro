// Crystal Clear Wing Synchro Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 59765225;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_CLEAR_WING], Some(sc), sumtype, tp)
    }
}

// On the field with the opponent's Dragite (a monster effect) and Dark Hole and Raigeki, and WIND monsters (Raptite) in the Deck to
// add. Its search needs a Synchro Summon, and no ported Level 3 Synchro Tuner makes Level 10 with Clear Wing Synchro Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (9464441, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (73079836, DECK),
    (73079836, DECK),
    (73079836, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atkcon);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, discon);
    e2.set_target(duel, stock::negate_destroy_target);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel)) && a.rp == 1 - a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    let atk = rc.get_base_attack(duel);
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && rc.is_controler(duel, 1 - tp)
        && c.update_attack(duel, atk, Some(RESETS_STANDARD_PHASE_END), None, None) == atk
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(
            duel,
            EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT,
        );
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, Value::Effect(immval));
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let c = e.get_handler(duel);
    let owner = te.get_owner(duel);
    owner != c && te.is_monster_effect(duel) && te.is_activated(duel) && owner.get_controler(duel) == 1 - c.get_controler(duel)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8) && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_MZONE) && c.is_synchro_summoned(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
