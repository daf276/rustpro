// Port of utility.lua and cards_specific_functions.lua: the Auxiliary and Cost helpers and the Card, Effect and Duel
// methods they add.
use engine::{Args, CardId, CardSet, CostFn, CostReplaceCall, CostReplaceCondition, Duel, EffectId, Operation, Script, Value};

use crate::chain::{DuelChain, EffectChain};
use crate::constant::*;
use crate::proc_fusion::Fusion;
use crate::proc_workaround::{CardWorkaround, DuelWorkaround};

// The chain data slots of cd.cost_detached_materials and cd.cost_discarded_cards, above those card ports use.
pub const COST_DETACHED_MATERIALS: u8 = u8::MAX;
const COST_DISCARDED_CARDS: u8 = u8::MAX - 1;
const COST_LP_PAID: u8 = u8::MAX - 2;
const COST_CHOICE: u8 = u8::MAX - 3;

// Cost.Choice's {cost, description, additional_check}.
pub type CostChoice = (CostFn, u64, fn(&Duel, &Args) -> bool);

pub mod aux {
    use engine::{Args, CardId, CardSet, Duel, EffectId, ExtraMaterialCall, Operation, Script, Value};

    use crate::chain::DuelChain;
    use crate::constant::*;
    use crate::proc_workaround::CardWorkaround;
    use crate::utility::CardUtility;

    pub const TRUE: fn(CardId, &Duel) -> bool = |_, _| true;

