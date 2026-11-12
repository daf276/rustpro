// Battlin' Boxer Cheat Commissioner
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 76589815;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// Copies and another Boxer Xyz Monster that trigger each other by attacking, Kojikocy (a Normal Monster) forced to attack, and Spells in the opponent's hand to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (59627393, MZONE),
    (59627393, MZONE),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e3.set_condition(duel, atcon);
    e3.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e3);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, cfcon);
    e3.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e3.set_target(duel, cftg);
    e3.set_operation(duel, cfop);
    c.register_effect(duel, e3);
}

fn atfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BATTLIN_BOXER])
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(atfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn cfcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let boxer_of_mine = |tc: CardId| tc.is_controler(duel, tp) && tc != c && tc.is_set_card(duel, &[SET_BATTLIN_BOXER]);
    duel.get_attacker().is_some_and(boxer_of_mine) || duel.get_attack_target().is_some_and(|at| at.is_faceup(duel) && boxer_of_mine(at))
}

fn cftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0 && duel.get_location_count(tp, LOCATION_SZONE) > 0)
}

fn cffilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_ssetable(duel)
}

fn cfop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    if duel.get_location_count(tp, LOCATION_SZONE) > 0 {
        let sg = g.filter(duel, cffilter, None);
        if !sg.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let setg = sg.select(duel, tp, 1, 1, None)?;
            if let Some(tc) = setg.get_first() {
                duel.sset(tp, tc)?;
            }
        }
    }
    duel.shuffle_hand(1 - tp)?;
    Ok(())
}
