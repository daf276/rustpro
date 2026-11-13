// Blue-Eyes Toon Ultimate Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71808988;
const BLUE_EYES_TOON_DRAGON: u32 = 53183600;

pub const LISTED_NAMES: &[u32] = &[BLUE_EYES_TOON_DRAGON];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(BLUE_EYES_TOON_DRAGON),
        FusionMaterial::Filter(toonmatfilter),
        FusionMaterial::Filter(toonmatfilter),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::Int(0));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(
            CardId::is_able_to_deck_or_extra_as_cost,
            tp,
            LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE,
            0,
            None,
        )
    }

    fn op(duel: &mut Duel, g: CardSet, tp: u8, _: CardId) -> Script<()> {
        let (fu, fd) = g.split(duel, CardId::is_faceup, None);
        if !fu.is_empty() {
            duel.hint_selection(fu);
        }
        if !fd.is_empty() {
            duel.confirm_cards(1 - tp, fd)?;
        }
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// In the Extra Deck, Special Summoned by shuffling Blue-Eyes Toon Dragon and two Toon monsters, on the field with Toon cards in the
// Graveyard to add, my Toon monsters attacked by the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (BLUE_EYES_TOON_DRAGON, HAND),
    (BLUE_EYES_TOON_DRAGON, MZONE),
    (59383041, HAND),
    (59383041, MZONE),
    (65458948, MZONE),
    (65458948, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_TOON));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, rmcon);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn toonmatfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_TOON, Some(fc), sumtype, tp) && c.is_monster(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_TOON]) || c.lists_code_with_archetype(duel, &[SET_TOON])) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_attack_target()
        .is_some_and(|bc| bc.is_controler(duel, tp) && bc.is_type(duel, TYPE_TOON) && bc.is_able_to_remove(duel))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = duel.get_attack_target();
    duel.set_operation_info(0, CATEGORY_REMOVE, bc.map(CardSet::from), 1, a.tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(bc) = duel.get_attack_target()
        && bc.is_relate_to_battle(duel)
    {
        let temp_banish_eff = aux::remove_until::<aux::DefaultFieldReturnOp>(duel, bc, 0, REASON_EFFECT, PHASE_DAMAGE, ID, e, tp)?;
        unported!(duel, temp_banish_eff.is_none());
        if let Some(temp_banish_eff) = temp_banish_eff {
            let e1 = temp_banish_eff.clone_effect(duel);
            e1.set_code(duel, EVENT_DAMAGE_STEP_END);
            duel.register_effect(e1, tp);
            temp_banish_eff.reset(duel);
        }
    }
    Ok(())
}