    // The group check (sg, mg, c) of SelectUnselectGroup, whose e and tp are captures; returns (res, stop).
    pub type GroupCheck<'a> = &'a dyn Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool);

    // The emt table of aux.GetExtraMaterials: per EFFECT_EXTRA_MATERIAL effect the extra materials it offers.
    pub type ExtraMaterials = Vec<(CardSet, EffectId)>;

    pub const fn stringid(code: u32, id: u64) -> u64 {
        (id & 0xfffff) | ((code as u64) << 20)
    }

    // s.global_check per script, as script state keyed by the script's id.
    pub fn global_check(duel: &mut Duel, id: u32, func: impl FnOnce(&mut Duel)) {
        let key = (1 << 61) | u64::from(id);
        if duel.get_script_state(key) == 0 {
            duel.set_script_state(key, 1);
            func(duel);
        }
    }

    // aux.AddValuesReset: the first call creates the turn end effect that runs every function registered.
    pub fn add_values_reset(duel: &mut Duel, resetfunc: fn(&mut Duel)) {
        if duel.add_values_reset(resetfunc) {
            let ge = duel.global_effect();
            ge.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            ge.set_code(duel, EVENT_TURN_END);
            ge.set_count_limit(duel, 1, 0);
            ge.set_condition_mut(duel, values_reset);
            duel.register_effect(ge, 0);
        }
    }

    pub fn values_reset(duel: &mut Duel, _: &Args) -> bool {
        for resetfunc in duel.get_values_reset() {
            resetfunc(duel);
        }
        false
    }

    // aux.GetAttributeStrings: the system string (1010 for EARTH to 1016 for DIVINE) of each attribute in v, lowest first.
    pub fn get_attribute_strings(v: u32) -> impl Iterator<Item = u64> {
        (0..7u64).filter(move |bit| v & 1 << bit != 0).map(|bit| 1010 + bit)
    }

    pub fn stat_change_damage_step_condition(duel: &Duel, _: &Args) -> bool {
        !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated())
    }

    pub fn exccon(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        duel.get_turn_count() != c.get_turn_id(duel) || c.is_reason(duel, REASON_RETURN)
    }

    pub fn faceup_filter(filter: impl Fn(CardId, &Duel) -> bool) -> impl Fn(CardId, &Duel) -> bool {
        move |c, duel| c.is_faceup(duel) && filter(c, duel)
    }

    pub fn lnklimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, sum_type: u32, _: u8, _: u8) -> bool {
        sum_type & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
    }

    pub fn fuslimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, sum_type: u32, _: u8, _: u8) -> bool {
        sum_type & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
    }

    pub fn ritlimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, sum_type: u32, _: u8, _: u8) -> bool {
        sum_type & SUMMON_TYPE_RITUAL == SUMMON_TYPE_RITUAL
    }

    pub fn synlimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, sum_type: u32, _: u8, _: u8) -> bool {
        sum_type & SUMMON_TYPE_SYNCHRO == SUMMON_TYPE_SYNCHRO
    }

    // Auxiliary.EvilHeroLimit; a nil se fails the Lua's `se:GetHandler()`.
    pub fn evil_hero_limit(duel: &Duel, e: EffectId, se: Option<EffectId>, _: u8, sum_type: u32, _: u8, _: u8) -> bool {
        let Some(handler) = se.map(|se| se.get_handler(duel)) else {
            return false;
        };
        let tp = e.get_handler_player(duel);
        let affected = |code| duel.is_player_affected_by_effect(tp, code).is_some();
        handler.is_code(duel, CARD_DARK_FUSION)
            || (affected(SKILL_DARK_UNITY) && handler.is_code(duel, CARD_SUPER_POLYMERIZATION))
            || (affected(EFFECT_SUPREME_CASTLE) && sum_type & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION)
    }

    // Auxiliary.FossilLimit; a nil se fails the Lua's `se:GetHandler()` for a card in the Extra Deck.
    pub fn fossil_limit(duel: &Duel, e: EffectId, se: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
        !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
            || se.is_some_and(|se| se.get_handler(duel).is_code(duel, CARD_FOSSIL_FUSION))
    }

    pub fn nvfilter(c: CardId, duel: &Duel) -> bool {
        c.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_none()
    }

    // aux.ReleaseCheckMMZ, a check of Duel.CheckReleaseGroupCost.
    pub fn release_check_mmz(duel: &Duel, sg: CardSet, tp: u8, _: CardSet) -> (bool, bool) {
        (duel.get_mzone_count(tp, Some(sg)) > 0, false)
    }

    // aux.ReleaseCheckTarget with its dg as a capture, a check of Duel.CheckReleaseGroupCost.
    pub fn release_check_target(dg: CardSet) -> impl Fn(&Duel, CardSet, u8, CardSet) -> (bool, bool) {
        move |duel, sg, _, _| (dg.is_exists(duel, TRUE, 1, Some(sg)), false)
    }

    // The required and filter parameters of Auxiliary.AddLavaProcedure; a nil filter is Card.IsMonster.
    pub trait LavaProcedure {
        const REQUIRED: usize;

        fn filter(c: CardId, duel: &Duel) -> bool {
            c.is_monster(duel)
        }
    }

    pub fn add_lava_procedure<P: LavaProcedure>(
        duel: &mut Duel,
        c: CardId,
        position: u32,
        value: i32,
        description: Option<u64>,
    ) -> EffectId {
        let e1 = duel.create_effect(c);
        if let Some(description) = description {
            e1.set_description(duel, description);
        }
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
        e1.set_range(duel, LOCATION_HAND);
        e1.set_target_range(duel, position, 1);
        e1.set_value(duel, value);
        e1.set_condition(duel, lava_condition::<P>);
        e1.set_target(duel, lava_target::<P>);
        e1.set_operation(duel, lava_operation);
        c.register_effect(duel, e1);
        e1
    }

    fn lava_check(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
        (
            duel.get_mzone_count_full(1 - tp, Some(sg), tp, LOCATION_REASON_TOFIELD, 0xff) > 0,
            false,
        )
    }

    fn lava_materials<P: LavaProcedure>(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(|c, duel| c.is_releasable(duel) && P::filter(c, duel), tp, 0, LOCATION_MZONE, None)
    }

    fn lava_condition<P: LavaProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = c.get_controler(duel);
        let required = P::REQUIRED.max(1);
        select_unselect_group_check(duel, lava_materials::<P>(duel, tp), required, required, &|duel, sg, _, _| {
            lava_check(duel, sg, tp)
        })
    }

    fn lava_target<P: LavaProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let tp = a.tp;
        let required = P::REQUIRED.max(1);
        let mg = lava_materials::<P>(duel, tp);
        let g = select_unselect_group_full(
            duel,
            mg,
            required,
            required,
            &|duel, sg, _, _| lava_check(duel, sg, tp),
            tp,
            HINTMSG_RELEASE,
            None,
            true,
        )?;
        if g.is_empty() {
            return Ok(false);
        }
        a.e.set_label_object_group(duel, Some(g));
        Ok(true)
    }

    fn lava_operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let g = a.e.get_label_object_group(duel).unwrap_or_default();
        duel.release(g, REASON_COST)?;
        Ok(())
    }

    struct Kaiju;

    impl LavaProcedure for Kaiju {
        const REQUIRED: usize = 1;
    }

    // Auxiliary.AddKaijuProcedure.
    pub fn add_kaiju_procedure(duel: &mut Duel, c: CardId) -> (EffectId, EffectId) {
        c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_set_card(duel, &[SET_KAIJU]), LOCATION_MZONE);
        let e1 = add_lava_procedure::<Kaiju>(duel, c, POS_FACEUP_ATTACK, 0, None);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_SPSUMMON_PROC);
        e2.set_range(duel, LOCATION_HAND);
        e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
        e2.set_target_range(duel, POS_FACEUP_ATTACK, 0);
        e2.set_condition(duel, |duel, a| {
            let Some(c) = a.c else { return true };
            let tp = c.get_controler(duel);
            duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(
                    |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_KAIJU]),
                    tp,
                    0,
                    LOCATION_MZONE,
                    1,
                    None,
                )
        });
        c.register_effect(duel, e2);
        (e1, e2)
    }

    // The cost most Kaiju monsters share: Duel.RemoveCounter of `COUNT` Kaiju Counters from either field.
    pub fn kaiju_counter_cost<const COUNT: u16>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(duel.is_can_remove_counter(a.tp, 1, 1, COUNTER_KAIJU, COUNT, REASON_COST));
        }
        duel.remove_counter(a.tp, 1, 1, COUNTER_KAIJU, COUNT, REASON_COST)?;
        Ok(false)
    }

    // The extracat, extrainfo and extraop parameters of Auxiliary.EnableNeosReturn; returneff has no caller.
    pub trait NeosReturn {
        const EXTRACAT: u64 = 0;

        fn extrainfo(_duel: &mut Duel, _a: &Args) {}

        fn extraop(_duel: &mut Duel, _a: &Args) -> Script<()> {
            Ok(())
        }
    }

    pub fn enable_neos_return<P: NeosReturn>(duel: &mut Duel, c: CardId) {
        let e1 = duel.create_effect(c);
        e1.set_category(duel, CATEGORY_TODECK | P::EXTRACAT);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_has_effect(duel, 42015635).is_none());
        e1.set_target(duel, neos_return_target::<P>);
        e1.set_operation(duel, neos_return_operation::<P>);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e2.set_property(duel, 0);
        e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_has_effect(duel, 42015635).is_some());
        c.register_effect(duel, e2);
    }

    fn neos_return_target<P: NeosReturn>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_TODECK, Some(a.e.get_handler(duel).into()), 1, 0, 0);
        P::extrainfo(duel, a);
        Ok(false)
    }

    fn neos_return_substitute_filter(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, 14088859) && c.is_able_to_remove_as_cost(duel)
    }

    fn neos_return_operation<P: NeosReturn>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
            return Ok(());
        }
        let sc = duel.get_first_matching_card(necro_valley_filter(neos_return_substitute_filter), tp, LOCATION_GRAVE, 0, None);
        match sc {
            Some(sc) if duel.select_yes_no(tp, stringid(14088859, 0))? => {
                duel.remove(sc, POS_FACEUP, REASON_COST)?;
            }
            _ => {
                duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        if c.is_location(duel, LOCATION_EXTRA) {
            P::extraop(duel, a)?;
        }
        Ok(())
    }

    // Auxiliary.GetMMZonesPointedTo(player) with the default filter, locations and target player.
    pub fn get_mm_zones_pointed_to(duel: &Duel, player: u8) -> u32 {
        let links = duel.get_matching_group(
            |c, duel| c.is_faceup(duel) && c.is_type(duel, TYPE_LINK),
            player,
            LOCATION_MZONE,
            LOCATION_MZONE,
            None,
        );
        links.get_bitwise_or(duel, |c, duel| u64::from(c.get_linked_zone_full(duel, player))) as u32 & ZONES_MMZ
    }

    pub fn necro_valley_filter(filter: impl Fn(CardId, &Duel) -> bool) -> impl Fn(CardId, &Duel) -> bool {
        move |c, duel| filter(c, duel) && !(c.is_has_effect(duel, EFFECT_NECRO_VALLEY).is_some() && duel.is_chain_disablable(0))
    }

    pub fn sp_elim_filter(c: CardId, duel: &Duel, mustbefaceup: bool, includemzone: bool) -> bool {
        if !c.is_monster(duel) {
            return includemzone || c.is_location(duel, LOCATION_GRAVE);
        }
        if mustbefaceup && c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel) {
            return false;
        }
        let spirit_elimination = duel.is_player_affected_by_effect(c.get_controler(duel), 69832741).is_some();
        if includemzone {
            return c.is_location(duel, LOCATION_MZONE) || !spirit_elimination;
        }
        c.is_location(duel, if spirit_elimination { LOCATION_MZONE } else { LOCATION_GRAVE })
    }

    // Auxiliary.WelcomeLabrynthTrapDestroyOperation: Labrynth Labyrinth, the only source of the effect, is not ported.
    pub fn welcome_labrynth_trap_destroy_operation(duel: &mut Duel, tp: u8) {
        unported!(duel, duel.is_player_affected_by_effect(tp, CARD_LABRYNTH_LABYRINTH).is_some());
    }

    pub fn ice_barrier_discard_filter(c: CardId, duel: &Duel, tp: u8) -> bool {
        c.is_has_effect_full(duel, EFFECT_ICEBARRIER_REPLACE, tp).is_some() && c.is_able_to_remove_as_cost(duel)
    }

    fn ice_barrier_cost_filter(c: CardId, duel: &Duel, f: impl Fn(CardId, &Duel) -> bool, discard: bool) -> bool {
        f(c, duel)
            && if discard {
                c.is_discardable(duel)
            } else {
                c.is_able_to_grave_as_cost(duel)
            }
    }

    // The chk 0 half of Auxiliary.IceBarrierDiscardCost(f, discard, minc, maxc).
    pub fn ice_barrier_discard_cost_check(duel: &Duel, tp: u8, f: impl Fn(CardId, &Duel) -> bool, discard: bool, minc: usize) -> bool {
        duel.is_existing_matching_card(
            |c, duel| ice_barrier_cost_filter(c, duel, &f, discard),
            tp,
            LOCATION_HAND,
            0,
            minc,
            None,
        ) || duel.is_existing_matching_card(|c, duel| ice_barrier_discard_filter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None)
    }

    // The cost half of Auxiliary.IceBarrierDiscardCost(f, discard, minc, maxc): the number of cards sent or banished.
    pub fn ice_barrier_discard_cost(
        duel: &mut Duel,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        discard: bool,
        (minc, maxc): (usize, usize),
    ) -> Script<i32> {
        let mut g = duel.get_matching_group(|c, duel| ice_barrier_cost_filter(c, duel, &f, discard), tp, LOCATION_HAND, 0, None);
        g.merge(duel.get_matching_group(|c, duel| ice_barrier_discard_filter(c, duel, tp), tp, LOCATION_GRAVE, 0, None));
        let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
            (
                sg.filter_count(duel, |c, duel| ice_barrier_discard_filter(c, duel, tp), None) <= 1 && sg.len() >= minc,
                false,
            )
        };
        let mut sg = select_unselect_group(duel, g, minc, maxc, &rescon, tp, stringid(CARD_REVEALER_ICEBARRIER, 1))?;
        let mut removed = 0;
        if sg.is_exists(
            duel,
            |c, duel| c.is_has_effect_full(duel, EFFECT_ICEBARRIER_REPLACE, tp).is_some(),
            1,
            None,
        ) {
            let te = sg.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_ICEBARRIER_REPLACE).is_some(), None);
            let replace = te
                .get_first()
                .and_then(|c| c.get_card_effect(duel, EFFECT_ICEBARRIER_REPLACE).first().copied());
            if let Some(replace) = replace {
                replace.use_count_limit(duel, tp);
            }
            removed = duel.remove(te, POS_FACEUP, REASON_COST)?;
            sg = sg.difference(te);
        }
        if sg.is_empty() {
            return Ok(removed);
        }
        Ok(duel.send_to_grave(sg, REASON_COST | if discard { REASON_DISCARD } else { 0 })? + removed)
    }

    // Auxiliary.zptgroup: `filter` is the Lua filter with its player argument, aux::TRUE for nil.
    pub fn zptgroup(duel: &Duel, eg: CardSet, filter: impl Fn(CardId, &Duel) -> bool, c: CardId) -> CardSet {
        let fil = eg.filter(duel, filter, None);
        fil.intersection(c.get_linked_group(duel))
            .union(eg.filter(duel, |ec, duel| zptfilter(ec, duel, c), None))
    }

    // Auxiliary.zptgroupcon, which aux.zptcon(filter) calls with the effect's handler.
    pub fn zptgroupcon(duel: &Duel, eg: CardSet, filter: impl Fn(CardId, &Duel) -> bool, c: CardId) -> bool {
        !eg.filter(duel, filter, None).intersection(c.get_linked_group(duel)).is_empty()
            || eg.is_exists(duel, |ec, duel| zptfilter(ec, duel, c), 1, None)
    }

    fn zptfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
        !c.is_location(duel, LOCATION_MZONE)
            && ec.get_linked_zone_full(duel, c.get_previous_controler(duel)) & (1 << c.get_previous_sequence(duel)) != 0
    }

    pub fn register_client_hint(
        duel: &mut Duel,
        card: CardId,
        property: u32,
        tp: u8,
        player1: u32,
        player2: u32,
        description: Option<u64>,
    ) -> EffectId {
        register_client_hint_full(duel, card, property, tp, player1, player2, description, 0, 0)
    }

    // `description` None takes the card's second string.
    pub fn register_client_hint_full(
        duel: &mut Duel,
        card: CardId,
        property: u32,
        tp: u8,
        player1: u32,
        player2: u32,
        description: Option<u64>,
        reset: u32,
        reset_count: u16,
    ) -> EffectId {
        let eff = duel.create_effect(card);
        eff.set_property(duel, (EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT) | property);
        eff.set_target_range(duel, player1, player2);
        let description = description.unwrap_or_else(|| stringid(card.get_original_code(duel), 1));
        eff.set_description(duel, description);
        eff.set_reset_full(duel, (RESET_PHASE + PHASE_END) | reset, reset_count);
        duel.register_effect(eff, tp);
        eff
    }

    // Auxiliary.chainreg: the flag effect Spell Counter cards read to count a Spell activated in the chain.
    pub fn chainreg(duel: &mut Duel, a: &Args) -> Script<()> {
        let c = a.e.get_handler(duel);
        if c.get_flag_effect(duel, 1) == 0 {
            c.register_flag_effect(duel, 1, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET | RESET_CHAIN, 0, 1);
        }
        Ok(())
    }

    // Auxiliary.sumreg: the flag effect (code = the effect's label) for a summoned card of that original code, kept until the
    // end of the turn.
    pub fn sumreg(duel: &mut Duel, a: &Args) -> Script<()> {
        let code = a.e.get_label(duel) as u32;
        for tc in a.eg.unwrap_or_default() {
            if tc.get_original_code(duel) == code {
                tc.register_flag_effect(duel, code, RESETS_STANDARD_PHASE_END, 0, 1);
            }
        }
        Ok(())
    }

    // Auxiliary.ToHandOrElse for one card: to the hand, or else `oper` where `check` allows it (`str` its option).
    pub fn to_hand_or_else(
        duel: &mut Duel,
        card: CardId,
        player: u8,
        check: impl Fn(CardId, &Duel) -> bool,
        oper: impl FnOnce(&mut Duel) -> Script<()>,
        str: u64,
    ) -> Script<()> {
        to_hand_or_else_result(duel, card, player, check, |duel| oper(duel).map(|()| true), str).map(drop)
    }

    // Auxiliary.ToHandOrElse for a caller that reads its result: the to-hand option is a Lua number, always truthy, and
    // the other option is what `oper` returns.
    pub fn to_hand_or_else_result(
        duel: &mut Duel,
        card: CardId,
        player: u8,
        check: impl Fn(CardId, &Duel) -> bool,
        oper: impl FnOnce(&mut Duel) -> Script<bool>,
        str: u64,
    ) -> Script<bool> {
        let b1 = card.is_able_to_hand(duel);
        let b2 = check(card, duel);
        let opt = if b1 && b2 {
            duel.select_option(player, &[573, str])?
        } else if b1 {
            duel.select_option(player, &[573])?
        } else {
            duel.select_option(player, &[str])? + 1
        };
        if opt != 0 {
            return oper(duel);
        }
        if duel.send_to_hand(card, PLAYER_NONE, REASON_EFFECT)? > 0 {
            let og = duel
                .get_operated_group()
                .filter(duel, |c, duel| c.is_previous_location(duel, LOCATION_DECK), None);
            if !og.is_empty() {
                duel.confirm_cards(1 - player, og)?;
            }
        }
        Ok(true)
    }

    fn select_unselect_loop(duel: &Duel, c: CardId, sg: CardSet, mg: CardSet, minc: usize, maxc: usize, rescon: GroupCheck) -> bool {
        if sg.len() >= maxc {
            return false;
        }
        let mut sg = sg;
        sg.insert(c);
        let (mut res, stop) = rescon(duel, sg, mg, Some(c));
        if stop {
            return false;
        }
        if sg.len() < minc || (sg.len() < maxc && !res) {
            res = mg.is_exists(
                duel,
                |other, duel| select_unselect_loop(duel, other, sg, mg, minc, maxc, rescon),
                1,
                Some(sg),
            );
        }
        res
    }

    // Auxiliary.SelectUnselectGroup with chk=0: whether `rescon` accepts some group of `minc` to `maxc` cards of g.
    pub fn select_unselect_group_check(duel: &Duel, g: CardSet, minc: usize, maxc: usize, rescon: GroupCheck) -> bool {
        if g.len() < minc {
            return false;
        }
        let mut eg = g;
        for c in g {
            if select_unselect_loop(duel, c, CardSet::default(), eg, minc, maxc, rescon) {
                return true;
            }
            eg.remove(c);
        }
        false
    }

    // Group.CheckDifferentPropertyBinary: whether the cards can each take a different bit of the property f returns.
    pub fn check_different_property_binary(duel: &Duel, g: CardSet, f: &dyn Fn(CardId, &Duel) -> u64) -> bool {
        fn checkrecbin(duel: &Duel, c: CardId, g: CardSet, val: u64, f: &dyn Fn(CardId, &Duel) -> u64) -> bool {
            let prop = f(c, duel) & !val;
            let rest = g.difference(c.into());
            (0..u64::BITS)
                .map(|bit| 1 << bit)
                .filter(|bit| prop & bit != 0)
                .any(|bit| g.len() < 2 || rest.into_iter().any(|other| checkrecbin(duel, other, rest, val | bit, f)))
        }
        g.len() < 2 || g.into_iter().any(|c| checkrecbin(duel, c, g, 0, f))
    }

    // Group.CheckSameProperty: whether the cards share a bit of the property f returns, and the shared bits.
    pub fn check_same_property(duel: &Duel, g: CardSet, f: &dyn Fn(CardId, &Duel) -> u64) -> (bool, u64) {
        let mut prop: Option<u64> = None;
        for tc in g {
            let value = prop.map_or_else(|| f(tc, duel), |prop| prop & f(tc, duel));
            if value == 0 {
                return (false, 0);
            }
            prop = Some(value);
        }
        (true, prop.unwrap_or(0))
    }

    // Group.CheckDifferentProperty: whether the cards can each take a different one of the properties f returns.
    pub fn check_different_property(duel: &Duel, g: CardSet, f: &dyn Fn(CardId, &Duel) -> Vec<u32>) -> bool {
        fn checkrec(duel: &Duel, c: CardId, g: CardSet, taken: &mut Vec<u32>, f: &dyn Fn(CardId, &Duel) -> Vec<u32>) -> bool {
            for prop in f(c, duel) {
                if taken.contains(&prop) {
                    continue;
                }
                taken.push(prop);
                let rest = g.difference(c.into());
                if g.len() < 2 || rest.into_iter().any(|other| checkrec(duel, other, rest, taken, f)) {
                    return true;
                }
                taken.retain(|&other| other != prop);
            }
            false
        }
        g.len() < 2 || g.into_iter().any(|c| checkrec(duel, c, g, &mut Vec::new(), f))
    }

    // Auxiliary.PropertyTableFilter: the values f returns that are among `values`. The Lua also caches them per card.
    pub fn property_table_filter<'a>(
        f: impl Fn(CardId, &Duel) -> Vec<u32> + 'a,
        values: &'a [u32],
    ) -> impl Fn(CardId, &Duel) -> Vec<u32> + 'a {
        move |c, duel| f(c, duel).into_iter().filter(|value| values.contains(value)).collect()
    }

    pub fn select_unselect_group(
        duel: &mut Duel,
        g: CardSet,
        minc: usize,
        maxc: usize,
        rescon: GroupCheck,
        seltp: u8,
        hintmsg: u64,
    ) -> Script<CardSet> {
        select_unselect_group_full(duel, g, minc, maxc, rescon, seltp, hintmsg, None, false)
    }

    // Auxiliary.SelectUnselectGroup with chk=1: `seltp` picks or drops one card at a time.
    pub fn select_unselect_group_full(
        duel: &mut Duel,
        g: CardSet,
        minc: usize,
        maxc: usize,
        rescon: GroupCheck,
        seltp: u8,
        hintmsg: u64,
        finishcon: Option<GroupCheck>,
        cancelable: bool,
    ) -> Script<CardSet> {
        select_unselect_group_break(duel, g, minc, maxc, rescon, seltp, hintmsg, finishcon, None, cancelable)
    }

    // The Lua's `breakcon` ends the selection as soon as it accepts the cards selected so far.
    pub fn select_unselect_group_break(
        duel: &mut Duel,
        g: CardSet,
        minc: usize,
        maxc: usize,
        rescon: GroupCheck,
        seltp: u8,
        hintmsg: u64,
        finishcon: Option<GroupCheck>,
        breakcon: Option<GroupCheck>,
        cancelable: bool,
    ) -> Script<CardSet> {
        let mut sg = CardSet::default();
        loop {
            let finishable = sg.len() >= minc && finishcon.is_none_or(|finishcon| finishcon(duel, sg, g, None).0);
            let mg = g.filter(duel, |c, duel| select_unselect_loop(duel, c, sg, g, minc, maxc, rescon), Some(sg));
            if breakcon.is_some_and(|breakcon| breakcon(duel, sg, mg, None).0) || mg.is_empty() || sg.len() >= maxc {
                break;
            }
            duel.hint(HINT_SELECTMSG, seltp, hintmsg);
            let cancel = finishable || (cancelable && sg.is_empty());
            let Some(tc) = mg.select_unselect(duel, Some(sg), seltp, finishable, cancel, minc as u8, maxc as u8)? else {
                break;
            };
            if sg.is_contains(tc) {
                sg.remove(tc);
            } else {
                sg.insert(tc);
            }
        }
        Ok(sg)
    }

    // Clock Lizard's check, which nothing ported reads; the effects are registered all the same.
    fn create_lizard_check(duel: &mut Duel, c: CardId, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, CARD_CLOCK_LIZARD);
        e1.set_target_range(duel, 0xff, 0);
        e1.set_target_card(duel, filter);
        e1.set_value(duel, 1);
        e1
    }

    // Auxiliary.DoubleSnareValidity(c, range), a marker effect that nothing ported reads.
    pub fn double_snare_validity(duel: &mut Duel, c: CardId, range: u32) {
        let eff = duel.create_effect(c);
        eff.set_type(duel, EFFECT_TYPE_SINGLE);
        eff.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_SINGLE_RANGE);
        eff.set_range(duel, range);
        eff.set_code(duel, 3682106);
        c.register_effect(duel, eff);
    }

    pub fn add_er_equip_limit(duel: &mut Duel, c: CardId, con: Option<engine::ConditionFn>, linkedeff: EffectId) {
        add_er_equip_limit_full(duel, c, con, linkedeff, None, None, None);
    }

    // The equipval and equipop of aux.AddEREquipLimit are left out: only c78063197, which is not ported, reads them.
    pub fn add_er_equip_limit_full(
        duel: &mut Duel,
        c: CardId,
        con: Option<engine::ConditionFn>,
        linkedeff: EffectId,
        prop: Option<u32>,
        resetflag: Option<u32>,
        resetcount: Option<u16>,
    ) {
        let finalprop = EFFECT_FLAG_CANNOT_DISABLE | prop.unwrap_or(0);
        let reset = |duel: &mut Duel, effect: EffectId| match (resetflag, resetcount) {
            (Some(resetflag), Some(resetcount)) => effect.set_reset_full(duel, resetflag, resetcount),
            (Some(resetflag), None) => effect.set_reset(duel, resetflag),
            _ => {}
        };
        let e1 = duel.create_effect(c);
        if let Some(con) = con {
            e1.set_condition(duel, con);
        }
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property_full(duel, finalprop, EFFECT_FLAG2_MAJESTIC_MUST_COPY);
        e1.set_code(duel, 89785779);
        e1.set_label_object(duel, Some(linkedeff));
        reset(duel, e1);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property_full(duel, finalprop & !EFFECT_FLAG_CANNOT_DISABLE, EFFECT_FLAG2_MAJESTIC_MUST_COPY);
        e2.set_code(duel, 89785779 + EFFECT_EQUIP_LIMIT);
        reset(duel, e2);
        c.register_effect(duel, e2);
        linkedeff.set_label_object(duel, Some(e2));
    }

    pub fn add_lizard_check(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, CARD_CLOCK_LIZARD);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_value(duel, 1);
        c.register_effect(duel, e1);
        e1
    }

    pub fn add_temp_lizard_check(duel: &mut Duel, c: CardId, tp: u8) -> EffectId {
        add_temp_lizard_check_full(duel, c, tp, |_, _, _| true)
    }

    pub fn add_temp_lizard_check_full(duel: &mut Duel, c: CardId, tp: u8, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = create_lizard_check(duel, c, filter);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        e1
    }

    pub fn create_continuous_lizard_check(
        duel: &mut Duel,
        c: CardId,
        location: u32,
        filter: fn(&Duel, EffectId, CardId) -> bool,
    ) -> EffectId {
        let e1 = create_lizard_check(duel, c, filter);
        e1.set_range(duel, location);
        e1
    }

    pub fn add_continuous_lizard_check(duel: &mut Duel, c: CardId, location: u32, filter: fn(&Duel, EffectId, CardId) -> bool) -> EffectId {
        let e1 = create_continuous_lizard_check(duel, c, location, filter);
        c.register_effect(duel, e1);
        e1
    }

    // The oper of DelayedOperation and RemoveUntil, as a type because effect operations cannot capture it.
    pub trait DelayedOperation {
        fn operate(duel: &mut Duel, rg: CardSet, a: &Args) -> Script<()>;

        // The Lua's cond(ag, e, ...); the effect's first label is the fid delayed_operation_full made.
        fn condition(_duel: &Duel, _ag: CardSet, _a: &Args) -> bool {
            true
        }
    }

    // delayed_operation_id, the global counter of utility.lua.
    const DELAYED_OPERATION_ID: u64 = 3;

    // The delayed effect keeps the flag beside the fid in its label, where the Lua closures capture it.
    fn affected_group(duel: &Duel, e: EffectId) -> CardSet {
        let &[fid, flag] = e.get_labels(duel) else {
            return CardSet::default();
        };
        let g = e.get_label_object_group(duel).unwrap_or_default();
        g.filter(duel, |c, duel| c.get_flag_effect_label(duel, flag as u32) == Some(fid), None)
    }

    pub fn delayed_operation<O: DelayedOperation>(
        duel: &mut Duel,
        g: CardSet,
        phase: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
    ) -> Option<EffectId> {
        delayed_operation_reset::<O>(duel, g, phase, RESET_PHASE | phase, flag, e, tp)
    }

    // `hint` is the Lua's hint argument, the string shown on the affected cards.
    pub fn delayed_operation_hint<O: DelayedOperation>(
        duel: &mut Duel,
        g: CardSet,
        phase: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
        hint: u64,
    ) -> Option<EffectId> {
        delayed_operation_full::<O>(duel, g, phase, RESET_PHASE | phase, flag, e, tp, hint, 1)
    }

    // `reset` is the Lua's reset argument; 0 is not nil there, so the effect never resets.
    pub fn delayed_operation_reset<O: DelayedOperation>(
        duel: &mut Duel,
        g: CardSet,
        phase: u32,
        reset: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
    ) -> Option<EffectId> {
        delayed_operation_full::<O>(duel, g, phase, reset, flag, e, tp, 0, 1)
    }

    // `reset_count` is the Lua's.
    pub fn delayed_operation_full<O: DelayedOperation>(
        duel: &mut Duel,
        g: CardSet,
        phase: u32,
        reset: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
        hint: u64,
        reset_count: u16,
    ) -> Option<EffectId> {
        if g.is_empty() {
            return None;
        }
        let fid = duel.get_script_state(DELAYED_OPERATION_ID) + 1;
        duel.set_script_state(DELAYED_OPERATION_ID, fid);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_code(duel, EVENT_PHASE | phase);
        e1.set_reset_full(duel, reset, reset_count);
        e1.set_count_limit(duel, 1, 0);
        e1.set_labels(duel, &[fid, flag.into()]);
        e1.set_label_object_group(duel, Some(g));
        e1.set_condition(duel, |duel, a| {
            let ag = affected_group(duel, a.e);
            !ag.is_empty() && O::condition(duel, ag, a)
        });
        e1.set_operation(duel, |duel, a| O::operate(duel, affected_group(duel, a.e), a));
        duel.register_effect(e1, tp);
        for tc in g {
            let property = if hint != 0 { EFFECT_FLAG_CLIENT_HINT } else { 0 };
            let flag_effect = tc.register_flag_effect_full(duel, flag, RESET_EVENT + RESETS_STANDARD, property, 1, fid, hint);
            flag_effect.set_label_object(duel, Some(e1));
            flag_effect.set_condition(duel, |duel, a| !a.e.get_label_object(duel).is_some_and(|e1| e1.is_deleted(duel)));
        }
        Some(e1)
    }

    pub fn remove_until<O: DelayedOperation>(
        duel: &mut Duel,
        g: impl Into<CardSet>,
        position: u32,
        reason: u32,
        phase: u32,
        flag: u32,
        e: EffectId,
        tp: u8,
    ) -> Script<Option<EffectId>> {
        let g = g.into();
        if g.is_empty() || duel.remove(g, position, reason | REASON_TEMPORARY)? == 0 {
            return Ok(None);
        }
        let g = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), None);
        Ok(if g.is_empty() {
            None
        } else {
            delayed_operation::<O>(duel, g, phase, flag, e, tp)
        })
    }

    // Auxiliary.IsZone: whether the card's zone, as seen from tp, is in `zone`.
    pub fn is_zone(c: CardId, duel: &Duel, zone: u32, tp: u8) -> bool {
        let sequence = c.get_sequence(duel);
        let own = c.is_controler(duel, tp);
        let mut rzone = if own { 1 << sequence } else { 1 << (16 + sequence) };
        if c.is_sequence(duel, 5) || c.is_sequence(duel, 6) {
            rzone |= if own { 1 << (16 + 11 - sequence) } else { 1 << (11 - sequence) };
        }
        rzone & zone > 0
    }

    // The lowest limit of the player's Summon Gate effects, which come with the cards that register them.
    pub fn check_summon_gate(duel: &Duel, tp: u8) -> Option<i32> {
        if !duel.get_player_effect(tp, CARD_SUMMON_GATE).is_empty() {
            duel.reach_unported(concat!(file!(), ":", line!()));
        }
        None
    }

    // Auxiliary.GetMustBeMaterialGroup: the monsters EFFECT_MUST_BE_MATERIAL effects of the player force on a summon;
    // those effects come with the cards that register them.
    pub fn get_must_be_material_group(
        duel: &Duel,
        tp: u8,
        _eg: Option<CardSet>,
        _sump: u8,
        _sc: Option<CardId>,
        _g: Option<CardSet>,
        _r: u32,
    ) -> CardSet {
        unported!(
            duel,
            duel.may_have_effect(EFFECT_MUST_BE_MATERIAL) && !duel.get_player_effect(tp, EFFECT_MUST_BE_MATERIAL).is_empty()
        );
        CardSet::default()
    }

    // Auxiliary.GetExtraMaterials: per EFFECT_EXTRA_MATERIAL effect whose count limit allows a use, the materials its
    // value offers (chk 0, which may register effects), and all of them together.
    pub fn get_extra_materials(duel: &mut Duel, tp: u8, mustg: CardSet, sc: CardId, summon_type: u32) -> (ExtraMaterials, CardSet) {
        let (mut t, mut tg) = (ExtraMaterials::new(), CardSet::default());
        for te in duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL) {
            if te.check_count_limit(duel, tp) {
                let Value::ExtraMaterial(value) = te.get_value(duel) else {
                    continue;
                };
                let eg = value(duel, te, ExtraMaterialCall::Get { summon_type, tp, sc }).difference(mustg);
                tg = tg.union(eg);
                t.push((eg, te));
            }
        }
        (t, tg)
    }

    // efun(c, e, tp, sg, mg, lc, og, chk) of an extra material entry; a missing operation is aux.TRUE.
    pub fn extra_material_filter(
        duel: &Duel,
        (eg, te): (CardSet, EffectId),
        c: CardId,
        tp: u8,
        sg: CardSet,
        mg: CardSet,
        lc: CardId,
        chk: i32,
    ) -> bool {
        match te.get_operation(duel) {
            Some(Operation::ExtraMaterial(efun)) => efun(duel, c, te, tp, sg, mg, lc, eg, chk),
            _ => true,
        }
    }

    // Auxiliary.CheckValidExtra: whether an extra material entry takes c, adding every such entry to `filt`.
    pub fn check_valid_extra(
        duel: &Duel,
        c: CardId,
        tp: u8,
        sg: CardSet,
        mg: CardSet,
        lc: CardId,
        emt: &ExtraMaterials,
        filt: &mut Vec<usize>,
    ) -> bool {
        let mut res = false;
        for (index, &ex) in emt.iter().enumerate() {
            if ex.0.is_contains(c) && extra_material_filter(duel, ex, c, tp, sg, mg, lc, 0) {
                res = true;
                filt.push(index);
            }
        }
        res
    }

    // Auxiliary.DeleteExtraMaterialGroups: value chk 2 of every entry, which resets what chk 0 registered.
    pub fn delete_extra_material_groups(duel: &mut Duel, emt: &ExtraMaterials) {
        for &(eg, te) in emt {
            if let Value::ExtraMaterial(value) = te.get_value(duel) {
                value(duel, te, ExtraMaterialCall::Delete { eg });
            }
        }
    }

    const KEPT_EXTRA_MATERIALS: u64 = 1 << 60;

    // Link.Target keeps {sg, filters, emt} as the label object of e for Link.Operation: the filters (indices into emt,
    // at most 15) packed into the script state, each entry's group in the script groups.
    pub fn keep_extra_materials(duel: &mut Duel, e: EffectId, emt: &ExtraMaterials, filt: &[usize]) {
        let key = KEPT_EXTRA_MATERIALS | (e.index() as u64) << 16;
        for (index, &(eg, _)) in emt.iter().enumerate() {
            duel.set_script_group(key | index as u64, Some(eg));
        }
        duel.set_script_state(key, filt.iter().rev().fold(0, |packed, &index| packed << 4 | (index as i64 + 1)));
    }

    // What keep_extra_materials kept. The effects are looked up again, in the state the target left.
    pub fn kept_extra_materials(duel: &Duel, e: EffectId, tp: u8) -> (ExtraMaterials, Vec<usize>) {
        let key = KEPT_EXTRA_MATERIALS | (e.index() as u64) << 16;
        let effects = duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL);
        let usable = effects.iter().filter(|te| te.check_count_limit(duel, tp));
        let emt = usable
            .enumerate()
            .map(|(index, &te)| (duel.get_script_group(key | index as u64).unwrap_or_default(), te))
            .collect();
        let packed = std::iter::successors(Some(duel.get_script_state(key)), |packed| Some(packed >> 4));
        (
            emt,
            packed
                .take_while(|&packed| packed != 0)
                .map(|packed| (packed & 0xf) as usize - 1)
                .collect(),
        )
    }

    // Auxiliary.dpcheck(fun): a SelectUnselectGroup check that the selected cards all differ in `fun`.
    pub fn dpcheck(fun: impl Fn(CardId, &Duel) -> i64) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
        move |duel, sg, _, _| {
            let (c1, c2) = (sg.get_class_count(duel, &fun), sg.len());
            (c1 == c2, c1 != c2)
        }
    }

    // Auxiliary.ChkfMMZ, with the tp SelectUnselectGroup passes as a capture.
    pub fn chkf_mmz(tp: u8, sumcount: i32) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
        move |duel, sg, _, _| (duel.get_mzone_count(tp, Some(sg)) >= sumcount, false)
    }

    pub fn dncheck(duel: &Duel, sg: CardSet, mg: CardSet, c: Option<CardId>) -> (bool, bool) {
        dpcheck(|c, duel| i64::from(c.get_code(duel)))(duel, sg, mg, c)
    }

    // Auxiliary.bdcon: the condition of EVENT_BATTLE_DESTROYING.
    pub fn bdcon(duel: &Duel, a: &Args) -> bool {
        a.e.get_handler(duel).is_relate_to_battle(duel)
    }

    // Auxiliary.bdocon: the condition of EVENT_BATTLE_DESTROYING against an opponent's monster.
    pub fn bdocon(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        c.is_relate_to_battle(duel) && c.is_status(duel, STATUS_OPPO_BATTLE)
    }

    // Auxiliary.bdgcon: the condition of EVENT_BATTLE_DESTROYING against a monster that went to the Graveyard.
    pub fn bdgcon(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        c.is_relate_to_battle(duel)
            && c.get_battle_target(duel)
                .is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
    }

    // Auxiliary.bdogcon: bdocon and bdgcon together.
    pub fn bdogcon(duel: &Duel, a: &Args) -> bool {
        bdocon(duel, a) && bdgcon(duel, a)
    }

    // Auxiliary.ChangeBattleDamage(PLAYER, VALUE): -1 leaves the damage alone.
    pub fn change_battle_damage<const PLAYER: u8, const VALUE: u32>(duel: &Duel, e: EffectId, damp: u8, _: CardId) -> i64 {
        let target = if PLAYER == 0 { damp } else { 1 - damp };
        if e.get_owner_player(duel) == target { i64::from(VALUE) } else { -1 }
    }

    // Auxiliary.imval1: EFFECT_CANNOT_BE_BATTLE_TARGET unless the attacker is immune to the effect.
    pub fn imval1(duel: &Duel, e: EffectId, c: CardId) -> i32 {
        i32::from(!c.is_immune_to_effect(duel, e))
    }

    // Auxiliary.imval2: imval1 for the opponent's attackers.
    pub fn imval2(duel: &Duel, e: EffectId, c: CardId) -> i32 {
        i32::from(imval1(duel, e, c) != 0 && c.get_controler(duel) != e.get_handler_player(duel))
    }

    // Auxiliary.indoval: EFFECT_INDESTRUCTABLE_EFFECT for the opponent's effects.
    pub fn indoval(duel: &Duel, e: EffectId, _re: EffectId, rp: u8) -> bool {
        rp == 1 - e.get_handler_player(duel)
    }

    // Auxiliary.tgoval: EFFECT_CANNOT_BE_EFFECT_TARGET for the opponent's effects.
    pub fn tgoval(duel: &Duel, e: EffectId, _re: EffectId, rp: u8) -> bool {
        rp != e.get_handler_player(duel)
    }

    // Auxiliary.RemainFieldCost: the card stays on the field after the chain, unless its activation is negated.
    pub fn remain_field_cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let (c, tp) = (a.e.get_handler(duel), a.tp);
        let cid: i64 = duel.get_chain_info(0, CHAININFO_CHAIN_ID);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_REMAIN_FIELD);
        e1.set_property(duel, EFFECT_FLAG_OATH);
        e1.set_reset(duel, RESET_CHAIN);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_CHAIN_DISABLED);
        e2.set_operation(duel, remain_field_disabled);
        e2.set_label(duel, cid);
        e2.set_reset(duel, RESET_CHAIN);
        duel.register_effect(e2, tp);
        Ok(false)
    }

    // Auxiliary.RemainFieldDisabled
    fn remain_field_disabled(duel: &mut Duel, a: &Args) -> Script<()> {
        let cid: i64 = duel.get_chain_info(a.ev as u8, CHAININFO_CHAIN_ID);
        if cid != a.e.get_label(duel) {
            return Ok(());
        }
        let owner = a.e.get_owner(duel);
        if owner.is_location(duel, LOCATION_ONFIELD) {
            owner.cancel_to_grave_full(duel, false);
        }
        Ok(())
    }

    // Auxiliary.CheckStealEquip(c, e, tp).
    pub fn check_steal_equip(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
        if c.is_facedown(duel) || !c.is_controler_can_be_changed(duel) || !c.is_controler(duel, 1 - tp) {
            return false;
        }
        if e.get_handler(duel).is_location(duel, LOCATION_SZONE) {
            return true;
        }
        if !duel.is_duel_type(DUEL_TRAP_MONSTERS_NOT_USE_ZONE) && c.is_type(duel, TYPE_TRAPMONSTER) {
            return duel.get_location_count_full(tp, LOCATION_SZONE, tp, LOCATION_REASON_CONTROL, 0xff) > 0
                && duel.get_location_count_full(tp, LOCATION_SZONE, tp, 0, 0xff) >= 2;
        }
        true
    }

    // Auxiliary.cannotmatfilter(...) applied: whether `sumtype` is one of the summon types in `allowed`.
    pub fn cannotmatfilter(allowed: &[u64], sumtype: u64) -> bool {
        let tot = allowed.iter().fold(0, |tot, &value| tot | value);
        let sum = tot & sumtype;
        allowed.contains(&sum)
    }

    fn select_field_return_cards(duel: &mut Duel, p: u8, g: CardSet) -> Script<()> {
        if g.is_empty() {
            return Ok(());
        }
        let mut g = g;
        let ft = duel.get_location_count(p, LOCATION_MZONE);
        if ft > 0 && g.len() > ft as usize {
            duel.hint(HINT_SELECTMSG, p, HINTMSG_TOFIELD);
            let tg = g.select(duel, p, ft as u8, ft as u8, None)?;
            for tc in tg {
                duel.return_to_field(tc)?;
            }
            g = g.difference(tg);
        }
        for tc in g {
            duel.return_to_field(tc)?;
        }
        Ok(())
    }

    // Auxiliary.DefaultFieldReturnOp: the banished cards return to the monster zones, the turn player's first.
    pub struct DefaultFieldReturnOp;

    impl DelayedOperation for DefaultFieldReturnOp {
        fn operate(duel: &mut Duel, rg: CardSet, _: &Args) -> Script<()> {
            if rg.is_empty() {
                return Ok(());
            }
            let turn_p = duel.get_turn_player();
            let g0 = rg.filter(duel, |c, duel| c.is_previous_controler(duel, turn_p), None);
            let g1 = rg.difference(g0);
            select_field_return_cards(duel, turn_p, g0)?;
            select_field_return_cards(duel, 1 - turn_p, g1)
        }
    }

    // ReincarnationChecked, the upvalue guarding the global effect.
    const REINCARNATION_CHECKED: u64 = 1 << 59;

    // Auxiliary.EnableCheckReincarnation: the first call registers the granting effect for every Extra Deck monster.
    pub fn enable_check_reincarnation(duel: &mut Duel) {
        if duel.get_script_state(REINCARNATION_CHECKED) != 0 {
            return;
        }
        duel.set_script_state(REINCARNATION_CHECKED, 1);
        let e1 = duel.global_effect();
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_MATERIAL_CHECK);
        e1.set_value(duel, Value::MaterialCheck(reincarnation_check_value));
        let ge1 = duel.global_effect();
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
        ge1.set_label_object(duel, Some(e1));
        ge1.set_target_range(duel, 0xff, 0xff);
        ge1.set_target_card(duel, |duel, _, c| {
            c.is_type(duel, TYPE_FUSION | TYPE_RITUAL | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK)
        });
        duel.register_effect(ge1, 0);
    }

    // Auxiliary.ReincarnationRitualFilter.
    fn reincarnation_ritual_filter(c: CardId, duel: &Duel, rc: CardId, id: u32, tp: u8) -> bool {
        c.is_summon_code(duel, Some(rc), SUMMON_TYPE_RITUAL.into(), tp, &[id])
            && c.is_controler(duel, tp)
            && c.is_location(duel, LOCATION_MZONE)
    }

    // Auxiliary.ReincarnationCheckValue.
    fn reincarnation_check_value(duel: &mut Duel, _: EffectId, c: CardId) {
        let g = c.get_material(duel);
        let id = c.get_code(duel);
        let tp = c.get_controler(duel);
        let summoned_with_same_code =
            |duel: &Duel, sumtype: u32| g.is_exists(duel, |m, duel| m.is_summon_code(duel, Some(c), sumtype.into(), tp, &[id]), 1, None);
        let mut label = 3;
        let rc = if c.is_link_monster(duel) {
            summoned_with_same_code(duel, SUMMON_TYPE_LINK)
        } else if c.is_type(duel, TYPE_FUSION) {
            summoned_with_same_code(duel, SUMMON_TYPE_FUSION)
        } else if c.is_type(duel, TYPE_RITUAL) {
            label = 0;
            if g.is_exists(duel, |m, duel| reincarnation_ritual_filter(m, duel, c, id, 0), 1, None) {
                label = 1;
            }
            if g.is_exists(duel, |m, duel| reincarnation_ritual_filter(m, duel, c, id, 1), 1, None) {
                label += 2;
            }
            label != 0
        } else if c.is_type(duel, TYPE_SYNCHRO) {
            summoned_with_same_code(duel, SUMMON_TYPE_SYNCHRO)
        } else if c.is_type(duel, TYPE_XYZ) {
            summoned_with_same_code(duel, SUMMON_TYPE_XYZ)
        } else {
            false
        };
        if rc {
            let reset = RESET_EVENT + RESETS_STANDARD - RESET_TOFIELD;
            c.register_flag_effect_full(duel, CARD_SALAMANGREAT_SANCTUARY, reset, 0, 1, label, 0);
        }
    }
}

