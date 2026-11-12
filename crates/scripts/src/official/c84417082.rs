// Number 91: Thunder Spark Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 84417082;
pub const XYZ_NUMBER: u32 = 91;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters, and monsters on both fields to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (2118022, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn desmonscheck(duel: &Duel, a: &Args) -> bool {
    let exception = CardSet::from(a.e.get_handler(duel));
    duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(exception))
}

fn desallcheck(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_ONFIELD) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let choices: [CostChoice; 2] = [
        (Cost::detach_from_self::<3, 3>, aux::stringid(ID, 1), desmonscheck),
        (Cost::detach_from_self::<5, 5>, aux::stringid(ID, 2), desallcheck),
    ];
    Cost::choice(duel, a, chk, &choices)
}

fn destroyed(duel: &Duel, a: &Args, exception: Option<CardSet>) -> CardSet {
    if Cost::get_choice(duel, a.e) == Some(1) {
        duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, exception)
    } else {
        duel.get_field_group(a.tp, 0, LOCATION_ONFIELD)
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = destroyed(duel, a, Some(a.e.get_handler(duel).into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, a.e).then(|| CardSet::from(c));
    let g = destroyed(duel, a, exc);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