pub struct Cost;

impl Cost {
    pub fn hint_selected_effect(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.hint(HINT_OPSELECTED, 1 - args.tp, args.e.get_description(duel));
        Ok(false)
    }

    pub fn self_reveal(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(!c.is_public(duel));
        }
        duel.confirm_cards(1 - args.tp, c)?;
        Ok(false)
    }

    pub fn self_discard(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_discardable(duel));
        }
        duel.send_to_grave(c, REASON_DISCARD | REASON_COST)?;
        Ok(false)
    }

    pub fn self_discard_to_grave(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel));
        }
        duel.send_to_grave(c, REASON_DISCARD | REASON_COST)?;
        Ok(false)
    }

    pub fn self_to_grave(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_grave_as_cost(duel));
        }
        duel.send_to_grave(c, REASON_COST)?;
        Ok(false)
    }

    pub fn self_banish(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_remove_as_cost(duel));
        }
        duel.remove(c, POS_FACEUP, REASON_COST)?;
        Ok(false)
    }

    pub fn self_to_hand(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_hand_as_cost(duel));
        }
        duel.send_to_hand(c, PLAYER_NONE, REASON_COST)?;
        Ok(false)
    }

    pub fn self_to_deck(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_deck_as_cost(duel));
        }
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
        Ok(false)
    }

    pub fn self_to_extra(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_able_to_extra_as_cost(duel));
        }
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
        Ok(false)
    }

    // Cost.Replaceable(base) applied: the base cost, unless one of the player's EFFECT_COST_REPLACE effects stands
    // in for it.
    pub fn replaceable(duel: &mut Duel, args: &Args, chk: i32, base: CostFn) -> Script<bool> {
        Cost::replaceable_full(duel, args, chk, base, |_, _, _| true)
    }

    // Cost.Replaceable(base, extracon) applied.
    pub fn replaceable_full(duel: &mut Duel, args: &Args, chk: i32, base: CostFn, extracon: CostReplaceCondition) -> Script<bool> {
        let base_chk = base(duel, args, 0)?;
        if chk == 0 {
            return Ok(base_chk || !Cost::usable_cost_replace_effects(duel, args, chk, extracon).is_empty());
        }
        let Some(eff) = Cost::cost_replace_effect_to_apply(duel, base_chk, args, chk, extracon)? else {
            return base(duel, args, 1);
        };
        duel.hint(HINT_CARD, 0, eff.get_handler(duel).get_original_code(duel).into());
        if let Some(Operation::Event(operation)) = eff.get_operation(duel) {
            operation(duel, &Args { e: eff, ..Args::default() })?;
        }
        eff.use_count_limit(duel, args.tp);
        Ok(false)
    }

    // check_cost_replace_effect of the player's EFFECT_COST_REPLACE effects.
    fn usable_cost_replace_effects(duel: &Duel, args: &Args, chk: i32, extracon: CostReplaceCondition) -> Vec<EffectId> {
        duel.get_player_effect(args.tp, EFFECT_COST_REPLACE)
            .into_iter()
            .filter(|&eff| {
                eff.check_count_limit(duel, args.tp)
                    && match eff.get_value(duel) {
                        Value::Int(value) => value == 1,
                        Value::CostReplace(value) => value(duel, eff, CostReplaceCall { extracon, args, chk }),
                        _ => false,
                    }
            })
            .collect()
    }

    // get_cost_replace_effect_to_apply: None where the base cost is used.
    fn cost_replace_effect_to_apply(
        duel: &mut Duel,
        base_chk: bool,
        args: &Args,
        chk: i32,
        extracon: CostReplaceCondition,
    ) -> Script<Option<EffectId>> {
        let tp = args.tp;
        let effs = Cost::usable_cost_replace_effects(duel, args, chk, extracon);
        let handlers = effs.iter().fold(CardSet::default(), |g, eff| g.union(eff.get_handler(duel).into()));
        let select_effect = |duel: &Duel, handler: CardId| {
            let own: Vec<EffectId> = effs.iter().copied().filter(|eff| eff.get_handler(duel) == handler).collect();
            unported!(duel, own.len() > 1);
            own.first().copied()
        };
        let Some(first) = handlers.first() else { return Ok(None) };
        if handlers.len() == 1 {
            if base_chk && !duel.select_effect_yes_no(tp, first)? {
                return Ok(None);
            }
            return Ok(select_effect(duel, first));
        }
        loop {
            if base_chk && !duel.select_yes_no(tp, 98)? {
                return Ok(None);
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RESOLVEEFFECT);
            if let Some(handler) = handlers.select_full(duel, tp, 1, 1, base_chk, None)?.first() {
                return Ok(select_effect(duel, handler));
            }
        }
    }

    pub fn self_tribute(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_releasable(duel));
        }
        duel.release(c, REASON_COST)?;
        Ok(false)
    }

    // Cost.Discard() applied: one card from the hand; the discarded cards are kept as the chain data's
    // cost_discarded_cards.
    pub fn discard(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::discard_full(duel, args, chk, aux::TRUE, false, (1, 1))
    }

    // Cost.Discard(filter, other, min, max) applied, with the counts given.
    pub fn discard_full(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        filter: impl Fn(CardId, &Duel) -> bool,
        other: bool,
        (min, max): (u8, u8),
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let exclude = other.then(|| CardSet::from(e.get_handler(duel)));
        let filter_final = |c: CardId, duel: &Duel| filter(c, duel) && c.is_discardable(duel);
        if chk == 0 {
            return Ok(min > 0
                && max >= min
                && duel.is_existing_matching_card(filter_final, tp, LOCATION_HAND, 0, usize::from(min), exclude));
        }
        duel.discard_hand_full(tp, filter_final, min, max, REASON_COST | REASON_DISCARD, exclude)?;
        let discarded = duel.get_operated_group();
        e.set_chain_data_group(duel, COST_DISCARDED_CARDS, discarded);
        Ok(false)
    }

    // Cost.DetachFromSelf(min, max) applied; the detached materials are kept as the chain data's
    // cost_detached_materials.
    pub fn detach_from_self<const MIN: u16, const MAX: u16>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::detach_from_self_op::<MIN, MAX>(duel, args, chk, |_, _, _| {})
    }

    // Cost.DetachFromSelf(min, max, op): `op(e, og)` runs on the detached materials.
    pub fn detach_from_self_op<const MIN: u16, const MAX: u16>(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        op: fn(&mut Duel, EffectId, CardSet),
    ) -> Script<bool> {
        Cost::detach_from_self_with(duel, args, chk, MIN, MAX, op)
    }

    // Cost.DetachFromSelf(min, max, op) for a min and max that are functions in the Lua, already called.
    pub fn detach_from_self_with(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        min: u16,
        max: u16,
        op: fn(&mut Duel, EffectId, CardSet),
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let c = e.get_handler(duel);
        if chk == 0 {
            return Ok(min > 0 && max >= min && c.check_remove_overlay_card(duel, tp, min, REASON_COST));
        }
        if c.remove_overlay_card(duel, tp, min, max, REASON_COST)? > 0 {
            let detached = duel.get_operated_group();
            e.set_chain_data_group(duel, COST_DETACHED_MATERIALS, detached);
            op(duel, e, detached);
        }
        Ok(false)
    }

    // Cost.DetachChoiceFromSelf(choices) applied, for choices that are a function in the Lua, already called: one of the
    // amounts the card can detach is announced and detached, as the chain data's cost_detached_materials.
    pub fn detach_choice_from_self(duel: &mut Duel, args: &Args, chk: i32, choices: &[i64]) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let c = e.get_handler(duel);
        let amounts: Vec<u64> = choices
            .iter()
            .filter(|&&ct| c.check_remove_overlay_card(duel, tp, ct as u16, REASON_COST))
            .map(|&ct| ct as u64)
            .collect();
        if chk == 0 {
            return Ok(!amounts.is_empty());
        }
        let amt = match amounts.as_slice() {
            [amt] => *amt,
            [] => return Ok(false),
            _ => duel.announce_number(tp, &amounts)?,
        };
        if c.remove_overlay_card(duel, tp, amt as u16, amt as u16, REASON_COST)? > 0 {
            let detached = duel.get_operated_group();
            e.set_chain_data_group(duel, COST_DETACHED_MATERIALS, detached);
        }
        Ok(false)
    }

    // Cost.PayLP(LP) applied for an LP of at least 1; the LP paid is kept as the chain data's cost_lp_paid.
    pub fn pay_lp<const LP: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(duel.check_lp_cost(args.tp, LP));
        }
        duel.pay_lp_cost(args.tp, LP)?;
        args.e.set_chain_data(duel, COST_LP_PAID, i64::from(LP));
        Ok(false)
    }

    // Cost.PayLP(1/2) applied: half of the LP, rounded down, is paid.
    pub fn pay_lp_half(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let paid = (duel.get_lp(args.tp) / 2) as u32;
        duel.pay_lp_cost(args.tp, paid)?;
        args.e.set_chain_data(duel, COST_LP_PAID, i64::from(paid));
        Ok(false)
    }

    // Cost.SoftOncePerChain(flag) applied: the handler is flagged until the chain ends.
    pub fn soft_once_per_chain<const FLAG: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::soft_once::<FLAG, RESET_CHAIN>(duel, args, chk)
    }

    // Cost.SoftOncePerBattle(flag) applied: the handler is flagged until the end of the Damage Step.
    pub fn soft_once_per_battle<const FLAG: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::soft_once::<FLAG, { RESET_PHASE | PHASE_DAMAGE }>(duel, args, chk)
    }

    // Cost.HardOncePerChain(flag) applied: the player is flagged until the chain ends.
    pub fn hard_once_per_chain<const FLAG: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(!duel.has_flag_effect(args.tp, FLAG));
        }
        duel.register_flag_effect(args.tp, FLAG, RESET_CHAIN, 0, 1);
        Ok(false)
    }

    fn soft_once<const FLAG: u32, const RESET: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(!c.has_flag_effect(duel, FLAG));
        }
        c.register_flag_effect(duel, FLAG, RESET_EVENT | RESETS_STANDARD | RESET, 0, 1);
        Ok(false)
    }

    // Cost.AND(...) applied. A cost that returns false when executed, which stops the others, has no caller.
    pub fn and(duel: &mut Duel, args: &Args, chk: i32, costs: &[CostFn]) -> Script<bool> {
        for cost in costs {
            if !cost(duel, args, chk)? && chk == 0 {
                return Ok(false);
            }
        }
        Ok(chk == 0)
    }

    // Cost.Choice(...) applied to {cost, description, additional_check}; the chosen index (from 1) is kept as the chain
    // data's cost_choice.
    pub fn choice(duel: &mut Duel, args: &Args, chk: i32, choices: &[CostChoice]) -> Script<bool> {
        let mut ops = Vec::with_capacity(choices.len());
        for &(cost, description, additional_check) in choices {
            let check = cost(duel, args, 0)? && additional_check(duel, args);
            ops.push((check, description));
        }
        if chk == 0 {
            return Ok(ops.iter().any(|&(check, _)| check));
        }
        let Some(choice) = duel.select_effect(args.tp, &ops)? else {
            return Ok(false);
        };
        args.e.set_chain_data(duel, COST_CHOICE, choice as i64);
        if let Some(&(cost, ..)) = choice.checked_sub(1).and_then(|index| choices.get(index)) {
            cost(duel, args, 1)?;
        }
        Ok(false)
    }

    pub fn get_choice(duel: &Duel, e: EffectId) -> Option<i64> {
        e.get_chain_data(duel, COST_CHOICE)
    }

    // Cost.Reveal(filter, other, min, max, op, location) applied, with the counts given.
    pub fn reveal(
        duel: &mut Duel,
        args: &Args,
        chk: i32,
        filter: fn(CardId, &Duel, EffectId, u8) -> bool,
        other: bool,
        (min, max): (u8, u8),
        op: Option<fn(&mut Duel, &Args, CardSet)>,
        location: u32,
    ) -> Script<bool> {
        let (e, tp) = (args.e, args.tp);
        let exclude = other.then(|| CardSet::from(e.get_handler(duel)));
        let filter_final = |c: CardId, duel: &Duel| filter(c, duel, e, tp) && !c.is_public(duel);
        if chk == 0 {
            return Ok(min > 0 && max >= min && duel.is_existing_matching_card(filter_final, tp, location, 0, usize::from(min), exclude));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, filter_final, tp, location, 0, min, max, exclude)?;
        duel.confirm_cards(1 - tp, g)?;
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None) {
            duel.shuffle_hand(tp)?;
        }
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
            duel.shuffle_deck(tp)?;
        }
        if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), 1, None) {
            duel.shuffle_extra(tp)?;
        }
        if let Some(op) = op {
            op(duel, args, g);
        }
        Ok(false)
    }

    // Cost.RemoveCounterFromSelf(counter_type, count) applied.
    pub fn remove_counter_from_self<const COUNTER_TYPE: u16, const COUNT: u16>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_can_remove_counter(duel, args.tp, COUNTER_TYPE, COUNT, REASON_COST));
        }
        c.remove_counter(duel, args.tp, COUNTER_TYPE, COUNT, REASON_COST)?;
        Ok(false)
    }

    // Cost.RemoveCounterFromField(counter_type, count) applied.
    pub fn remove_counter_from_field<const COUNTER_TYPE: u16, const COUNT: u16>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(duel.is_can_remove_counter(args.tp, 1, 0, COUNTER_TYPE, COUNT, REASON_COST));
        }
        duel.remove_counter(args.tp, 1, 0, COUNTER_TYPE, COUNT, REASON_COST)?;
        Ok(false)
    }
}

pub trait CardUtility: Copy {
    fn is_monster(self, duel: &Duel) -> bool;
    fn is_spell(self, duel: &Duel) -> bool;
    fn is_trap(self, duel: &Duel) -> bool;
    fn is_equip_card(self, duel: &Duel) -> bool;
    fn is_spell_trap(self, duel: &Duel) -> bool;
    fn is_monster_card(self, duel: &Duel) -> bool;
    fn is_trap_monster(self, duel: &Duel) -> bool;
    fn has_non_zero_attack(self, duel: &Duel) -> bool;
    fn has_non_zero_defense(self, duel: &Duel) -> bool;
    fn is_owner(self, duel: &Duel, player: u8) -> bool;
    fn is_equip_spell(self, duel: &Duel) -> bool;
    fn is_previous_attribute_on_field(self, duel: &Duel, attribute: u32) -> bool;
    fn is_previous_type_on_field(self, duel: &Duel, card_type: u32) -> bool;
    fn is_previous_code_on_field(self, duel: &Duel, codes: &[u32]) -> bool;
    fn is_field_spell(self, duel: &Duel) -> bool;
    fn is_ritual_spell(self, duel: &Duel) -> bool;
    fn is_ritual_monster(self, duel: &Duel) -> bool;
    fn is_normal_spell(self, duel: &Duel) -> bool;
    fn is_normal_trap(self, duel: &Duel) -> bool;
    fn is_normal_summoned(self, duel: &Duel) -> bool;
    fn is_normal_spell_trap(self, duel: &Duel) -> bool;
    fn is_continuous_spell(self, duel: &Duel) -> bool;
    fn is_continuous_trap(self, duel: &Duel) -> bool;
    fn is_continuous_spell_trap(self, duel: &Duel) -> bool;
    fn get_main_card_type(self, duel: &Duel) -> u32;
    fn is_quick_play_spell(self, duel: &Duel) -> bool;
    fn is_synchro_monster(self, duel: &Duel) -> bool;
    fn is_xyz_monster(self, duel: &Duel) -> bool;
    fn is_base_attack(self, duel: &Duel, atk: i32) -> bool;
    fn is_base_defense(self, duel: &Duel, def: i32) -> bool;
    fn is_effect_monster(self, duel: &Duel) -> bool;
    fn is_non_effect_monster(self, duel: &Duel) -> bool;
    fn is_in_main_mzone(self, duel: &Duel, tp: Option<u8>) -> bool;
    fn is_sequence(self, duel: &Duel, sequence: u32) -> bool;
    fn has_counter(self, duel: &Duel, counter: u16) -> bool;
    fn has_level(self, duel: &Duel) -> bool;
    fn has_defense(self, duel: &Duel) -> bool;
    fn is_original_level(self, duel: &Duel, level: u32) -> bool;
    fn has_rank(self, duel: &Duel) -> bool;
    fn is_original_rank(self, duel: &Duel, rank: u32) -> bool;
    fn lists_code(self, duel: &Duel, code: u32) -> bool;
    fn xyz_number(self, duel: &Duel) -> Option<u32>;
    fn lists_code_as_material(self, duel: &Duel, codes: &[u32]) -> bool;
    fn lists_archetype_as_material(self, duel: &Duel, setcodes: &[u16]) -> bool;
    fn lists_code_with_archetype(self, duel: &Duel, setcodes: &[u16]) -> bool;
    fn is_negatable_monster(self, duel: &Duel) -> bool;
    fn is_negatable_spell_trap(self, duel: &Duel) -> bool;
    fn is_negatable(self, duel: &Duel) -> bool;
    fn is_attribute_except(self, duel: &Duel, att: u32) -> bool;
    fn add_must_be_special_summoned(self, duel: &mut Duel) -> EffectId;
    fn add_must_be_special_summoned_by_card_effect(self, duel: &mut Duel) -> EffectId;
    fn add_must_be_fusion_summoned(self, duel: &mut Duel) -> EffectId;
    fn add_must_be_synchro_summoned(self, duel: &mut Duel) -> EffectId;
    fn add_must_first_be_fusion_summoned(self, duel: &mut Duel) -> EffectId;
    fn add_must_be_special_summoned_by_dark_fusion(self, duel: &mut Duel) -> EffectId;

    fn add_setcodes_rule(self, duel: &mut Duel, code: u32, copyable: bool, setcodes: &[u16]);
    fn negate_effects(self, duel: &mut Duel, c: CardId, reset: u32, negates_cards: bool);
    fn is_fusion_monster(self, duel: &Duel) -> bool;
    fn is_link_monster(self, duel: &Duel) -> bool;
    fn is_pendulum_monster(self, duel: &Duel) -> bool;
    fn is_tribute_summoned(self, duel: &Duel) -> bool;
    fn can_summon_or_set(self, duel: &Duel, ignore_count: bool, procedure: Option<EffectId>, min_tribute: u16) -> bool;
    fn is_fusion_summoned(self, duel: &Duel) -> bool;
    fn is_synchro_summoned(self, duel: &Duel) -> bool;
    fn is_link_summoned(self, duel: &Duel) -> bool;
    fn is_xyz_summoned(self, duel: &Duel) -> bool;
    fn is_pendulum_summoned(self, duel: &Duel) -> bool;
    fn is_in_extra_mzone(self, duel: &Duel) -> bool;
    fn is_ritual_summoned(self, duel: &Duel) -> bool;
    fn is_special_summoned(self, duel: &Duel) -> bool;
    fn is_reincarnation_summoned(self, duel: &Duel) -> bool;
    fn is_reason_card(self, duel: &Duel, c: CardId) -> bool;
    fn has_flag_effect(self, duel: &Duel, id: u32) -> bool;
    fn has_flag_effect_full(self, duel: &Duel, id: u32, count: usize) -> bool;
    fn announce_another_attribute(self, duel: &mut Duel, tp: u8) -> Script<u32>;
    fn is_race_except(self, duel: &Duel, race: u64) -> bool;
    fn update_level(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>) -> i32;
    fn update_attack(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32;
    fn update_defense(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32;
    fn equip_by_effect_and_limit_register(
        self,
        duel: &mut Duel,
        e: EffectId,
        tp: u8,
        tc: CardId,
        code: Option<u32>,
        mustbefaceup: Option<bool>,
    ) -> Script<bool>;
}

impl CardUtility for CardId {
    fn is_fusion_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_FUSION)
    }

    fn is_link_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_LINK)
    }

    fn is_pendulum_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_PENDULUM)
    }

    fn is_tribute_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_TRIBUTE])
    }

    // Card.CanSummonOrSet with the arguments (ignore_count, procedure, min_tribute) given.
    fn can_summon_or_set(self, duel: &Duel, ignore_count: bool, procedure: Option<EffectId>, min_tribute: u16) -> bool {
        self.is_summonable_full(duel, ignore_count, procedure, min_tribute, 0x1f)
            || self.is_msetable_full(duel, ignore_count, procedure, min_tribute, 0xff)
    }

    fn is_fusion_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_FUSION])
    }

    fn is_synchro_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_SYNCHRO])
    }

    fn is_link_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_LINK])
    }

    fn is_xyz_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_XYZ])
    }

    fn is_pendulum_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_PENDULUM])
    }

    // Card.IsInExtraMZone called without a player.
    fn is_in_extra_mzone(self, duel: &Duel) -> bool {
        self.is_location(duel, LOCATION_EMZONE)
    }

    fn is_ritual_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_RITUAL])
    }

    fn is_special_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL])
    }

    // Card.IsReincarnationSummoned.
    fn is_reincarnation_summoned(self, duel: &Duel) -> bool {
        let label = self
            .get_flag_effect_labels(duel, CARD_SALAMANGREAT_SANCTUARY)
            .into_iter()
            .fold(0, |label, lab| label | lab);
        label & (i64::from(self.get_summon_player(duel)) + 1) != 0
    }

    fn is_reason_card(self, duel: &Duel, c: CardId) -> bool {
        c.get_reason_card(duel) == Some(self)
    }

    fn has_flag_effect(self, duel: &Duel, id: u32) -> bool {
        self.has_flag_effect_full(duel, id, 1)
    }

    fn has_flag_effect_full(self, duel: &Duel, id: u32, count: usize) -> bool {
        self.get_flag_effect(duel, id) >= count
    }

    fn announce_another_attribute(self, duel: &mut Duel, tp: u8) -> Script<u32> {
        announce_another_attribute(duel, self.get_attribute(duel), tp)
    }

    // Card.IsRaceExcept(c, race, scard, sumtype, playerid): the scard arguments only reach race effects that apply as
    // material.
    fn is_race_except(self, duel: &Duel, race: u64) -> bool {
        let own = self.get_race(duel);
        own & race != own
    }

    // Card.UpdateLevel(c, amt, reset, rc): `reset` None is RESET_EVENT with the standard resets (and RESET_DISABLE
    // when `rc` is the card itself, its default).
    fn update_level(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>) -> i32 {
        let c = self;
        let rc = rc.unwrap_or(c);
        let r = if c == rc { RESETS_STANDARD_DISABLE } else { RESETS_STANDARD };
        let reset = reset.unwrap_or(RESET_EVENT + r);
        let lv = c.get_level(duel) as i32;
        if !c.is_level_below(duel, 2_147_483_647) {
            return 0;
        }
        let amt = if lv + amt <= 0 { -(lv - 1) } else { amt };
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, amt);
        e1.set_reset(duel, reset);
        c.register_effect(duel, e1);
        c.get_level(duel) as i32 - lv
    }

    fn update_attack(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32 {
        let c = self;
        let rc = rc.unwrap_or(c);
        let r = if c == rc { RESETS_STANDARD_DISABLE } else { RESETS_STANDARD };
        let reset = reset.unwrap_or(RESET_EVENT + r);
        let atk = c.get_attack(duel);
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, amt);
        e1.set_reset_full(duel, reset, reset_count.unwrap_or(1));
        c.register_effect(duel, e1);
        c.get_attack(duel) - atk
    }

    fn update_defense(self, duel: &mut Duel, amt: i32, reset: Option<u32>, rc: Option<CardId>, reset_count: Option<u16>) -> i32 {
        let c = self;
        let rc = rc.unwrap_or(c);
        let r = if c == rc { RESETS_STANDARD_DISABLE } else { RESETS_STANDARD };
        let reset = reset.unwrap_or(RESET_EVENT + r);
        let def = c.get_defense(duel);
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
        e1.set_value(duel, amt);
        e1.set_reset_full(duel, reset, reset_count.unwrap_or(1));
        c.register_effect(duel, e1);
        c.get_defense(duel) - def
    }

    // Card.EquipByEffectAndLimitRegister; `up` is nil in Lua unless mustbefaceup is given, which Duel.Equip reads as false.
    fn equip_by_effect_and_limit_register(
        self,
        duel: &mut Duel,
        e: EffectId,
        tp: u8,
        tc: CardId,
        code: Option<u32>,
        mustbefaceup: Option<bool>,
    ) -> Script<bool> {
        let c = self;
        if !duel.equip_full(tp, tc, c, mustbefaceup.unwrap_or(false), false)? {
            return Ok(false);
        }
        if let Some(code) = code {
            tc.register_flag_effect(duel, code, RESET_EVENT + RESETS_STANDARD, 0, 0);
        }
        let te = e.get_label_object(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_OWNER_RELATE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        e1.set_value(duel, Value::Card(|duel, e, c| equip_by_effect_limit(duel, e, c).into()));
        e1.set_label_object(duel, te);
        tc.register_effect(duel, e1);
        Ok(true)
    }

    fn is_monster(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_MONSTER)
    }

    fn is_spell(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_SPELL)
    }

    fn is_trap(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_TRAP)
    }

    fn is_equip_card(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_EQUIP)
    }

    fn is_spell_trap(self, duel: &Duel) -> bool {
        self.is_type(duel, TYPE_SPELL | TYPE_TRAP)
    }

    fn is_monster_card(self, duel: &Duel) -> bool {
        self.is_original_type(duel, TYPE_MONSTER)
    }

    fn is_trap_monster(self, duel: &Duel) -> bool {
        self.is_original_type(duel, TYPE_TRAP)
            && (self.get_original_level(duel) > 0 || self.get_original_attribute(duel) > 0 || self.get_original_race(duel) > 0)
    }

    fn has_non_zero_attack(self, duel: &Duel) -> bool {
        self.is_faceup(duel) && self.get_attack(duel) > 0
    }

    fn has_non_zero_defense(self, duel: &Duel) -> bool {
        self.is_faceup(duel) && self.get_defense(duel) > 0
    }

    fn is_owner(self, duel: &Duel, player: u8) -> bool {
        self.get_owner(duel) == player
    }

    fn is_previous_attribute_on_field(self, duel: &Duel, attribute: u32) -> bool {
        self.get_previous_attribute_on_field(duel) & attribute > 0
    }

    fn is_previous_type_on_field(self, duel: &Duel, card_type: u32) -> bool {
        self.get_previous_type_on_field(duel) & card_type > 0
    }

    fn is_previous_code_on_field(self, duel: &Duel, codes: &[u32]) -> bool {
        let previous = [self.get_previous_code_on_field(duel), self.get_previous_code2_on_field(duel)];
        codes.iter().any(|code| previous.contains(code))
    }

    fn is_equip_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_EQUIP)
    }

    fn is_field_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_FIELD)
    }

    fn is_ritual_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_RITUAL)
    }

    fn is_ritual_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_RITUAL)
    }

    fn is_normal_spell(self, duel: &Duel) -> bool {
        self.get_type(duel) == TYPE_SPELL
    }

    fn is_normal_trap(self, duel: &Duel) -> bool {
        self.get_type(duel) == TYPE_TRAP
    }

    fn is_normal_summoned(self, duel: &Duel) -> bool {
        self.is_summon_type(duel, &[SUMMON_TYPE_NORMAL])
    }

    fn is_normal_spell_trap(self, duel: &Duel) -> bool {
        [TYPE_SPELL, TYPE_TRAP].contains(&self.get_type(duel))
    }

    fn is_continuous_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_CONTINUOUS)
    }

    fn is_continuous_trap(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_TRAP | TYPE_CONTINUOUS)
    }

    fn is_continuous_spell_trap(self, duel: &Duel) -> bool {
        self.is_continuous_spell(duel) || self.is_continuous_trap(duel)
    }

    fn get_main_card_type(self, duel: &Duel) -> u32 {
        self.get_type(duel) & (TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP)
    }

    fn is_quick_play_spell(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_SPELL | TYPE_QUICKPLAY)
    }

    fn is_synchro_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_SYNCHRO)
    }

    fn is_xyz_monster(self, duel: &Duel) -> bool {
        self.is_composite_type(duel, TYPE_MONSTER | TYPE_XYZ)
    }

    fn is_base_attack(self, duel: &Duel, atk: i32) -> bool {
        self.get_base_attack(duel) == atk
    }

    fn is_base_defense(self, duel: &Duel, def: i32) -> bool {
        self.has_defense(duel) && self.get_base_defense(duel) == def
    }

    fn is_effect_monster(self, duel: &Duel) -> bool {
        self.is_monster(duel) && self.is_type(duel, TYPE_EFFECT)
    }

    fn is_non_effect_monster(self, duel: &Duel) -> bool {
        self.is_monster(duel) && !self.is_type(duel, TYPE_EFFECT)
    }

    fn is_in_main_mzone(self, duel: &Duel, tp: Option<u8>) -> bool {
        self.is_location(duel, LOCATION_MMZONE) && tp.is_none_or(|tp| self.is_controler(duel, tp))
    }

    fn has_counter(self, duel: &Duel, counter: u16) -> bool {
        self.get_counter(duel, counter) > 0
    }

    fn is_sequence(self, duel: &Duel, sequence: u32) -> bool {
        self.get_sequence(duel) == sequence
    }

    // Card.HasDefense; a card in Maximum Mode carries the flag effects of proc_maximum.lua (170000000 and 170000001).
    fn has_defense(self, duel: &Duel) -> bool {
        if self.is_monster(duel) {
            !(self.is_type(duel, TYPE_LINK) || self.has_flag_effect(duel, 170_000_000) || self.has_flag_effect(duel, 170_000_001))
        } else {
            self.is_original_type(duel, TYPE_MONSTER) && !self.is_original_type(duel, TYPE_LINK)
        }
    }

    fn has_level(self, duel: &Duel) -> bool {
        if self.is_monster(duel) {
            let card_type = self.get_type(duel);
            return card_type & TYPE_LINK != TYPE_LINK
                && card_type & TYPE_XYZ != TYPE_XYZ
                && !(self.is_has_effect(duel, EFFECT_RANK_LEVEL).is_some() || self.is_has_effect(duel, EFFECT_RANK_LEVEL_S).is_some())
                && !self.is_status(duel, STATUS_NO_LEVEL);
        }
        if self.is_original_type(duel, TYPE_MONSTER) {
            return !(self.is_original_type(duel, TYPE_XYZ + TYPE_LINK) || self.is_status(duel, STATUS_NO_LEVEL));
        }
        false
    }

    fn is_original_level(self, duel: &Duel, level: u32) -> bool {
        self.has_level(duel) && self.get_original_level(duel) == level
    }

    fn has_rank(self, duel: &Duel) -> bool {
        if self.is_monster(duel) {
            return self.is_type(duel, TYPE_XYZ)
                || (self.has_level(duel)
                    && (self.is_has_effect(duel, EFFECT_LEVEL_RANK).is_some() || self.is_has_effect(duel, EFFECT_LEVEL_RANK_S).is_some()));
        }
        self.is_original_type(duel, TYPE_MONSTER) && self.is_original_type(duel, TYPE_XYZ)
    }

    fn is_original_rank(self, duel: &Duel, rank: u32) -> bool {
        self.has_rank(duel) && self.get_original_rank(duel) == rank
    }

    fn lists_code(self, duel: &Duel, code: u32) -> bool {
        crate::official::listed_names(self.get_original_code(duel)).contains(&code)
    }

    fn xyz_number(self, duel: &Duel) -> Option<u32> {
        crate::official::xyz_number(self.get_original_code(duel))
    }

    fn lists_code_as_material(self, duel: &Duel, codes: &[u32]) -> bool {
        let original = self.get_original_code(duel);
        let material = Fusion::material(duel, original);
        codes
            .iter()
            .any(|code| material.contains(code) || crate::official::material(original).contains(code))
    }

    fn lists_archetype_as_material(self, duel: &Duel, setcodes: &[u16]) -> bool {
        let listed = crate::official::material_setcode(self.get_original_code(duel));
        setcodes
            .iter()
            .any(|&setcode| listed.iter().any(|&to_match| match_setcode(setcode, to_match)))
    }

    fn lists_code_with_archetype(self, duel: &Duel, setcodes: &[u16]) -> bool {
        crate::official::listed_names(self.get_original_code(duel))
            .iter()
            .flat_map(|&code| duel.get_card_setcode_from_code(code))
            .any(|to_match| setcodes.iter().any(|&setcode| match_setcode(setcode, to_match)))
    }

    fn is_negatable_monster(self, duel: &Duel) -> bool {
        self.is_faceup(duel)
            && !self.is_disabled(duel)
            && (!self.is_non_effect_monster(duel) || self.get_original_type(duel) & TYPE_EFFECT != 0)
    }

    fn is_negatable_spell_trap(self, duel: &Duel) -> bool {
        self.is_faceup(duel) && !self.is_disabled(duel) && self.is_spell_trap(duel)
    }

    fn is_attribute_except(self, duel: &Duel, att: u32) -> bool {
        let current = self.get_attribute(duel);
        current & att != current
    }

    fn is_negatable(self, duel: &Duel) -> bool {
        self.is_negatable_monster(duel) || self.is_negatable_spell_trap(duel)
    }

    fn add_must_be_special_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_be_special_summoned_by_card_effect(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(
            duel,
            Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
        );
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_be_fusion_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(
            duel,
            Value::SpSummonCondition(|_, _, _, _, sum_type, _, _| sum_type & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION),
        );
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_be_synchro_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(duel, Value::SpSummonCondition(aux::synlimit));
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_first_be_fusion_summoned(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(
            duel,
            Value::SpSummonCondition(|duel, e, _, _, sum_type, _, _| {
                !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || sum_type & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
            }),
        );
        self.register_effect(duel, e0);
        e0
    }

    fn add_must_be_special_summoned_by_dark_fusion(self, duel: &mut Duel) -> EffectId {
        let e0 = duel.create_effect(self);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
        e0.set_value(duel, Value::SpSummonCondition(aux::evil_hero_limit));
        self.register_effect(duel, e0);
        let e1 = duel.create_effect(self);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, CARD_CLOCK_LIZARD);
        e1.set_condition(duel, |duel, a| {
            duel.is_player_affected_by_effect(a.e.get_handler_player(duel), EFFECT_SUPREME_CASTLE)
                .is_none()
        });
        e1.set_value(duel, 1);
        self.register_effect(duel, e1);
        e0
    }

    fn add_setcodes_rule(self, duel: &mut Duel, code: u32, copyable: bool, setcodes: &[u16]) {
        let property = if copyable { 0 } else { EFFECT_FLAG_UNCOPYABLE };
        for &setcode in setcodes {
            let e = duel.create_effect(self);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + property);
            e.set_code(duel, EFFECT_ADD_SETCODE);
            e.set_value(duel, i32::from(setcode));
            e.set_label(duel, i64::from(code));
            e.set_condition(duel, |duel, a| {
                let c = a.e.get_handler(duel);
                let label = a.e.get_label(duel);
                label <= 0 || c.get_original_code_rule(duel) != label as u32 || c.is_code(duel, label as u32)
            });
            self.register_effect(duel, e);
        }
    }

    // Card.NegateEffects(tc, c, reset, negates_cards) with ct 1.
    fn negate_effects(self, duel: &mut Duel, c: CardId, reset: u32, negates_cards: bool) {
        let tc = self;
        let mut reset = reset | (RESET_EVENT | RESETS_STANDARD);
        let trap_monster_chk = negates_cards && tc.is_type(duel, TYPE_TRAPMONSTER);
        if trap_monster_chk {
            reset &= !(RESET_TOFIELD | RESET_LEAVE | RESET_TURN_SET);
        }
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, reset);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
        if trap_monster_chk {
            let e3 = e1.clone_effect(duel);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            tc.register_effect(duel, e3);
        }
    }
}

pub trait EffectUtility: Copy {
    fn is_monster_effect(self, duel: &Duel) -> bool;
    fn is_spell_effect(self, duel: &Duel) -> bool;
    fn is_trap_effect(self, duel: &Duel) -> bool;
    fn is_spell_trap_effect(self, duel: &Duel) -> bool;
    fn has_self_discard_cost(self, duel: &Duel) -> bool;
    fn has_self_change_position_cost(self, duel: &Duel) -> bool;
}

impl EffectUtility for EffectId {
    fn is_spell_trap_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_SPELL | TYPE_TRAP)
    }

    // Effect.HasSelfDiscardCost, which looks the cost up in a table of the library's own costs: Cost.SelfDiscard, Cost.SelfDiscardToGrave and
    // the Dragon Ruler ones, the only composite costs ported with it.
    fn has_self_discard_cost(self, duel: &Duel) -> bool {
        let costs: [CostFn; 7] = [
            Cost::self_discard,
            Cost::self_discard_to_grave,
            DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_EARTH>,
            DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_WATER>,
            DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_FIRE>,
            DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_WIND>,
            DragonRuler::self_discard_cost::<ATTRIBUTE_LIGHT>,
        ];
        self.get_cost(duel)
            .is_some_and(|cost| costs.into_iter().any(|known| std::ptr::fn_addr_eq(cost, known)))
    }

    // Effect.HasSelfChangePositionCost: Cost.SelfChangePosition is not ported, so no ported effect has it.
    fn has_self_change_position_cost(self, _: &Duel) -> bool {
        false
    }

    fn is_monster_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_MONSTER)
    }

    fn is_spell_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_SPELL)
    }

    fn is_trap_effect(self, duel: &Duel) -> bool {
        self.is_active_type(duel, TYPE_TRAP)
    }
}

// Card.AnnounceAnotherAttribute and Duel.AnnounceAnotherAttribute: any attribute but the one of `attribute`, if it is single.
fn match_setcode(setcode: u16, to_match: u16) -> bool {
    setcode & 0xfff == to_match & 0xfff && setcode & to_match == setcode
}

fn announce_another_attribute(duel: &mut Duel, attribute: u32, tp: u8) -> Script<u32> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let available = if attribute & attribute.wrapping_sub(1) == 0 {
        !attribute & ATTRIBUTE_ALL
    } else {
        ATTRIBUTE_ALL
    };
    duel.announce_attribute(tp, 1, available)
}

pub trait DuelUtility {
    fn select_effect(&mut self, tp: u8, effects: &[(bool, u64)]) -> Script<Option<usize>>;
    fn has_flag_effect(&self, tp: u8, id: u32) -> bool;
    fn has_flag_effect_full(&self, tp: u8, id: u32, ct: usize) -> bool;
    fn get_target_cards(&self, e: EffectId) -> Option<CardSet>;
    fn announce_another_attribute(&mut self, g: CardSet, tp: u8) -> Script<u32>;
    fn check_pendulum_zones(&self, tp: u8) -> bool;
    fn move_to_deck_bottom_count(&mut self, count: usize, tp: u8) -> Script<()>;
    fn move_to_deck_bottom(&mut self, targets: CardSet) -> Script<()>;
    fn move_to_deck_top(&mut self, targets: CardSet) -> Script<()>;
    fn place_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool>;
    fn activate_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool>;
    fn count_heads(&self, results: &[u8]) -> usize;
    fn call_coin(&mut self, tp: u8) -> Script<bool>;
    fn get_zone_with_linked_count(&self, count: usize, tp: u8) -> u32;
    fn summon_or_set(&mut self, tp: u8, c: CardId, ignore_count: bool, procedure: Option<EffectId>, min_tribute: u8) -> Script<()>;
}

impl DuelUtility for Duel {
    fn check_pendulum_zones(&self, tp: u8) -> bool {
        self.check_location(tp, LOCATION_PZONE, 0) || self.check_location(tp, LOCATION_PZONE, 1)
    }

    // Duel.GetTargetCards: the chain link's targets still related to `e`, nil without targets.
    fn get_target_cards(&self, e: EffectId) -> Option<CardSet> {
        let tg = self.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)?;
        Some(tg.filter(self, |c, duel| c.is_relate_to_effect(duel, e), None))
    }

    // The 1-based index of the effect chosen among those whose condition holds, None if none does.
    fn select_effect(&mut self, tp: u8, effects: &[(bool, u64)]) -> Script<Option<usize>> {
        let (sel, eff): (Vec<usize>, Vec<u64>) = effects
            .iter()
            .enumerate()
            .filter(|(_, (available, _))| *available)
            .map(|(index, &(_, description))| (index + 1, description))
            .unzip();
        if eff.is_empty() {
            return Ok(None);
        }
        let option = self.select_option(tp, &eff)?;
        Ok(usize::try_from(option).ok().and_then(|option| sel.get(option)).copied())
    }

    fn has_flag_effect(&self, tp: u8, id: u32) -> bool {
        self.has_flag_effect_full(tp, id, 1)
    }

    // Duel.GetZoneWithLinkedCount: the zones, seen from `tp`, that at least `count` Link monsters of either player point to.
    fn get_zone_with_linked_count(&self, count: usize, tp: u8) -> u32 {
        let links = self.get_matching_group(|c, duel| c.is_type(duel, TYPE_LINK), tp, LOCATION_MZONE, LOCATION_MZONE, None);
        let linked: Vec<u32> = links.iter().map(|c| c.get_linked_zone_full(self, tp)).collect();
        let pointed_by = |zone: u32| linked.iter().filter(|&&pointed| zone & pointed != 0).count();
        (0..7)
            .map(|bit| 1 << bit)
            .filter(|&zone| pointed_by(zone) >= count)
            .fold(0, |rzone, zone| rzone | zone)
    }

    fn call_coin(&mut self, tp: u8) -> Script<bool> {
        let call = self.announce_coin(tp)?;
        Ok(self.toss_coin(tp, 1)?.first() == Some(&call))
    }

    fn count_heads(&self, results: &[u8]) -> usize {
        results.iter().filter(|&&result| u32::from(result) == COIN_HEADS).count()
    }

    fn has_flag_effect_full(&self, tp: u8, id: u32, ct: usize) -> bool {
        self.get_flag_effect(tp, id) >= ct
    }

    fn summon_or_set(&mut self, tp: u8, c: CardId, ignore_count: bool, procedure: Option<EffectId>, min_tribute: u8) -> Script<()> {
        let s1 = c.is_summonable_full(self, ignore_count, procedure, u16::from(min_tribute), 0x1f);
        let s2 = c.is_msetable_full(self, ignore_count, procedure, u16::from(min_tribute), 0xff);
        if (s1 && s2 && self.select_position(tp, c, (POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE) as u8)? == POS_FACEUP_ATTACK) || !s2 {
            self.summon_full(tp, c, ignore_count, procedure, min_tribute, 0x1f)
        } else {
            self.mset_full(tp, c, ignore_count, procedure, min_tribute, 0x1f)
        }
    }

    // Duel.PlaceFieldSpell(c, tp, target_p) with target_p left out.
    fn place_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool> {
        let fc = self.get_field_card(tp, LOCATION_FZONE, 0);
        if self.is_duel_type(DUEL_1_FIELD) {
            if let Some(fc) = fc {
                self.destroy(fc, REASON_RULE)?;
            }
        } else if let Some(fc) = fc
            && self.send_to_grave(fc, REASON_RULE)? == 0
        {
            self.send_to_grave(c, REASON_RULE)?;
            return Ok(false);
        }
        self.break_effect()?;
        let res = self.move_to_field(c, tp, tp, LOCATION_FZONE, POS_FACEUP, true)?;
        if res
            && self.is_duel_type(DUEL_1_FIELD)
            && let Some(fc) = self.get_field_card(1 - tp, LOCATION_FZONE, 0)
            && self.destroy(fc, REASON_RULE)? == 0
        {
            self.send_to_grave(c, REASON_RULE)?;
            return Ok(false);
        }
        Ok(res)
    }

    // Duel.ActivateFieldSpell(c, e, tp, ...) with target_p left out; the cost of the activate effect is not run.
    fn activate_field_spell(&mut self, c: CardId, tp: u8) -> Script<bool> {
        if !self.place_field_spell(c, tp)? {
            return Ok(false);
        }
        let te = c.get_activate_effect(self);
        unported!(self, te.is_none_or(|te| te.get_cost(self).is_some()));
        let Some(te) = te else { return Ok(false) };
        te.use_count_limit_full(self, tp, 1, true);
        let chain = u32::from(self.get_current_chain());
        self.raise_event(c, 4179255, Some(te), 0, tp, tp, chain)?;
        Ok(true)
    }

    // Duel.MoveToDeckBottom(count, tp): the top card of the Deck, `count` times.
    fn move_to_deck_bottom_count(&mut self, count: usize, tp: u8) -> Script<()> {
        for _ in 0..count {
            if let Some(top) = self.get_decktop_group(tp, 1).get_first() {
                self.move_sequence(top, SEQ_DECKBOTTOM as u8)?;
            }
        }
        Ok(())
    }

    // Duel.MoveToDeckBottom(group): the cards in the Deck, in group order.
    fn move_to_deck_bottom(&mut self, targets: CardSet) -> Script<()> {
        for c in targets.filter(self, |c, duel| c.is_location(duel, LOCATION_DECK), None) {
            self.move_sequence(c, SEQ_DECKBOTTOM as u8)?;
        }
        Ok(())
    }

    // Duel.MoveToDeckTop(obj): the cards in the Deck, in group order.
    fn move_to_deck_top(&mut self, targets: CardSet) -> Script<()> {
        for c in targets.filter(self, |c, duel| c.is_location(duel, LOCATION_DECK), None) {
            self.move_sequence(c, SEQ_DECKTOP as u8)?;
        }
        Ok(())
    }

    fn announce_another_attribute(&mut self, g: CardSet, tp: u8) -> Script<u32> {
        let attribute = g.get_bitwise_or(self, |c, duel| u64::from(c.get_attribute(duel))) as u32;
        announce_another_attribute(self, attribute, tp)
    }
}

// Card.EquipByEffectLimit.
fn equip_by_effect_limit(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_owner(duel) == c
        && c.get_card_effect(duel, 89785779 + EFFECT_EQUIP_LIMIT)
            .into_iter()
            .any(|te| Some(te) == e.get_label_object(duel))
}

// The parameters of Cyberdark.EquipFilter, EquipTarget and EquipOperation for the effects that target (targets true); the ones that
// do not come with the cards that use them. `f` is the filter before EquipFilter wraps it, `equipop` the op.
pub trait CyberdarkEquip {
    const MANDATORY: bool;

    fn f(c: CardId, duel: &Duel) -> bool;

    fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()>;
}

pub struct Cyberdark;

impl Cyberdark {
    fn equip_filter<P: CyberdarkEquip>(c: CardId, duel: &Duel, tp: u8) -> bool {
        c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel) && P::f(c, duel)
    }

    pub fn equip_target<P: CyberdarkEquip>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        let tp = a.tp;
        let wc = duel.is_player_affected_by_effect(tp, EFFECT_CYBERDARK_WORLD).is_some();
        let filter = |c: CardId, duel: &Duel| Cyberdark::equip_filter::<P>(c, duel, tp);
        if let Some(chkc) = chkc {
            return Ok(chkc.is_location(duel, LOCATION_GRAVE) && (wc || chkc.is_controler(duel, tp)) && filter(chkc, duel));
        }
        let loc = if wc { LOCATION_GRAVE } else { 0 };
        if chk == 0 {
            return Ok(P::MANDATORY
                || (duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, loc, 1, None)));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, loc, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
        duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
        Ok(false)
    }

    pub fn equip_operation<P: CyberdarkEquip>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
            return Ok(());
        }
        let c = e.get_handler(duel);
        if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
            && Cyberdark::equip_filter::<P>(tc, duel, tp)
        {
            P::equipop(duel, c, e, tp, tc)?;
        }
        Ok(())
    }
}

// The Witchcrafter functions of cards_specific_functions.lua; CreateCostReplaceEffect comes with the cards that call it.
pub struct Witchcrafter;

impl Witchcrafter {
    // Witchcrafter.DiscardCost: Cost.Replaceable(Cost.Discard(Card.IsSpell)).
    pub fn discard_cost(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::replaceable(duel, args, chk, |duel, args, chk| {
            Cost::discard_full(duel, args, chk, CardId::is_spell, false, (1, 1))
        })
    }

    // Witchcrafter.TributeAndDiscardCost: Cost.AND(Cost.SelfTribute, Witchcrafter.DiscardCost).
    pub fn tribute_and_discard_cost(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        Cost::and(duel, args, chk, &[Cost::self_tribute, Witchcrafter::discard_cost])
    }
}

// The Dragon Ruler functions of cards_specific_functions.lua.
pub struct DragonRuler;

impl DragonRuler {
    // discard_with_other_cost(filter): returns false when executed, which stops the Cost.AND it is part of: it sends both cards.
    fn discard_with_other_cost(duel: &mut Duel, args: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool) -> Script<bool> {
        let (c, tp) = (args.e.get_handler(duel), args.tp);
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
        duel.send_to_grave(g.union(c.into()), REASON_COST | REASON_DISCARD)?;
        Ok(false)
    }

    // DragonRuler.SelfDiscardToGraveCost(attr): Cost.AND(discard_with_other_cost(filter), Cost.SelfDiscardToGrave).
    pub fn self_discard_to_grave_cost<const ATTRIBUTE: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        let filter = |c: CardId, duel: &Duel| c.is_attribute(duel, ATTRIBUTE) && c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel);
        let first = DragonRuler::discard_with_other_cost(duel, args, chk, filter)?;
        Ok(chk == 0 && first && c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel))
    }

    // DragonRuler.SelfDiscardCost(attr): Cost.AND(discard_with_other_cost(filter), Cost.SelfDiscard).
    pub fn self_discard_cost<const ATTRIBUTE: u32>(duel: &mut Duel, args: &Args, chk: i32) -> Script<bool> {
        let c = args.e.get_handler(duel);
        let filter = |c: CardId, duel: &Duel| (c.is_attribute(duel, ATTRIBUTE) || c.is_race(duel, RACE_DRAGON)) && c.is_discardable(duel);
        let first = DragonRuler::discard_with_other_cost(duel, args, chk, filter)?;
        Ok(chk == 0 && first && c.is_discardable(duel))
    }
}

// The parameters of Effect.CreateMysteruneQPEffect: the unique effect's category, target and operation (the operation returns
// whether to go on to the banishment), the cards banished from the opponent's Deck and the unique effect's extra property.
// The uniquecode variant, which returns two effects, and a rmcount function have no caller.
pub trait MysteruneQuickPlay {
    const ID: u32;
    const UNIQUECAT: u64;
    const RMCOUNT: usize;
    const UNIQUEPROP: Option<u32> = None;

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool>;

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool>;
}

// The Mysterune functions of cards_specific_functions.lua.
pub struct Mysterune;

impl Mysterune {
    pub fn create_qp_effect<P: MysteruneQuickPlay>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER + TIMING_END_PHASE);
        e1.set_count_limit_full(duel, 1, P::ID, EFFECT_COUNT_CODE_OATH);
        e1.set_target(duel, Mysterune::choicetg::<P>);
        e1
    }

    fn skipop(duel: &mut Duel, a: &Args) {
        if !a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            return;
        }
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_SKIP_BP);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_target_range(duel, 1, 0);
        let reset = RESET_PHASE + PHASE_BATTLE + RESET_SELF_TURN;
        if duel.is_turn_player(a.tp) && duel.is_battle_phase() {
            let turn = i64::from(duel.get_turn_count());
            e1.set_label(duel, turn);
            e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
            e1.set_reset_full(duel, reset, 2);
        } else {
            e1.set_reset_full(duel, reset, 1);
        }
        duel.register_effect(e1, a.tp);
    }

    fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
        c.is_set_card(duel, &[SET_RUNICK])
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.get_location_count_from_ex_full(tp, tp, None, c, ZONES_EMZ) > 0
    }

    fn sptg(duel: &mut Duel, a: &Args, chk: i32) -> bool {
        let (e, tp) = (a.e, a.tp);
        if chk == 0 {
            return duel.is_existing_matching_card(|c, duel| Mysterune::spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
        false
    }

    fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| Mysterune::spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon_full(g, 0, tp, tp, false, false, POS_FACEUP, ZONES_EMZ)?;
        }
        Mysterune::skipop(duel, a);
        Ok(())
    }

    fn rmtg<P: MysteruneQuickPlay>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        let tp = a.tp;
        if chk == 0 {
            return Ok(P::uniquetg(duel, a, 0, None)?
                && P::RMCOUNT > 0
                && duel.is_player_can_remove(tp, None)
                && duel.get_field_group_count(tp, 0, LOCATION_DECK) >= P::RMCOUNT);
        }
        P::uniquetg(duel, a, 1, None)?;
        if P::RMCOUNT > 0 {
            duel.set_operation_info(0, CATEGORY_REMOVE, None, P::RMCOUNT as u8, 1 - tp, LOCATION_DECK as i32);
        }
        Ok(false)
    }

    fn rmop<P: MysteruneQuickPlay>(duel: &mut Duel, a: &Args) -> Script<()> {
        let tp = a.tp;
        if P::uniqueop(duel, a)? && duel.is_player_can_remove(tp, None) {
            let rg = duel.get_decktop_group(1 - tp, P::RMCOUNT);
            if !rg.is_empty() {
                duel.disable_shuffle_check(true);
                duel.break_effect()?;
                duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
            }
        }
        Mysterune::skipop(duel, a);
        Ok(())
    }

    fn choicetg<P: MysteruneQuickPlay>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        if chkc.is_some() {
            return P::uniquetg(duel, a, chk, chkc);
        }
        let b1 = Mysterune::rmtg::<P>(duel, a, 0)?;
        let b2 = Mysterune::sptg(duel, a, 0);
        if chk == 0 {
            return Ok(b1 || b2);
        }
        match duel.select_effect(tp, &[(b1, aux::stringid(P::ID, 0)), (b2, aux::stringid(P::ID, 1))])? {
            Some(1) => {
                if let Some(property) = P::UNIQUEPROP {
                    e.set_property(duel, property);
                }
                e.set_category(duel, P::UNIQUECAT | CATEGORY_REMOVE);
                Mysterune::rmtg::<P>(duel, a, 1)?;
                e.set_operation(duel, Mysterune::rmop::<P>);
            }
            Some(2) => {
                e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
                e.set_property(duel, 0);
                Mysterune::sptg(duel, a, 1);
                e.set_operation(duel, Mysterune::spop);
            }
            _ => {
                e.set_category(duel, 0);
                e.set_property(duel, 0);
                e.clear_operation(duel);
            }
        }
        Ok(false)
    }
}
