use std::num::NonZeroU64;
use std::ops::{DerefMut, Index, IndexMut};

use crate::card::{CardId, CardSet, LocInfo, Relation};
use crate::chain::Continuous;
use crate::constants::*;
use crate::duel::{Duel, Shared};
use crate::effect::{
    Chain, ChainLimit, Container, CounterFilter, CustomCounter, EffectId, EffectSet, Event, Target, container_key, remove_item,
};
use crate::libduel::ReleaseOptions;
use crate::list::List;
use crate::operations::{SelfDestroy, SelfDestroyUnique, SendToArgs};
use crate::processor::{Adjust, Unit};

pub(crate) type ReleaseFilter<'a> = &'a dyn Fn(CardId, &Duel) -> bool;

// Per-player arrays indexed by player id like the core's `x[playerid]`; any id but 0 reads player 1.
#[derive(Debug, Copy, Default, PartialEq, Eq, Hash)]
pub struct PerPlayer<T>(pub [T; 2]);

impl<T: Clone> Clone for PerPlayer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<T> Index<u8> for PerPlayer<T> {
    type Output = T;

    fn index(&self, player: u8) -> &T {
        let [zero, one] = &self.0;
        if player == 0 { zero } else { one }
    }
}

impl<T> IndexMut<u8> for PerPlayer<T> {
    fn index_mut(&mut self, player: u8) -> &mut T {
        let [zero, one] = &mut self.0;
        if player == 0 { zero } else { one }
    }
}

// What get_attack_target reports besides the targets: the type of attack and must_attack_map.
#[derive(Debug, Clone)]
pub(crate) struct AttackInfo {
    pub atype: u8,
    pub must_attack_map: MustAttackMap,
}

// The EFFECT_MUST_ATTACK_MONSTER effect each must-attack target is under, in the core's multimap order: by effect, then insertion.
pub type MustAttackMap = Vec<(EffectId, CardId)>;

// The monster an Extra Deck zone count is for; without a card, the core's temp card with no type or the type GetLocationCountFromEx names.
#[derive(Debug, Clone, Copy)]
pub struct FromEx {
    pub card: Option<CardId>,
    pub card_type: u32,
    pub facedown: bool,
}

impl FromEx {
    pub fn card(duel: &Duel, card: Option<CardId>) -> Self {
        let Some(card) = card else {
            return Self {
                card: None,
                card_type: 0,
                facedown: false,
            };
        };
        Self {
            card: Some(card),
            card_type: duel.cards[card].data.card_type,
            facedown: duel.cards[card].is_position(POS_FACEDOWN),
        }
    }

    pub fn card_type(card_type: u32) -> Self {
        Self {
            card: None,
            card_type: TYPE_MONSTER | card_type,
            facedown: card_type & TYPE_PENDULUM == 0,
        }
    }
}

// The zones a count reads: the field's, without some monsters (Duel.GetMZoneCount), or with a monster placed for check_extra_link.
#[derive(Debug, Clone, Copy)]
pub struct Zones {
    pub used: PerPlayer<u32>,
    pub disabled: PerPlayer<u32>,
    pub mzone: PerPlayer<[Option<CardId>; 7]>,
    pub placed: Option<(CardId, u8, u32)>,
}

// player_info; tag and relay duels (extra_lists_*) are unsupported.
#[derive(Debug, Default)]
pub struct PlayerInfo {
    pub lp: i32,
    pub start_count: i32,
    pub draw_count: i32,
    pub used_location: u32,
    pub disabled_location: u32,
    pub extra_p_count: u32,
    pub list_mzone: [Option<CardId>; 7],
    pub list_szone: [Option<CardId>; 8],
    pub list_main: Pile,
    pub list_grave: Pile,
    pub list_hand: Pile,
    pub list_remove: Pile,
    pub list_extra: Pile,
}

clone_by_field!(PlayerInfo {
    lp,
    start_count,
    draw_count,
    used_location,
    disabled_location,
    extra_p_count,
    list_mzone,
    list_szone,
    list_main,
    list_grave,
    list_hand,
    list_remove,
    list_extra
});

impl PlayerInfo {
    pub fn pile(&self, location: u8) -> Option<&Pile> {
        match location {
            LOCATION_DECK => Some(&self.list_main),
            LOCATION_HAND => Some(&self.list_hand),
            LOCATION_GRAVE => Some(&self.list_grave),
            LOCATION_REMOVED => Some(&self.list_remove),
            LOCATION_EXTRA => Some(&self.list_extra),
            _ => None,
        }
    }

    pub fn zone_mut(&mut self, location: u8, sequence: u32) -> Option<&mut Option<CardId>> {
        let sequence = sequence as usize;
        if location == LOCATION_MZONE {
            self.list_mzone.get_mut(sequence)
        } else {
            self.list_szone.get_mut(sequence)
        }
    }
}

// `field_id` changes while effects are only read (effect::is_available), so it is a Cell.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub event_id: u32,
    pub field_id: std::cell::Cell<u32>,
    pub turn_id: i16,
    pub turn_id_by_player: PerPlayer<i16>,
    pub phase: u16,
    pub turn_player: u8,
    pub priorities: PerPlayer<u8>,
    pub can_shuffle: bool,
    pub copy_id: u16,
}

impl Default for FieldInfo {
    fn default() -> Self {
        Self {
            event_id: 1,
            field_id: std::cell::Cell::new(1),
            turn_id: 0,
            turn_id_by_player: PerPlayer::default(),
            phase: 0,
            turn_player: 0,
            priorities: PerPlayer::default(),
            can_shuffle: true,
            copy_id: 1,
        }
    }
}

// The inline sizes of the lists below cover typical duels; longer lists still work but allocate when cloned.
pub type Pile = List<CardId, 48>;
pub type Cards16 = List<CardId, 16>;
pub type Cards8 = List<CardId, 8>;
pub type Events = List<Event, 4>;
// Event lists the step that fills them also empties, so they rarely hold an event at a prompt.
pub type PendingEvents = List<Event, 1>;
pub type FieldIds = List<u32, 8>;
pub type Relations = List<Relation, 16>;
pub type CountCodes = List<(u64, u32), 8>;

// The codes field_check_tribute looks up for the card, its player and the monsters on the field.
const TRIBUTE_CODES: [u32; 10] = [
    EFFECT_ADD_EXTRA_TRIBUTE,
    EFFECT_CANNOT_RELEASE,
    EFFECT_UNRELEASABLE_SUM,
    EFFECT_TRIBUTE_LIMIT,
    EFFECT_TRIPLE_TRIBUTE,
    EFFECT_DOUBLE_TRIBUTE,
    EFFECT_EXTRA_RELEASE,
    EFFECT_EXTRA_RELEASE_SUM,
    EFFECT_MUST_USE_MZONE,
    EFFECT_MAX_MZONE,
];

// The counts of field_check_tribute for its key (sumplayer, toplayer, zone, releasable), which decide it for every min and max.
#[derive(Debug, Clone, Copy)]
pub struct Tributes {
    key: (u8, u8, u32, u32),
    count: i32,
    extra: i32,
    free: i32,
    own: i32,
    freed: i32,
    limit: i32,
}

impl Tributes {
    fn suffice(self, min: i32, max: i32) -> bool {
        let max = max.min(self.count);
        min <= max
            && (self.free > 0 || max > 0)
            && self.free + self.freed > 0
            && self.own > -self.limit
            && (max - self.extra).max(0) > -self.limit
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReturnCards {
    pub canceled: bool,
    pub list: Cards8,
}

impl ReturnCards {
    pub fn clear(&mut self) {
        self.canceled = false;
        self.list.clear();
    }
}

// processor in field.h, reduced to what the port reads so far; `units` keeps its front at the end.
#[derive(Debug, Default)]
pub struct Processor {
    pub units: Vec<Unit>,
    pub subunits: Vec<Unit>,
    // The damage of Duel.Damage in steps, run by Duel.RDComplete.
    pub recover_damage_reserve: Vec<Unit>,
    pub just_sent_cards: CardSet,
    pub select_cards: Cards16,
    pub unselect_cards: Cards16,
    pub select_options: List<u64, 4>,
    // The last toss's results: coins 1 for heads, dice 1 to 6.
    pub coin_results: List<u8, 5>,
    pub dice_results: List<u8, 2>,
    pub select_effects: List<EffectId, 4>,
    // The group a summon procedure of EFFECT_SPSUMMON_PROC_G fills (SpSummonRule's cards_to_summon_g).
    pub cards_to_summon_g: CardSet,
    // The summon, attack and chain counters of action_counter_t, keyed by id like the core's maps.
    pub custom_counters: List<CustomCounter, 2>,
    pub release_cards: CardSet,
    pub release_cards_ex: CardSet,
    pub release_cards_ex_oneof: CardSet,
    pub summonable_cards: Cards8,
    pub spsummonable_cards: Cards8,
    pub repositionable_cards: Cards8,
    pub msetable_cards: Cards8,
    pub ssetable_cards: Cards8,
    pub attackable_cards: Cards8,
    pub point_event: Events,
    pub instant_event: Events,
    pub queue_event: PendingEvents,
    pub delayed_activate_event: Events,
    pub full_event: Events,
    pub single_event: PendingEvents,
    pub solving_event: PendingEvents,
    pub sub_solving_event: PendingEvents,
    pub select_chains: Vec<Chain>,
    // The turn player's chain links the core collects in the order a Processors::SortChain prompt sets (the attack costs).
    pub tpchain: Vec<EffectId>,
    pub current_chain: Vec<Chain>,
    pub real_chain_count: i32,
    pub ignition_priority_chains: Vec<Chain>,
    pub continuous_chain: Vec<Chain>,
    // How often a card's adjustment came again within one adjust_all, for Duel.Readjust and the control adjustment.
    pub readjust_map: std::collections::BTreeMap<CardId, u32>,
    // The continuous effects whose destruction replacement came out in battle; their operations run in step 29.
    pub desrep_chain: Vec<Chain>,
    // The group a Destroy shares with its DestroyReplace and OperationReplace units, innermost last.
    pub replace_targets: Vec<CardSet>,
    pub solving_continuous: Vec<Continuous>,
    pub sub_solving_continuous: Vec<Continuous>,
    pub delayed_continuous_tp: Vec<Continuous>,
    pub delayed_continuous_ntp: Vec<Continuous>,
    // The continuous effects an event triggers for the turn player and the other, kept for their capacity.
    pub continuous_tp: Vec<Continuous>,
    pub continuous_ntp: Vec<Continuous>,
    pub new_fchain: Vec<Chain>,
    pub new_fchain_s: Vec<Chain>,
    pub new_ochain: Vec<Chain>,
    pub new_ochain_s: Vec<Chain>,
    pub new_fchain_b: Vec<Chain>,
    pub new_ochain_b: Vec<Chain>,
    pub new_ochain_h: Vec<Chain>,
    pub new_chains: Vec<Chain>,
    // std::set of (effect, event) in the core, ordered by address; only filled by EFFECT_FLAG_DELAY quick effects.
    pub delayed_quick_tmp: Vec<(EffectId, Event)>,
    pub delayed_quick: Vec<(EffectId, Event)>,
    // std::map from effect to chain in the core, ordered by address; only filled by EFFECT_TYPE_QUICK_F effects.
    pub quick_f_chain: Vec<(EffectId, Chain)>,
    pub chain_limit: Vec<ChainLimit>,
    pub chain_limit_p: Vec<ChainLimit>,
    // What the SendTo units between their steps 4 and 9 keep, innermost last.
    pub send_to_args: Vec<SendToArgs>,
    pub leave_confirmed: CardSet,
    pub special_summoning: CardSet,
    pub equiping_cards: CardSet,
    pub ss_tograve_set: CardSet,
    pub delayed_enable_set: CardSet,
    pub destroy_canceled: CardSet,
    // The cards an EFFECT_DESTROY_SUBSTITUTE destroys in place of those destroyed by battle.
    pub battle_destroy_rep: CardSet,
    // What is left of the cards a battle destroys after BattleCommand step 28's Destroy; the core shares the group between both.
    pub battle_destroyed: CardSet,
    pub operated_set: CardSet,
    pub reseted_effects: List<EffectId, 16>,
    // effect_count_code, effect_count_code_duel and effect_count_code_chain: hash maps in the core, only looked up.
    pub effect_count_code: CountCodes,
    pub effect_count_code_duel: CountCodes,
    pub effect_count_code_chain: CountCodes,
    pub global_flag: u32,
    pub chain_solving: bool,
    pub conti_solving: bool,
    pub conti_player: u8,
    pub check_level: i32,
    pub win_player: u8,
    pub win_reason: u8,
    pub re_adjust: bool,
    pub reason_effect: std::cell::Cell<Option<EffectId>>,
    pub reason_player: std::cell::Cell<u8>,
    pub attacker: Option<CardId>,
    pub attack_target: Option<CardId>,
    pub pre_field: [u32; 2],
    // std::set in the core; kept sorted, since only equality is asked.
    pub opp_mzone: FieldIds,
    pub attack_rollback: bool,
    pub battle_damage: PerPlayer<i32>,
    pub summon_count: PerPlayer<i32>,
    pub extra_summon: PerPlayer<u8>,
    pub spe_effect: PerPlayer<i32>,
    pub duel_options: u64,
    pub to_bp: bool,
    pub to_m2: bool,
    pub to_ep: bool,
    pub skip_m2: bool,
    pub chain_attack: bool,
    pub chain_attacker_id: u32,
    pub chain_attack_target: Option<CardId>,
    pub attack_player: bool,
    pub selfdes_disabled: bool,
    pub unique_destroy_set: CardSet,
    pub self_destroy_set: CardSet,
    pub overdraw: PerPlayer<bool>,
    pub shuffle_check_disabled: bool,
    pub shuffle_hand_check: PerPlayer<bool>,
    pub trap_monster_adjust_set: PerPlayer<CardSet>,
    pub shuffle_deck_check: PerPlayer<bool>,
    pub deck_reversed: bool,
    pub flip_delayed: bool,
    pub damage_calculated: bool,
    pub hand_adjusted: bool,
    pub summon_state_count: PerPlayer<u8>,
    pub normalsummon_state_count: PerPlayer<u8>,
    pub flipsummon_state_count: PerPlayer<u8>,
    pub spsummon_state_count: PerPlayer<u8>,
    pub spsummon_state_count_rst: PerPlayer<u8>,
    pub spsummon_state_count_tmp: PerPlayer<u8>,
    // The codes of cards with Card.SetSPSummonOnce a player special summoned this turn, and how often.
    pub spsummon_once_map: PerPlayer<Vec<(u32, u32)>>,
    pub spsummon_rst: bool,
    pub attack_state_count: PerPlayer<u8>,
    pub battle_phase_count: PerPlayer<u8>,
    pub battled_count: PerPlayer<u8>,
    pub phase_action: bool,
    pub hint_timing: PerPlayer<u32>,
    pub current_player: u8,
    pub force_turn_end: bool,
    pub attack_cancelable: bool,
    pub attack_cost_paid: u8,
    pub summon_depth: u8,
    pub summon_cancelable: bool,
    // A procedure summon a chain link started, run once the link has resolved.
    pub reserved: Option<Box<Unit>>,
    // 1 while Duel.CalculateDamage waits for the chain to end, 2 while the rest of its damage step runs.
    pub effect_damage_step: u8,
    pub summoning_card: Option<CardId>,
    // The group of the SpSummon running, which its failed steps leave.
    pub spsummon_group: CardSet,
    pub fusion_materials: CardSet,
    // The reset the effects registered by Card.CopyEffect get.
    pub copy_reset: u32,
    pub copy_reset_count: u8,
    // Set and cleared by scripts from checks that only read.
    pub must_select_cards: std::cell::Cell<CardSet>,
    // temp.attack and temp.defense of a card while get_attack or get_defense evaluates it: (card, is attack, value).
    pub stat_temp: std::cell::RefCell<List<(CardId, bool, i32), 4>>,
    // temp.base_attack and temp.base_defense of a card while the stat functions evaluate it: (card, is attack, value).
    pub base_stat_temp: std::cell::RefCell<List<(CardId, bool, i32), 4>>,
    // temp.type of a card while get_type evaluates it: (card, value).
    pub type_temp: std::cell::RefCell<List<(CardId, u32), 4>>,
}

clone_by_field!(Processor {
    units,
    subunits,
    recover_damage_reserve,
    just_sent_cards,
    select_cards,
    unselect_cards,
    select_options,
    coin_results,
    dice_results,
    select_effects,
    cards_to_summon_g,
    custom_counters,
    release_cards,
    release_cards_ex,
    release_cards_ex_oneof,
    summonable_cards,
    spsummonable_cards,
    repositionable_cards,
    msetable_cards,
    ssetable_cards,
    attackable_cards,
    point_event,
    instant_event,
    queue_event,
    delayed_activate_event,
    full_event,
    single_event,
    solving_event,
    sub_solving_event,
    select_chains,
    tpchain,
    current_chain,
    real_chain_count,
    ignition_priority_chains,
    continuous_chain,
    readjust_map,
    desrep_chain,
    replace_targets,
    solving_continuous,
    sub_solving_continuous,
    delayed_continuous_tp,
    delayed_continuous_ntp,
    continuous_tp,
    continuous_ntp,
    new_fchain,
    new_fchain_s,
    new_ochain,
    new_ochain_s,
    new_fchain_b,
    new_ochain_b,
    new_ochain_h,
    new_chains,
    delayed_quick_tmp,
    delayed_quick,
    quick_f_chain,
    chain_limit,
    chain_limit_p,
    send_to_args,
    leave_confirmed,
    special_summoning,
    equiping_cards,
    ss_tograve_set,
    delayed_enable_set,
    destroy_canceled,
    battle_destroy_rep,
    battle_destroyed,
    operated_set,
    reseted_effects,
    effect_count_code,
    effect_count_code_duel,
    effect_count_code_chain,
    global_flag,
    chain_solving,
    conti_solving,
    conti_player,
    check_level,
    win_player,
    win_reason,
    re_adjust,
    reason_effect,
    reason_player,
    attacker,
    attack_target,
    pre_field,
    opp_mzone,
    attack_rollback,
    battle_damage,
    summon_count,
    extra_summon,
    spe_effect,
    duel_options,
    to_bp,
    to_m2,
    to_ep,
    skip_m2,
    chain_attack,
    chain_attacker_id,
    chain_attack_target,
    attack_player,
    selfdes_disabled,
    unique_destroy_set,
    self_destroy_set,
    overdraw,
    shuffle_check_disabled,
    shuffle_hand_check,
    trap_monster_adjust_set,
    shuffle_deck_check,
    deck_reversed,
    flip_delayed,
    damage_calculated,
    hand_adjusted,
    summon_state_count,
    normalsummon_state_count,
    flipsummon_state_count,
    spsummon_state_count,
    spsummon_state_count_rst,
    spsummon_state_count_tmp,
    spsummon_once_map,
    spsummon_rst,
    attack_state_count,
    battle_phase_count,
    battled_count,
    phase_action,
    hint_timing,
    current_player,
    force_turn_end,
    attack_cancelable,
    attack_cost_paid,
    summon_depth,
    summon_cancelable,
    reserved,
    effect_damage_step,
    summoning_card,
    spsummon_group,
    fusion_materials,
    copy_reset,
    copy_reset_count,
    must_select_cards,
    stat_temp,
    base_stat_temp,
    type_temp
});

impl Processor {
    pub fn new(duel_options: u64) -> Self {
        Self {
            duel_options,
            conti_player: PLAYER_NONE,
            win_player: 5,
            reason_player: std::cell::Cell::new(PLAYER_NONE),
            ..Self::default()
        }
    }

    // Every list of chain links but `quick_f_chain`, which pairs each with its effect; `select_chains` first.
    pub(crate) fn chain_lists(&self) -> [&[Chain]; 13] {
        [
            &self.select_chains,
            &self.current_chain,
            &self.new_chains,
            &self.ignition_priority_chains,
            &self.continuous_chain,
            &self.desrep_chain,
            &self.new_fchain,
            &self.new_fchain_s,
            &self.new_ochain,
            &self.new_ochain_s,
            &self.new_fchain_b,
            &self.new_ochain_b,
            &self.new_ochain_h,
        ]
    }

    pub(crate) fn chain_lists_mut(&mut self) -> [&mut [Chain]; 13] {
        [
            &mut self.select_chains,
            &mut self.current_chain,
            &mut self.new_chains,
            &mut self.ignition_priority_chains,
            &mut self.continuous_chain,
            &mut self.desrep_chain,
            &mut self.new_fchain,
            &mut self.new_fchain_s,
            &mut self.new_ochain,
            &mut self.new_ochain_s,
            &mut self.new_fchain_b,
            &mut self.new_ochain_b,
            &mut self.new_ochain_h,
        ]
    }

    // Every list of events but the delayed quick effects', which pair each with its effect.
    pub(crate) fn event_lists(&self) -> [&[Event]; 8] {
        [
            &self.point_event,
            &self.instant_event,
            &self.queue_event,
            &self.delayed_activate_event,
            &self.full_event,
            &self.single_event,
            &self.solving_event,
            &self.sub_solving_event,
        ]
    }

    pub(crate) fn event_lists_mut(&mut self) -> [&mut [Event]; 8] {
        [
            &mut self.point_event,
            &mut self.instant_event,
            &mut self.queue_event,
            &mut self.delayed_activate_event,
            &mut self.full_event,
            &mut self.single_event,
            &mut self.solving_event,
            &mut self.sub_solving_event,
        ]
    }

    pub(crate) fn continuous_lists(&self) -> [&[Continuous]; 6] {
        [
            &self.solving_continuous,
            &self.sub_solving_continuous,
            &self.delayed_continuous_tp,
            &self.delayed_continuous_ntp,
            &self.continuous_tp,
            &self.continuous_ntp,
        ]
    }

    pub(crate) fn continuous_lists_mut(&mut self) -> [&mut [Continuous]; 6] {
        [
            &mut self.solving_continuous,
            &mut self.sub_solving_continuous,
            &mut self.delayed_continuous_tp,
            &mut self.delayed_continuous_ntp,
            &mut self.continuous_tp,
            &mut self.continuous_ntp,
        ]
    }
}

fn field_used_count(flag: u32) -> i32 {
    (flag & 0x1f).count_ones() as i32
}

fn count_map_key(code: u32, flag: u8, hopt_index: u8, playerid: u8) -> u64 {
    (u64::from(code) << 32) | u64::from((u32::from(hopt_index) << 16) | (u32::from(flag) << 8) | u32::from(playerid))
}

impl Duel {
    pub(crate) fn is_flag(&self, flag: u64) -> bool {
        self.core.duel_options & flag == flag
    }

    pub(crate) fn get_extra_deck_types(&self) -> u32 {
        let types = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;
        if self.is_flag(DUEL_EXTRA_DECK_RITUAL) {
            types | TYPE_RITUAL
        } else {
            types
        }
    }

    pub(crate) fn get_pzone_index(&self, sequence: u32) -> u32 {
        if sequence > 1 {
            0
        } else if self.is_flag(DUEL_SEPARATE_PZONE) {
            sequence + 6
        } else if self.is_flag(DUEL_3_COLUMNS_FIELD) {
            sequence * 2 + 1
        } else {
            sequence * 4
        }
    }

    pub(crate) fn list_mut(&mut self, playerid: u8, location: u8) -> Option<&mut Pile> {
        let player = &mut self.player[playerid];
        match location {
            LOCATION_DECK => Some(&mut player.list_main),
            LOCATION_HAND => Some(&mut player.list_hand),
            LOCATION_GRAVE => Some(&mut player.list_grave),
            LOCATION_REMOVED => Some(&mut player.list_remove),
            LOCATION_EXTRA => Some(&mut player.list_extra),
            _ => None,
        }
    }

    // The core of moving cards; checks the rule redirection of Fusion/Synchro/Xyz/Link monsters and sets fieldid_r.
    pub(crate) fn add_card(&mut self, playerid: u8, card: CardId, location: u8, sequence: u8, pzone: bool) {
        if self.cards[card].current.location != 0 || !self.is_location_useable(playerid, u16::from(location), u32::from(sequence)) {
            return;
        }
        let mut location = location;
        let card_type = self.cards[card].data.card_type;
        // Fusion spell cards may start in the Extra Deck.
        if self.is_extra_deck_monster(card) || card_type & TYPE_FUSION != 0 {
            if location & (LOCATION_HAND | LOCATION_DECK) != 0 {
                location = LOCATION_EXTRA;
                self.cards[card].sendto_param.position = POS_FACEDOWN_DEFENSE as u8;
            }
        } else if location == LOCATION_EXTRA
            && (card_type & TYPE_PENDULUM == 0 || u32::from(self.cards[card].sendto_param.position) & POS_FACEUP == 0)
        {
            location = LOCATION_DECK;
            self.cards[card].sendto_param.position = POS_FACEDOWN_DEFENSE as u8;
        }
        self.cards[card].current.controler = playerid;
        self.cards[card].current.location = location;
        let player = &mut self.player[playerid];
        match location {
            LOCATION_MZONE | LOCATION_SZONE => {
                if let Some(slot) = player.zone_mut(location, u32::from(sequence)) {
                    *slot = Some(card);
                }
                self.cards[card].current.sequence = u32::from(sequence);
            }
            LOCATION_DECK => {
                if sequence == 1 {
                    player.list_main.insert(0, card);
                    self.reset_sequence(playerid, LOCATION_DECK);
                } else {
                    player.list_main.push(card);
                    self.cards[card].current.sequence = (player.list_main.len() - 1) as u32;
                    if sequence != 0 && !self.core.shuffle_check_disabled {
                        self.core.shuffle_deck_check[playerid] = true;
                    }
                }
                self.cards[card].sendto_param.position = POS_FACEDOWN as u8;
            }
            LOCATION_HAND => {
                player.list_hand.push(card);
                self.cards[card].current.sequence = (player.list_hand.len() - 1) as u32;
                let public = self.is_affected_by_effect(card, EFFECT_PUBLIC).is_some();
                self.cards[card].sendto_param.position = if public { POS_FACEUP as u8 } else { POS_FACEDOWN as u8 };
                if self.cards[card].current.reason & REASON_DRAW == 0 && !self.core.shuffle_check_disabled {
                    self.core.shuffle_hand_check[playerid] = true;
                }
            }
            LOCATION_GRAVE => {
                player.list_grave.push(card);
                self.cards[card].current.sequence = (player.list_grave.len() - 1) as u32;
            }
            LOCATION_REMOVED => {
                player.list_remove.push(card);
                self.cards[card].current.sequence = (player.list_remove.len() - 1) as u32;
            }
            LOCATION_EXTRA => {
                let faceup_pendulum = card_type & TYPE_PENDULUM != 0 && u32::from(self.cards[card].sendto_param.position) & POS_FACEUP != 0;
                if player.extra_p_count == 0 || faceup_pendulum {
                    player.list_extra.push(card);
                } else {
                    let index = player.list_extra.len().saturating_sub(player.extra_p_count as usize);
                    player.list_extra.insert(index, card);
                }
                if faceup_pendulum {
                    player.extra_p_count += 1;
                }
                self.reset_sequence(playerid, LOCATION_EXTRA);
            }
            _ => {}
        }
        self.cards[card].current.pzone = pzone;
        self.apply_field_effect(card);
        let field_id = self.next_field_id();
        let turn_id = self.infos.turn_id as u16;
        let target = &mut self.cards[card];
        target.fieldid = field_id;
        target.fieldid_r = field_id;
        target.turnid = turn_id;
        self.mark_unique_violation(card);
        if location == LOCATION_MZONE {
            self.player[playerid].used_location |= 1 << sequence;
        }
        if location == LOCATION_SZONE {
            self.player[playerid].used_location |= 256 << sequence;
        }
    }

    pub(crate) fn remove_card(&mut self, card: CardId) {
        let current = self.cards[card].current;
        if current.controler == PLAYER_NONE || current.location == 0 {
            return;
        }
        let playerid = current.controler;
        let sequence = current.sequence as usize;
        match current.location {
            LOCATION_MZONE | LOCATION_SZONE => {
                if let Some(slot) = self.player[playerid].zone_mut(current.location, current.sequence) {
                    *slot = None;
                }
            }
            location => {
                if let Some(list) = self.list_mut(playerid, location).filter(|list| sequence < list.len()) {
                    list.remove(sequence);
                }
                self.reset_sequence(playerid, location);
                if location == LOCATION_DECK && !self.core.shuffle_check_disabled {
                    self.core.shuffle_deck_check[playerid] = true;
                }
                if location == LOCATION_EXTRA && self.cards[card].data.card_type & TYPE_PENDULUM != 0 && current.position & POS_FACEUP != 0
                {
                    self.player[playerid].extra_p_count = self.player[playerid].extra_p_count.wrapping_sub(1);
                }
            }
        }
        self.cancel_field_effect(card);
        if current.location == LOCATION_MZONE {
            self.player[playerid].used_location &= !(1 << current.sequence);
        }
        if current.location == LOCATION_SZONE {
            self.player[playerid].used_location &= !(256 << current.sequence);
        }
        if !self.core.current_chain.is_empty() {
            self.core.just_sent_cards.insert(card);
        }
        let target = &mut self.cards[card];
        target.previous.set_place(current);
        target.current.controler = PLAYER_NONE;
        target.current.location = 0;
        target.current.sequence = 0;
    }

    // Checks the rule redirection of Fusion/Synchro/Xyz/Link monsters.
    pub(crate) fn move_card(&mut self, playerid: u8, card: CardId, location: u8, sequence: u8, pzone: bool) -> bool {
        if !self.is_location_useable(playerid, u16::from(location), u32::from(sequence)) {
            return false;
        }
        let mut location = location;
        let current = self.cards[card].current;
        let (preplayer, presequence) = (current.controler, current.sequence);
        if self.is_extra_deck_monster(card) && location & (LOCATION_HAND | LOCATION_DECK) != 0 {
            location = LOCATION_EXTRA;
            self.cards[card].sendto_param.position = POS_FACEDOWN_DEFENSE as u8;
        }
        if current.location != 0 {
            if current.location == location && current.pzone == pzone {
                if current.location == LOCATION_DECK {
                    if preplayer != playerid {
                        self.remove_card(card);
                        self.add_card(playerid, card, location, sequence, pzone);
                        return true;
                    }
                    let from = self.cards.get_info_location(card);
                    let list = &mut self.player[preplayer].list_main;
                    if (presequence as usize) < list.len() {
                        list.remove(presequence as usize);
                    }
                    if sequence == 1 {
                        list.insert(0, card);
                    } else {
                        list.push(card);
                        if sequence != 0 && !self.core.shuffle_check_disabled {
                            self.core.shuffle_deck_check[playerid] = true;
                        }
                    }
                    self.reset_sequence(playerid, LOCATION_DECK);
                    self.cards[card].previous.controler = preplayer;
                    self.cards[card].current.controler = playerid;
                    self.write_move(card, from);
                    return true;
                }
                if location & LOCATION_ONFIELD != 0 {
                    if playerid == preplayer && u32::from(sequence) == presequence {
                        return false;
                    }
                    if self.player[playerid]
                        .zone_mut(location, u32::from(sequence))
                        .is_none_or(|slot| slot.is_some())
                    {
                        return false;
                    }
                    let from = (preplayer == playerid).then(|| self.cards.get_info_location(card));
                    if !self.core.current_chain.is_empty() {
                        self.core.just_sent_cards.insert(card);
                    }
                    self.cards[card].previous.set_place(current);
                    let shift = if location == LOCATION_MZONE { 0 } else { 8 };
                    if let Some(slot) = self.player[preplayer].zone_mut(location, presequence) {
                        *slot = None;
                    }
                    self.player[preplayer].used_location &= !(1 << (presequence + shift));
                    if let Some(slot) = self.player[playerid].zone_mut(location, u32::from(sequence)) {
                        *slot = Some(card);
                    }
                    self.player[playerid].used_location |= 1 << (u32::from(sequence) + shift);
                    self.cards[card].current.controler = playerid;
                    self.cards[card].current.sequence = u32::from(sequence);
                    match from {
                        Some(from) => self.write_move(card, from),
                        None => {
                            self.cards[card].fieldid = self.next_field_id();
                            self.mark_unique_violation(card);
                        }
                    }
                    return true;
                }
                if location == LOCATION_HAND {
                    if preplayer == playerid {
                        return false;
                    }
                    self.remove_card(card);
                } else {
                    let length = self.list_mut(preplayer, location).map_or(0, |list| list.len());
                    if location != LOCATION_EXTRA && presequence as usize + 1 == length {
                        return false;
                    }
                    let from = self.cards.get_info_location(card);
                    if let Some(list) = self.list_mut(preplayer, location) {
                        if (presequence as usize) < list.len() {
                            list.remove(presequence as usize);
                        }
                        list.push(card);
                    }
                    self.reset_sequence(preplayer, location);
                    self.write_move(card, from);
                    return true;
                }
            } else {
                let from_field = (current.location == LOCATION_MZONE && !self.cards[card].is_status(STATUS_SUMMON_DISABLED))
                    || (current.location == LOCATION_SZONE && !self.cards[card].is_status(STATUS_ACTIVATE_DISABLED));
                if self.cards[card].data.card_type & TYPE_PENDULUM != 0
                    && location == LOCATION_GRAVE
                    && self.is_capable_send_to_extra(card, playerid)
                    && from_field
                {
                    location = LOCATION_EXTRA;
                    self.cards[card].sendto_param.position = POS_FACEUP_DEFENSE as u8;
                }
                self.remove_card(card);
            }
        }
        self.add_card(playerid, card, location, sequence, pzone);
        true
    }

    // field::swap_card(pcard1, pcard2): two cards of one zone kind swap places (Duel.SwapSequence checks the kind).
    pub(crate) fn swap_card(&mut self, card1: CardId, card2: CardId) {
        let (s1, s2) = (self.cards[card1].current.sequence, self.cards[card2].current.sequence);
        self.swap_card_to(card1, card2, s1, s2);
    }

    // field::swap_card(pcard1, pcard2, new_sequence1, new_sequence2); swapping cards of different locations is not ported.
    pub(crate) fn swap_card_to(&mut self, card1: CardId, card2: CardId, new1: u32, new2: u32) {
        let (current1, current2) = (self.cards[card1].current, self.cards[card2].current);
        let (p1, p2, location) = (current1.controler, current2.controler, current1.location);
        let (s1, s2) = (current1.sequence, current2.sequence);
        let (info1, info2) = (self.cards.get_info_location(card1), self.cards.get_info_location(card2));
        if location & LOCATION_ONFIELD == 0 || current2.location != location {
            return;
        }
        if (new1 != s1 && !self.is_location_useable(p1, u16::from(location), new1))
            || (new2 != s2 && !self.is_location_useable(p2, u16::from(location), new2))
            || (p1 == p2 && (new1 == s2 || new2 == s1))
        {
            return;
        }
        for (card, current, controler, sequence) in [(card1, current1, p2, new2), (card2, current2, p1, new1)] {
            let target = &mut self.cards[card];
            target.previous.set_place(current);
            target.current.controler = controler;
            target.current.sequence = sequence;
        }
        if p1 != p2 {
            for card in [card1, card2] {
                self.cards[card].fieldid = self.next_field_id();
                self.mark_unique_violation(card);
            }
        }
        let shift = if location == LOCATION_MZONE { 0 } else { 8 };
        for (p, sequence) in [(p1, s1), (p2, s2)] {
            if let Some(slot) = self.player[p].zone_mut(location, sequence) {
                *slot = None;
            }
            self.player[p].used_location &= !(1 << (sequence + shift));
        }
        for (p, sequence, card) in [(p2, new2, card1), (p1, new1, card2)] {
            if let Some(slot) = self.player[p].zone_mut(location, sequence) {
                *slot = Some(card);
            }
            self.player[p].used_location |= 1 << (sequence + shift);
        }
        let (code1, code2) = (self.cards[card1].data.code, self.cards[card2].data.code);
        if s1 == new1 && s2 == new2 {
            self.trace.message(MSG_SWAP).u32(code1).loc(info1).u32(code2).loc(info2);
            return;
        }
        let moves = [(code1, info1, card1), (code2, info2, card2)];
        for (code, from, card) in if s1 == new1 { moves } else { [moves[1], moves[0]] } {
            let to = self.cards.get_info_location(card);
            self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(0);
        }
    }

    // The MSG_MOVE that move_card writes when a card changes place within one location.
    pub(crate) fn write_move(&mut self, card: CardId, from: LocInfo) {
        let (code, to, reason) = (
            self.cards[card].data.code,
            self.cards.get_info_location(card),
            self.cards[card].current.reason,
        );
        self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(reason);
    }

    // Whether the zone (or any non-zone location) is free and not disabled.
    pub(crate) fn is_location_useable(&self, playerid: u8, location: u16, sequence: u32) -> bool {
        self.zone_useable(
            |p| self.player[p].disabled_location | self.player[p].used_location,
            playerid,
            location,
            sequence,
        )
    }

    pub(crate) fn zones(&self) -> Zones {
        let player = |p: u8| &self.player[p];
        Zones {
            used: PerPlayer([0, 1].map(|p| player(p).used_location)),
            disabled: PerPlayer([0, 1].map(|p| player(p).disabled_location)),
            mzone: PerPlayer([0, 1].map(|p| player(p).list_mzone)),
            placed: None,
        }
    }

    pub(crate) fn zones_without(&self, excluded: CardSet) -> Zones {
        let mut zones = self.zones();
        for p in 0..2 {
            for (sequence, card) in zones.mzone[p].iter_mut().enumerate() {
                if card.is_some_and(|card| excluded.contains(card)) {
                    zones.used[p] &= !(1 << sequence);
                    *card = None;
                }
            }
        }
        zones
    }

    pub(crate) fn is_location_useable_in(&self, zones: &Zones, playerid: u8, location: u16, sequence: u32) -> bool {
        self.zone_useable(|p| zones.disabled[p] | zones.used[p], playerid, location, sequence)
    }

    fn zone_useable(&self, blocked: impl Fn(u8) -> u32, playerid: u8, location: u16, sequence: u32) -> bool {
        let flag = blocked(playerid);
        let three_columns = u32::from(self.is_flag(DUEL_3_COLUMNS_FIELD));
        let (location, sequence) = match location {
            LOCATION_EMZONE => (u16::from(LOCATION_MZONE), sequence + 5),
            LOCATION_MMZONE => (u16::from(LOCATION_MZONE), sequence + three_columns),
            LOCATION_STZONE => (u16::from(LOCATION_SZONE), sequence + three_columns),
            _ => (location, sequence),
        };
        let bit = |shift: u32| 1u32.checked_shl(shift).unwrap_or(0);
        if location == u16::from(LOCATION_MZONE) {
            if flag & bit(sequence) != 0 {
                return false;
            }
            if sequence >= 5 {
                let opponent = blocked(1 - playerid);
                if opponent & bit(11u32.wrapping_sub(sequence)) != 0 {
                    return false;
                }
            }
        } else if location == u16::from(LOCATION_SZONE) {
            return flag & bit(8 + sequence) == 0;
        } else if location == LOCATION_FZONE {
            return flag & bit(8 + 5 + sequence) == 0;
        } else if location == LOCATION_PZONE {
            return self.is_flag(DUEL_PZONE) && flag & bit(8 + self.get_pzone_index(sequence)) == 0;
        }
        true
    }

    // The usable zone count (may be negative) and blocked zones of playerid for uplayer; PLAYER_NONE ignores EFFECT_MAX_MZONE/SZONE.
    pub(crate) fn get_useable_count(
        &self,
        card: Option<CardId>,
        playerid: u8,
        location: u8,
        uplayer: u8,
        reason: u32,
        zone: u32,
    ) -> (i32, u32) {
        let zones = self.zones();
        if location == LOCATION_MZONE && card.is_some_and(|card| self.cards[card].current.location == LOCATION_EXTRA) {
            return self.get_useable_count_fromex_in(&zones, FromEx::card(self, card), playerid, uplayer, zone);
        }
        self.get_useable_count_other_in(&zones, card, playerid, location, uplayer, reason, zone)
    }

    pub(crate) fn get_useable_count_fromex_in(&self, zones: &Zones, summoned: FromEx, playerid: u8, uplayer: u8, zone: u32) -> (i32, u32) {
        if !self.is_flag(DUEL_EMZONE) {
            let card = summoned.card;
            return self.get_useable_count_other_in(zones, card, playerid, LOCATION_MZONE, uplayer, LOCATION_REASON_TOFIELD, zone);
        }
        let (count, flag) = self.get_spsummonable_count_fromex_in(zones, summoned, playerid, uplayer, zone);
        (
            count.min(self.get_mzone_limit_in(zones, playerid, uplayer, LOCATION_REASON_TOFIELD)),
            flag,
        )
    }

    pub(crate) fn get_spsummonable_count_fromex(&self, card: Option<CardId>, playerid: u8, uplayer: u8, zone: u32) -> (i32, u32) {
        self.get_spsummonable_count_fromex_in(&self.zones(), FromEx::card(self, card), playerid, uplayer, zone)
    }

    fn get_spsummonable_count_fromex_in(&self, zones: &Zones, summoned: FromEx, playerid: u8, uplayer: u8, zone: u32) -> (i32, u32) {
        let card = summoned.card;
        if !self.is_flag(DUEL_EMZONE) {
            return self.get_tofield_count_in(zones, card, playerid, LOCATION_MZONE, uplayer, LOCATION_REASON_TOFIELD, zone);
        }
        let mut flag = zones.disabled[playerid] | zones.used[playerid];
        flag |= !self.get_forced_zones(card, playerid, LOCATION_MZONE, uplayer, LOCATION_REASON_TOFIELD);
        let [extra5, extra6] = [5, 6].map(|sequence| zones.mzone[playerid].get(sequence).is_some_and(Option::is_some));
        let mzone = u16::from(LOCATION_MZONE);
        if extra5 && self.is_location_useable_in(zones, playerid, mzone, 6) && self.check_extra_link(zones, playerid, summoned, 6) {
            flag |= 1 << 5;
        } else if extra6 && self.is_location_useable_in(zones, playerid, mzone, 5) && self.check_extra_link(zones, playerid, summoned, 5) {
            flag |= 1 << 6;
        } else if extra5 || extra6 {
            flag |= (1 << 5) | (1 << 6);
        } else {
            for sequence in [5, 6] {
                if !self.is_location_useable_in(zones, playerid, mzone, sequence) {
                    flag |= 1 << sequence;
                }
            }
        }
        flag = flag | !zone | !self.get_rule_zone_fromex(zones, playerid, summoned);
        let mut count = 5 - field_used_count(flag & 0x1f);
        if !flag & ((1 << 5) | (1 << 6)) != 0 {
            count += 1;
        }
        (count, flag & 0x7f)
    }

    fn get_rule_zone_fromex(&self, zones: &Zones, playerid: u8, summoned: FromEx) -> u32 {
        if !self.is_flag(DUEL_EMZONE) {
            return 0x1f;
        }
        let fusion_synchro_xyz = summoned.card_type & (self.get_extra_deck_types() & !TYPE_LINK) != 0;
        if self.is_flag(DUEL_FSX_MMZONE) && summoned.facedown && fusion_synchro_xyz {
            0x7f
        } else {
            self.get_linked_zone(zones, playerid, false) | (1 << 5) | (1 << 6)
        }
    }

    // field::get_linked_zone without `free`: the zones Link monsters point to, the opponent's mirrored into the high half.
    pub(crate) fn get_linked_zone(&self, zones: &Zones, playerid: u8, actually_linked: bool) -> u32 {
        let pointed = |p: u8| {
            let mzone = zones.mzone[p].into_iter().flatten();
            mzone
                .chain(self.player[p].list_szone.into_iter().flatten())
                .fold(0, |pointed, card| pointed | self.card_linked_zone(zones, card))
        };
        let opponent = pointed(1 - playerid);
        if !actually_linked {
            unported!(self, self.filter_field_effect(EFFECT_BECOME_LINKED_ZONE));
        }
        pointed(playerid) | ((opponent & 0xffff) << 16) | (opponent >> 16)
    }

    // A card's controller, location, sequence and position, or where the core places it for check_extra_link.
    fn link_state(&self, zones: &Zones, card: CardId) -> (u8, u8, u32, u32) {
        match zones.placed {
            Some((placed, controler, sequence)) if placed == card => (controler, LOCATION_MZONE, sequence, POS_FACEUP_ATTACK),
            _ => {
                let current = self.cards[card].current;
                (current.controler, current.location, current.sequence, current.position)
            }
        }
    }

    // The type card::get_link_marker reads, of a card the core may have placed for a check.
    fn link_type(&self, zones: &Zones, card: CardId) -> u32 {
        match zones.placed {
            Some((placed, controler, sequence)) if placed == card => self.get_type_placed(zones, card, controler, sequence),
            _ => self.get_type(card),
        }
    }

    // card::get_link_marker: rotated a quarter turn in defense position.
    pub(crate) fn get_link_marker(&self, zones: &Zones, card: CardId) -> u32 {
        let (_, location, _, position) = self.link_state(zones, card);
        let on_field = location & LOCATION_ONFIELD != 0;
        if (position & POS_FACEDOWN != 0 && on_field) || self.link_type(zones, card) & TYPE_LINK == 0 {
            return 0;
        }
        unported!(
            self,
            self.filter_effects(card, &[EFFECT_ADD_LINKMARKER, EFFECT_REMOVE_LINKMARKER, EFFECT_CHANGE_LINKMARKER])
        );
        let marker = self.cards[card].data.link_marker;
        if position & POS_ATTACK != 0 || !on_field {
            return marker;
        }
        [
            (LINK_MARKER_BOTTOM_LEFT, LINK_MARKER_BOTTOM_RIGHT),
            (LINK_MARKER_BOTTOM, LINK_MARKER_RIGHT),
            (LINK_MARKER_BOTTOM_RIGHT, LINK_MARKER_TOP_RIGHT),
            (LINK_MARKER_RIGHT, LINK_MARKER_TOP),
            (LINK_MARKER_TOP_RIGHT, LINK_MARKER_TOP_LEFT),
            (LINK_MARKER_TOP, LINK_MARKER_LEFT),
            (LINK_MARKER_TOP_LEFT, LINK_MARKER_BOTTOM_LEFT),
            (LINK_MARKER_LEFT, LINK_MARKER_BOTTOM),
        ]
        .into_iter()
        .filter(|&(from, _)| marker & from != 0)
        .fold(0, |rotated, (_, to)| rotated | to)
    }

    // card::get_linked_zone without `free`: bit s the controller's monster zone s, 8 + s their spell zone, 16 up the opponent's.
    pub(crate) fn card_linked_zone(&self, zones: &Zones, card: CardId) -> u32 {
        let (_, location, s, _) = self.link_state(zones, card);
        if self.link_type(zones, card) & TYPE_LINK == 0
            || location & LOCATION_ONFIELD == 0
            || self.cards[card].get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP)
        {
            return 0;
        }
        let marker = self.get_link_marker(zones, card);
        let has = |direction: u32| marker & direction != 0;
        let mut linked = 0u32;
        if location == LOCATION_MZONE {
            if (1..=4).contains(&s) && has(LINK_MARKER_LEFT) {
                linked |= 1 << (s - 1);
            }
            if s <= 3 && has(LINK_MARKER_RIGHT) {
                linked |= 1 << (s + 1);
            }
            if s < 5 {
                if s > 0 && has(LINK_MARKER_BOTTOM_LEFT) {
                    linked |= 1 << (s + 7);
                }
                if s < 4 && has(LINK_MARKER_BOTTOM_RIGHT) {
                    linked |= 1 << (s + 9);
                }
                if has(LINK_MARKER_BOTTOM) {
                    linked |= 1 << (s + 8);
                }
            }
            if self.is_flag(DUEL_EMZONE) {
                if (s == 0 && has(LINK_MARKER_TOP_RIGHT)) || (s == 1 && has(LINK_MARKER_TOP)) || (s == 2 && has(LINK_MARKER_TOP_LEFT)) {
                    linked |= (1 << 5) | (1 << (16 + 6));
                }
                if (s == 2 && has(LINK_MARKER_TOP_RIGHT)) || (s == 3 && has(LINK_MARKER_TOP)) || (s == 4 && has(LINK_MARKER_TOP_LEFT)) {
                    linked |= (1 << 6) | (1 << (16 + 5));
                }
                let extra = match s {
                    5 => [0, 1, 2, 16 + 4, 16 + 3, 16 + 2],
                    6 => [2, 3, 4, 16 + 2, 16 + 1, 16],
                    _ => [32; 6],
                };
                let directions = [
                    LINK_MARKER_BOTTOM_LEFT,
                    LINK_MARKER_BOTTOM,
                    LINK_MARKER_BOTTOM_RIGHT,
                    LINK_MARKER_TOP_LEFT,
                    LINK_MARKER_TOP,
                    LINK_MARKER_TOP_RIGHT,
                ];
                for (direction, bit) in directions.into_iter().zip(extra) {
                    if has(direction) {
                        linked |= 1u32.checked_shl(bit).unwrap_or(0);
                    }
                }
            }
        } else if location == LOCATION_SZONE {
            if s > 4 {
                return 0;
            }
            if s > 0 {
                if has(LINK_MARKER_LEFT) {
                    linked |= 1 << (s + 7);
                }
                if has(LINK_MARKER_TOP_LEFT) {
                    linked |= 1 << (s - 1);
                }
            }
            if s < 4 {
                if has(LINK_MARKER_RIGHT) {
                    linked |= 1 << (s + 9);
                }
                if has(LINK_MARKER_TOP_RIGHT) {
                    linked |= 1 << (s + 1);
                }
            }
            if has(LINK_MARKER_TOP) {
                linked |= 1 << s;
            }
        }
        linked
    }

    // The card in a zone bit of card_linked_zone, seen from `controler`.
    fn card_in_zone(&self, zones: &Zones, controler: u8, bit: u32) -> Option<CardId> {
        let (player, bit) = if bit < 16 { (controler, bit) } else { (1 - controler, bit - 16) };
        let index = bit as usize;
        if index < 8 {
            zones.mzone[player].get(index).copied().flatten()
        } else {
            self.player[player].list_szone.get(index - 8).copied().flatten()
        }
    }

    // card::get_mutual_linked_zone: the linked cards (Link monsters only in the spell zones) that point back.
    pub(crate) fn mutual_linked_zone(&self, zones: &Zones, card: CardId) -> u32 {
        let linked = self.card_linked_zone(zones, card);
        if linked == 0 {
            return 0;
        }
        let (controler, location, sequence, _) = self.link_state(zones, card);
        let mut mutual = 0;
        for bit in (0..32).filter(|bit| linked & (1 << bit) != 0) {
            let Some(other) = self.card_in_zone(zones, controler, bit) else {
                continue;
            };
            let (other_controler, other_location, _, _) = self.link_state(zones, other);
            if other_location == LOCATION_SZONE && self.get_type(other) & TYPE_LINK == 0 {
                continue;
            }
            let is_szone = if location == LOCATION_SZONE { 8 } else { 0 };
            let is_player = if controler == other_controler { 0 } else { 16 };
            if self.card_linked_zone(zones, other) & (1 << (sequence + is_szone + is_player)) != 0 {
                mutual |= 1 << bit;
            }
        }
        mutual
    }

    // card::is_extra_link_state: whether a chain of mutual links joins the two Extra Monster Zones.
    fn is_extra_link_state(&self, zones: &Zones, card: CardId) -> bool {
        let (controler, location, sequence, _) = self.link_state(zones, card);
        if location != LOCATION_MZONE {
            return false;
        }
        let mut checked = 1u32 << sequence;
        let mut linked = self.mutual_linked_zone(zones, card);
        loop {
            if ((linked >> 5) | (linked >> (16 + 6))) & ((linked >> 6) | (linked >> (16 + 5))) & 1 != 0 {
                return true;
            }
            let checking = linked & !checked;
            if checking == 0 {
                return false;
            }
            let rightmost = checking.isolate_lowest_one();
            checked |= rightmost;
            let bit = rightmost.trailing_zeros();
            let Some(other) = self.card_in_zone(zones, controler, bit) else {
                continue;
            };
            let zone = self.mutual_linked_zone(zones, other);
            linked |= if bit < 16 { zone } else { ((zone & 0xffff) << 16) | (zone >> 16) };
        }
    }

    // field::check_extra_link: whether the summoned monster, placed in the free Extra Monster Zone `sequence`, links both.
    fn check_extra_link(&self, zones: &Zones, playerid: u8, summoned: FromEx, sequence: usize) -> bool {
        let Some(card) = summoned.card else { return false };
        if zones.mzone[playerid].get(sequence).copied().flatten().is_some() {
            return false;
        }
        let mut placed = *zones;
        if let Some(slot) = placed.mzone[playerid].get_mut(sequence) {
            *slot = Some(card);
        }
        placed.placed = Some((card, playerid, sequence as u32));
        self.is_extra_link_state(&placed, card)
    }

    pub(crate) fn field_card(&self, playerid: u8, location: u16, sequence: u32) -> Option<CardId> {
        let player = &self.player[playerid];
        let sequence = sequence as usize;
        let slot = match location {
            LOCATION_MMZONE if sequence < 5 => player.list_mzone.get(sequence),
            LOCATION_EMZONE => player.list_mzone.get(sequence + 5),
            LOCATION_STZONE if sequence < 5 => player.list_szone.get(sequence),
            LOCATION_FZONE if sequence == 0 => player.list_szone.get(5),
            LOCATION_PZONE if sequence < 2 => {
                let card = player
                    .list_szone
                    .get(self.get_pzone_index(sequence as u32) as usize)
                    .copied()
                    .flatten();
                return card.filter(|&card| self.cards[card].current.pzone);
            }
            location if location == u16::from(LOCATION_MZONE) => player.list_mzone.get(sequence),
            location if location == u16::from(LOCATION_SZONE) => player.list_szone.get(sequence),
            location => return player.pile(location as u8)?.get(sequence).copied(),
        };
        slot.copied().flatten()
    }

    // filter_field_card: the cards of Duel.GetFieldGroup.
    pub(crate) fn filter_field_card(&self, own: u8, location1: u32, location2: u32) -> CardSet {
        let mut group = CardSet::default();
        if own > 1 {
            return group;
        }
        let outer = usize::from(self.is_flag(DUEL_3_COLUMNS_FIELD));
        let mut player = own;
        for location in [location1, location2] {
            let info = &self.player[player];
            let zones = location as u16;
            let mut add = |cards: &[Option<CardId>]| cards.iter().flatten().for_each(|&card| group.insert(card));
            if zones & u16::from(LOCATION_MZONE) != 0 {
                let summoning = |card: CardId| self.cards[card].get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP);
                add(&info.list_mzone.map(|card| card.filter(|&card| !summoning(card))));
            } else {
                if zones & LOCATION_MMZONE != 0 {
                    add(info.list_mzone.get(outer..5 - outer).unwrap_or_default());
                }
                if zones & LOCATION_EMZONE != 0 {
                    add(info.list_mzone.get(5..).unwrap_or_default());
                }
            }
            if zones & u16::from(LOCATION_SZONE) != 0 {
                add(&info.list_szone);
            } else {
                if zones & LOCATION_STZONE != 0 {
                    add(info.list_szone.get(outer..5 - outer).unwrap_or_default());
                }
                if zones & LOCATION_FZONE != 0 {
                    add(info.list_szone.get(5..6).unwrap_or_default());
                }
                if zones & LOCATION_PZONE != 0 {
                    let pzones = [0, 1].map(|index| info.list_szone.get(self.get_pzone_index(index) as usize).copied().flatten());
                    add(&pzones.map(|card| card.filter(|&card| self.cards[card].current.pzone)));
                }
            }
            for bit in [LOCATION_HAND, LOCATION_DECK, LOCATION_EXTRA, LOCATION_GRAVE, LOCATION_REMOVED] {
                if location & u32::from(bit) != 0 {
                    info.pile(bit).into_iter().flatten().for_each(|&card| group.insert(card));
                }
            }
            player = 1 - player;
        }
        group
    }

    pub(crate) fn get_useable_count_other_in(
        &self,
        zones: &Zones,
        card: Option<CardId>,
        playerid: u8,
        location: u8,
        uplayer: u8,
        reason: u32,
        zone: u32,
    ) -> (i32, u32) {
        let (count, flag) = self.get_tofield_count_in(zones, card, playerid, location, uplayer, reason, zone);
        let limit = if location == LOCATION_MZONE {
            self.get_mzone_limit_in(zones, playerid, uplayer, reason)
        } else {
            self.get_szone_limit(playerid, uplayer, reason)
        };
        (count.min(limit), flag)
    }

    pub(crate) fn get_tofield_count(
        &self,
        card: Option<CardId>,
        playerid: u8,
        location: u8,
        uplayer: u8,
        reason: u32,
        zone: u32,
    ) -> (i32, u32) {
        self.get_tofield_count_in(&self.zones(), card, playerid, location, uplayer, reason, zone)
    }

    pub(crate) fn get_tofield_count_in(
        &self,
        zones: &Zones,
        card: Option<CardId>,
        playerid: u8,
        location: u8,
        uplayer: u8,
        reason: u32,
        zone: u32,
    ) -> (i32, u32) {
        if location != LOCATION_MZONE && location != LOCATION_SZONE {
            return (0, 0);
        }
        let mut flag = zones.disabled[playerid] | zones.used[playerid];
        if location == LOCATION_MZONE {
            flag |= !self.get_forced_zones(card, playerid, location, uplayer, reason);
            flag = (flag | !zone) & 0x1f;
        } else {
            flag = ((flag >> 8) | !zone) & 0x1f;
        }
        let count = 5 - field_used_count(flag);
        if location == LOCATION_MZONE {
            flag |= (1 << 5) | (1 << 6);
        }
        (count, flag)
    }

    pub(crate) fn get_mzone_limit(&self, playerid: u8, uplayer: u8, reason: u32) -> i32 {
        self.get_mzone_limit_in(&self.zones(), playerid, uplayer, reason)
    }

    pub(crate) fn get_mzone_limit_in(&self, zones: &Zones, playerid: u8, uplayer: u8, _reason: u32) -> i32 {
        let mut max = 5;
        let mut used_count = field_used_count(zones.used[playerid]);
        if self.is_flag(DUEL_EMZONE) {
            max = 7;
            used_count += [5, 6]
                .into_iter()
                .filter(|&sequence| zones.mzone[playerid].get(sequence).is_some_and(Option::is_some))
                .count() as i32;
        }
        if uplayer < 2 {
            unported!(self, self.filter_player_effect(playerid, EFFECT_MAX_MZONE, true));
        }
        max - used_count
    }

    pub(crate) fn get_szone_limit(&self, playerid: u8, uplayer: u8, _reason: u32) -> i32 {
        if uplayer < 2 {
            unported!(self, self.filter_player_effect(playerid, EFFECT_MAX_SZONE, true));
        }
        5 - field_used_count(self.player[playerid].used_location >> 8)
    }

    pub(crate) fn get_forced_zones(&self, card: Option<CardId>, playerid: u8, location: u8, uplayer: u8, reason: u32) -> u32 {
        if location != LOCATION_MZONE {
            return 0xff;
        }
        let mut effects = if uplayer < 2 {
            self.filter_player_effect(uplayer, EFFECT_MUST_USE_MZONE, true)
        } else {
            EffectSet::default()
        };
        effects.extend(
            card.map(|card| self.filter_effect(card, EFFECT_MUST_USE_MZONE, true))
                .unwrap_or_default(),
        );
        let mut zones: u32 = 0xff7f_ff7f;
        for effect in effects {
            let peffect = &self.effects[effect];
            if peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) && peffect.count_limit == 0 {
                continue;
            }
            // The core passes the card to the value of an effect without a player target.
            unported!(
                self,
                (peffect.operation.is_some()
                    || (!peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET) && matches!(peffect.value, crate::Value::Zones(_))))
                .then_some(effect)
            );
            zones &= self.get_zones_value(
                effect,
                card.filter(|_| !peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET)),
                playerid,
                uplayer,
                reason,
            );
        }
        if uplayer == playerid || uplayer > 1 {
            zones & 0xff
        } else {
            (zones >> 16) & 0xff
        }
    }

    pub(crate) fn shuffle(&mut self, playerid: u8, location: u8) {
        if location & (LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA) == 0 {
            return;
        }
        let Some(mut list) = self.list_mut(playerid, location).map(std::mem::take) else {
            return;
        };
        let all_faceup = list.iter().all(|&card| self.cards[card].is_position(POS_FACEUP));
        let mut reordered = false;
        if !list.is_empty()
            && !(location == LOCATION_HAND && all_faceup)
            && (location == LOCATION_HAND || !self.is_flag(DUEL_PSEUDO_SHUFFLE))
        {
            let mut upper_bound = list.len() as i32;
            if location == LOCATION_EXTRA {
                upper_bound -= self.player[playerid].extra_p_count as i32;
            }
            for index in 0..upper_bound - 1 {
                let other = self.random.get_next_integer(index, upper_bound - 1);
                list.swap(index as usize, other as usize);
                reordered = true;
            }
        }
        let count = list.len();
        if let Some(slot) = self.list_mut(playerid, location) {
            *slot = list;
        }
        if count == 0 {
            return;
        }
        if location == LOCATION_HAND && all_faceup {
            self.core.shuffle_hand_check[playerid] = false;
            return;
        }
        if reordered {
            self.reset_sequence(playerid, location);
        }
        if location == LOCATION_DECK {
            self.trace.message(MSG_SHUFFLE_DECK).u8(playerid);
            self.core.shuffle_deck_check[playerid] = false;
            if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0
                && let Some(&top) = self.player[playerid].list_main.last()
            {
                self.hint_deck_top(playerid, 0, top);
            }
            return;
        }
        let Some(list) = self.player[playerid].pile(location) else { return };
        let mut message = self.trace.message(if location == LOCATION_HAND {
            MSG_SHUFFLE_HAND
        } else {
            MSG_SHUFFLE_EXTRA
        });
        message.u8(playerid).u32(list.len() as u32);
        for &card in list {
            message.u32(self.cards[card].data.code);
        }
        drop(message);
        if location == LOCATION_HAND {
            self.core.shuffle_hand_check[playerid] = false;
            // The core walks each card's indexer in address order; a hand card carries at most one such hint here.
            for card in self.player[playerid].list_hand.clone() {
                for effect in self.card_effects(card) {
                    let peffect = &self.effects[effect];
                    if peffect.is_flag(EFFECT_FLAG_CLIENT_HINT) && !peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET) {
                        let (location, description) = (self.cards.get_info_location(card), peffect.description);
                        self.trace.message(MSG_CARD_HINT).loc(location).u8(CHINT_DESC_ADD).u64(description);
                    }
                }
            }
        }
    }

    pub(crate) fn reset_sequence(&mut self, playerid: u8, location: u8) {
        if location & LOCATION_ONFIELD != 0 {
            return;
        }
        let Some(list) = self.list_mut(playerid, location).map(std::mem::take) else {
            return;
        };
        for (sequence, &card) in list.iter().enumerate() {
            // A write marks the card for the next script snapshot to copy.
            if self.cards[card].current.sequence != sequence as u32 {
                self.cards[card].current.sequence = sequence as u32;
            }
        }
        if let Some(slot) = self.list_mut(playerid, location) {
            *slot = list;
        }
    }

    pub(crate) fn field_swap_deck_and_grave(&mut self, playerid: u8) {
        let in_chain = !self.core.current_chain.is_empty();
        for (location, list) in [
            (LOCATION_GRAVE, self.player[playerid].list_grave.clone()),
            (LOCATION_DECK, self.player[playerid].list_main.clone()),
        ] {
            for &card in list.iter() {
                if in_chain {
                    self.core.just_sent_cards.insert(card);
                }
                let target = &mut self.cards[card];
                target.previous.location = location;
                target.previous.sequence = target.current.sequence;
                self.enable_field_effect(card, false);
                self.cancel_field_effect(card);
            }
        }
        let player = &mut self.player[playerid];
        std::mem::swap(&mut player.list_grave, &mut player.list_main);
        let (mut deck, mut extra) = (Pile::default(), Vec::new());
        let mut extra_flags = vec![0u8; player.list_main.len().div_ceil(8)];
        let swapped = player.list_main.clone();
        let extra_count = (player.list_extra.len() as u32).wrapping_sub(player.extra_p_count);
        for (index, &card) in swapped.iter().enumerate() {
            if self.is_extra_deck_monster(card) {
                if let Some(flags) = extra_flags.get_mut(index / 8) {
                    *flags |= 1 << (index % 8);
                }
                extra.push(card);
            } else {
                deck.push(card);
            }
        }
        self.player[playerid].list_main = deck;
        self.trace
            .message(MSG_SWAP_GRAVE_DECK)
            .u8(playerid)
            .u32(extra_count)
            .u32(extra_flags.len() as u32)
            .bytes(&extra_flags);
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        let moved = [
            (
                self.player[playerid].list_grave.iter().copied().collect::<Vec<_>>(),
                LOCATION_GRAVE,
                POS_FACEUP,
                RESET_TOGRAVE,
            ),
            (
                self.player[playerid].list_main.iter().copied().collect(),
                LOCATION_DECK,
                POS_FACEDOWN_DEFENSE,
                RESET_TODECK,
            ),
            (extra.clone(), LOCATION_EXTRA, POS_FACEDOWN_DEFENSE, RESET_TODECK),
        ];
        for (cards, location, position, reset) in moved {
            for card in cards {
                let target = &mut self.cards[card];
                target.current.position = position;
                target.current.location = location;
                target.current.reason = REASON_EFFECT;
                target.current.reason_effect = reason_effect;
                target.current.reason_player = reason_player;
                self.apply_field_effect(card);
                self.enable_field_effect(card, true);
                self.reset(card, reset, RESET_EVENT);
                if location != LOCATION_GRAVE {
                    self.field_raise_single_event(card, EVENT_LEAVE_GRAVE, reason_effect, 0, reason_player, 0, 0);
                }
            }
        }
        let insert_at = (self.player[playerid].list_extra.len()).saturating_sub(self.player[playerid].extra_p_count as usize);
        for (offset, &card) in extra.iter().enumerate() {
            self.player[playerid].list_extra.insert(insert_at + offset, card);
        }
        self.reset_sequence(playerid, LOCATION_GRAVE);
        self.reset_sequence(playerid, LOCATION_EXTRA);
        self.shuffle(playerid, LOCATION_DECK);
        if !self.player[playerid].list_main.is_empty() || !extra.is_empty() {
            let event_cards = extra
                .iter()
                .copied()
                .chain(self.player[playerid].list_main.iter().copied())
                .collect();
            self.field_raise_event(Some(event_cards), EVENT_LEAVE_GRAVE, reason_effect, 0, reason_player, 0, 0);
            self.process_single_event();
            self.process_instant_event();
        }
    }

    // field::add_effect: an effect without a handler is a global (field only) effect of `owner_player`.
    pub(crate) fn add_effect(&mut self, effect: EffectId, owner_player: u8) {
        if self.effects[effect].handler.is_none() {
            let id = self.next_field_id();
            self.effects.set_id(effect, id);
            let peffect = &mut self.effects[effect];
            peffect.flag[0] |= EFFECT_FLAG_FIELD_ONLY;
            peffect.handler = Some(peffect.owner);
            peffect.effect_owner = owner_player;
            peffect.initial_id = id;
        }
        let owner_type = self.cards[self.effects[effect].owner].data.card_type;
        let peffect = &mut self.effects[effect];
        peffect.card_type = owner_type;
        let (effect_type, code) = (peffect.effect_type, peffect.code);
        let container = if effect_type & EFFECT_TYPE_ACTIONS == 0 {
            if code == EFFECT_SELF_TOGRAVE {
                self.core.global_flag |= GLOBALFLAG_SELF_TOGRAVE;
            } else if code == EFFECT_SPSUMMON_COUNT_LIMIT {
                self.field_effects.spsummon_count_eff.push(effect);
            } else if code == EFFECT_REMOVE_BRAINWASHING {
                self.core.global_flag |= GLOBALFLAG_BRAINWASHING_CHECK;
            } else if code == EFFECT_REVERSE_DECK {
                self.core.global_flag |= GLOBALFLAG_DECK_REVERSE_CHECK;
            }
            if effect_type & EFFECT_TYPE_GRANT != 0 {
                let grants = &mut self.field_effects.grant_effect;
                if !grants.iter().any(|&(granting, _)| granting == effect) {
                    let initial_id = self.effects[effect].initial_id;
                    let at = grants.partition_point(|&(granting, _)| self.effects[granting].initial_id <= initial_id);
                    grants.insert(at, (effect, Vec::new()));
                }
            }
            Some(Container::Aura)
        } else if effect_type & EFFECT_TYPE_IGNITION != 0 {
            Some(Container::Ignition)
        } else if effect_type & EFFECT_TYPE_TRIGGER_O != 0 && effect_type & EFFECT_TYPE_FIELD != 0 {
            Some(Container::TriggerO)
        } else if effect_type & EFFECT_TYPE_TRIGGER_F != 0 && effect_type & EFFECT_TYPE_FIELD != 0 {
            Some(Container::TriggerF)
        } else if effect_type & EFFECT_TYPE_QUICK_O != 0 {
            Some(Container::QuickO)
        } else if effect_type & EFFECT_TYPE_QUICK_F != 0 {
            Some(Container::QuickF)
        } else if effect_type & EFFECT_TYPE_ACTIVATE != 0 {
            Some(Container::Activate)
        } else if effect_type & EFFECT_TYPE_CONTINUOUS != 0 {
            Some(Container::Continuous)
        } else {
            None
        };
        if let Some(container) = container {
            let key = container_key(container, None, code);
            self.field_effects.registry.insert(key, effect);
            self.effects[effect].field_key = NonZeroU64::new(key);
        }
        if code == EVENT_DETACH_MATERIAL {
            self.core.global_flag |= GLOBALFLAG_DETACH_EVENT;
        }
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            return;
        }
        let (flag, reset_flag) = (peffect.flag[0], peffect.reset_flag);
        if peffect.is_disable_related() {
            self.update_disable_check_list(effect);
        }
        self.field_effects.track(effect, flag, reset_flag, self.core.reason_effect.get());
        if flag & EFFECT_FLAG_PLAYER_TARGET != 0 && flag & EFFECT_FLAG_CLIENT_HINT != 0 {
            self.player_hint(effect, owner_player, PHINT_DESC_ADD);
        }
    }

    // MSG_PLAYER_HINT for the players a client hint player effect targets.
    fn player_hint(&mut self, effect: EffectId, owner_player: u8, hint: u8) {
        let peffect = &self.effects[effect];
        let ranges = [(owner_player & 1, peffect.s_range), ((1 - owner_player) & 1, peffect.o_range)];
        let description = peffect.description;
        for player in 0..2 {
            if ranges.iter().any(|&(targeted, range)| targeted == player && range != 0) {
                self.trace.message(MSG_PLAYER_HINT).u8(player).u8(hint).u64(description);
            }
        }
    }

    pub(crate) fn remove_effect(&mut self, effect: EffectId) {
        let Some(key) = self.effects[effect].field_key.take().map(NonZeroU64::get) else {
            return;
        };
        self.field_effects.registry.remove(key, effect);
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_ACTIONS == 0 {
            if peffect.code == EFFECT_SPSUMMON_COUNT_LIMIT {
                remove_item(&mut self.field_effects.spsummon_count_eff, &effect);
            }
            if peffect.effect_type & EFFECT_TYPE_GRANT != 0 {
                self.erase_grant_effect(effect);
            }
        }
        let peffect = &self.effects[effect];
        if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY) {
            return;
        }
        let (flag, reset_flag, owner) = (peffect.flag[0], peffect.reset_flag, peffect.effect_owner);
        if peffect.is_disable_related() {
            self.update_disable_check_list(effect);
        }
        self.field_effects.untrack(effect, flag, reset_flag);
        self.mark_reseted(effect);
        if flag & EFFECT_FLAG_PLAYER_TARGET != 0 && flag & EFFECT_FLAG_CLIENT_HINT != 0 {
            self.player_hint(effect, owner, PHINT_DESC_REMOVE);
        }
    }

    // card::refresh_control_status: the newest EFFECT_SET_CONTROL's controller, else the owner; EFFECT_REMOVE_BRAINWASHING is not ported.
    pub(crate) fn refresh_control_status(&self, card: CardId) -> (u8, Option<EffectId>) {
        match self.filter_effect(card, EFFECT_SET_CONTROL, true).last() {
            Some(&effect) => (self.get_value(effect, card) as u8, Some(effect)),
            None => (self.cards[card].owner, None),
        }
    }

    pub(crate) fn set_control(&mut self, card: CardId, playerid: u8, reset_phase: u16, reset_count: u16) {
        if self.refresh_control_status(card).0 == playerid {
            return;
        }
        let owner = self.core.reason_effect.get().map_or(card, |effect| self.get_handler(effect));
        let effect = self.new_effect(owner);
        let peffect = &mut self.effects[effect];
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        peffect.code = EFFECT_SET_CONTROL;
        peffect.value = crate::effect::Value::Int(i32::from(playerid));
        peffect.flag[0] = EFFECT_FLAG_CANNOT_DISABLE;
        peffect.reset_flag = RESET_EVENT | 0xc6c_0000;
        if reset_count != 0 {
            peffect.reset_flag |= RESET_PHASE | u32::from(reset_phase);
            if peffect.reset_flag & (RESET_SELF_TURN | RESET_OPPO_TURN) == 0 {
                peffect.reset_flag |= RESET_SELF_TURN | RESET_OPPO_TURN;
            }
            peffect.reset_count = reset_count;
        }
        self.card_add_effect(card, effect);
        self.cards[card].current.controler = playerid;
    }

    // core.reseted_effects is a set: the effects removed this turn, deleted at the start of the next.
    pub(crate) fn mark_reseted(&mut self, effect: EffectId) {
        if !self.core.reseted_effects.contains(&effect) {
            self.core.reseted_effects.push(effect);
        }
    }

    // Removes an effect from wherever it is registered, as Effect.Reset and the resets do.
    pub(crate) fn remove_registered_effect(&mut self, effect: EffectId) {
        if self.effects[effect].is_flag(EFFECT_FLAG_FIELD_ONLY) {
            self.remove_effect(effect);
        } else if let Some(handler) = self.effects[effect].handler {
            self.card_remove_effect(handler, effect);
        }
    }

    pub(crate) fn remove_oath_effect(&mut self, reason_effect: EffectId) {
        let oaths: Vec<EffectId> = self
            .field_effects
            .oath
            .iter()
            .filter(|&&(_, reason)| reason == Some(reason_effect))
            .map(|&(oath, _)| oath)
            .collect();
        for effect in oaths {
            self.field_effects.oath.retain(|&(oath, _)| oath != effect);
            self.remove_registered_effect(effect);
        }
    }

    pub(crate) fn release_oath_relation(&mut self, reason_effect: EffectId) {
        for (_, reason) in self.field_effects.oath.iter_mut() {
            if *reason == Some(reason_effect) {
                *reason = None;
            }
        }
    }

    pub(crate) fn reset_phase(&mut self, phase: u16) {
        for effect in self.field_effects.pheff.clone() {
            if self.effect_reset(effect, u32::from(phase), RESET_PHASE) {
                self.remove_registered_effect(effect);
            }
        }
    }

    pub(crate) fn reset_chain(&mut self) {
        self.core.chain_limit_p.clear();
        self.core.effect_count_code_chain.clear();
        for effect in self.field_effects.cheff.clone() {
            self.remove_registered_effect(effect);
        }
    }

    fn count_map(&mut self, flag: u8) -> &mut CountCodes {
        if flag & EFFECT_COUNT_CODE_DUEL != 0 {
            &mut self.core.effect_count_code_duel
        } else if flag & EFFECT_COUNT_CODE_CHAIN != 0 {
            &mut self.core.effect_count_code_chain
        } else {
            &mut self.core.effect_count_code
        }
    }

    // The core builds the key with hopt_index and flag in swapped places; kept, as only equality matters.
    pub(crate) fn add_effect_code(&mut self, code: u32, flag: u8, hopt_index: u8, playerid: u8) {
        let key = count_map_key(code, hopt_index, flag, playerid);
        let map = self.count_map(flag);
        match map.iter_mut().find(|(other, _)| *other == key) {
            Some((_, count)) => *count += 1,
            None => map.push((key, 1)),
        }
    }

    pub(crate) fn get_effect_code(&self, code: u32, flag: u8, hopt_index: u8, playerid: u8) -> u32 {
        let key = count_map_key(code, hopt_index, flag, playerid);
        let map = if flag & EFFECT_COUNT_CODE_DUEL != 0 {
            &self.core.effect_count_code_duel
        } else if flag & EFFECT_COUNT_CODE_CHAIN != 0 {
            &self.core.effect_count_code_chain
        } else {
            &self.core.effect_count_code
        };
        map.iter().find(|(other, _)| *other == key).map_or(0, |&(_, count)| count)
    }

    pub(crate) fn dec_effect_code(&mut self, code: u32, flag: u8, hopt_index: u8, playerid: u8) {
        let key = count_map_key(code, hopt_index, flag, playerid);
        if let Some((_, count)) = self.count_map(flag).iter_mut().find(|(other, _)| *other == key) {
            *count = count.saturating_sub(1);
        }
    }

    #[inline(always)]
    pub(crate) fn filter_field_effect(&self, code: u32) -> EffectSet {
        if !self.field_effects.registry.has_code(code) {
            return EffectSet::default();
        }
        self.filter_field_effect_lookup(code)
    }

    // filter_field_effect into a list of any inline size: the summon procedures of both Extra Decks outgrow an EffectSet.
    #[inline(always)]
    pub(crate) fn filter_field_effect_list<const N: usize>(&self, code: u32) -> List<EffectId, N> {
        if !self.field_effects.registry.has_code(code) {
            return List::default();
        }
        self.filter_field_effect_lookup(code)
    }

    // filter_field_effect where conditions may write; the core steps past each effect before its condition, which may reset it.
    pub(crate) fn filter_field_effect_mut(&mut self, code: u32) -> EffectSet {
        if !self.field_effects.registry.has_code(code) {
            return EffectSet::default();
        }
        let registered: EffectSet = self.field_effects.registry.equal_range(Container::Aura, None, code).collect();
        let mut effects: EffectSet = registered.into_iter().filter(|&effect| self.is_available_mut(effect)).collect();
        effects.sort_by_key(|&effect| self.effects.id(effect));
        effects
    }

    fn filter_field_effect_lookup<S: FromIterator<EffectId> + DerefMut<Target = [EffectId]>>(&self, code: u32) -> S {
        let mut effects: S = self
            .field_effects
            .registry
            .equal_range(Container::Aura, None, code)
            .filter(|&effect| self.is_available(effect))
            .collect();
        effects.sort_by_key(|&effect| self.effects.id(effect));
        effects
    }

    fn erase_grant_effect(&mut self, effect: EffectId) {
        let Some(at) = self.field_effects.grant_effect.iter().position(|&(granting, _)| granting == effect) else {
            return;
        };
        let (_, gained) = self.field_effects.grant_effect.remove(at);
        for (card, granted) in gained {
            self.card_remove_effect(card, granted);
        }
    }

    // field::adjust_grant_effect: gives each affected card a copy of the label object effect, and takes it back from the others.
    pub(crate) fn adjust_grant_effect(&mut self) -> bool {
        let mut adjusted = false;
        let mut at = 0;
        while let Some((effect, gained)) = self.field_effects.grant_effect.get(at).cloned() {
            at += 1;
            let Some(granted) = self.effects[effect].label_object.filter(|&label| !self.effects.is_deleted(label)) else {
                continue;
            };
            let affected = if self.is_available(effect) {
                self.filter_affected_cards(effect)
            } else {
                CardSet::default()
            };
            let add: CardSet = affected
                .iter()
                .filter(|&card| self.is_affect_by_effect(card, Some(effect)) && !gained.iter().any(|&(gained, _)| gained == card))
                .collect();
            let remove: CardSet = gained
                .iter()
                .map(|&(card, _)| card)
                .filter(|&card| !self.is_affect_by_effect(card, Some(effect)) || !affected.contains(card))
                .collect();
            for card in add {
                let copy = self.clone_effect(granted);
                self.effects[copy].owner = card;
                self.card_add_effect(card, copy);
                if let Some((_, gained)) = self.field_effects.grant_effect.get_mut(at - 1) {
                    gained.push((card, copy));
                }
            }
            for card in remove {
                let Some((_, gained)) = self.field_effects.grant_effect.get_mut(at - 1) else {
                    continue;
                };
                if let Some(position) = gained.iter().position(|&(gained, _)| gained == card) {
                    let (_, copy) = gained.remove(position);
                    self.card_remove_effect(card, copy);
                }
            }
            adjusted |= !add.is_empty() || !remove.is_empty();
        }
        adjusted
    }

    // The cards a field effect applies to (filter_affected_cards).
    fn filter_affected_cards(&self, effect: EffectId) -> CardSet {
        let peffect = &self.effects[effect];
        let mut cards = CardSet::default();
        if peffect.effect_type & EFFECT_TYPE_ACTIONS != 0
            || peffect.effect_type & EFFECT_TYPE_FIELD == 0
            || peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET | EFFECT_FLAG_SPSUM_PARAM)
        {
            return cards;
        }
        let mut own = self.get_handler_player(effect);
        if own == PLAYER_NONE {
            return cards;
        }
        for range in [peffect.s_range, peffect.o_range] {
            let player = &self.player[own];
            let mut add = |card: CardId| {
                if self.effect_is_target(effect, card) {
                    cards.insert(card);
                }
            };
            let range = range as u8;
            if range & LOCATION_MZONE != 0 {
                player.list_mzone.iter().flatten().for_each(|&card| add(card));
            }
            if range & LOCATION_SZONE != 0 {
                player.list_szone.iter().flatten().for_each(|&card| add(card));
            }
            for location in [LOCATION_GRAVE, LOCATION_REMOVED, LOCATION_HAND, LOCATION_DECK, LOCATION_EXTRA] {
                if range & location != 0 {
                    player.pile(location).into_iter().flatten().for_each(|&card| add(card));
                }
            }
            own = 1 - own;
        }
        cards
    }

    #[inline(always)]
    pub(crate) fn filter_player_effect(&self, playerid: u8, code: u32, sort: bool) -> EffectSet {
        if !self.field_effects.registry.has_code(code) {
            return EffectSet::default();
        }
        self.filter_player_effect_lookup(playerid, code, sort)
    }

    fn filter_player_effect_lookup(&self, playerid: u8, code: u32, sort: bool) -> EffectSet {
        let mut effects: EffectSet = self
            .field_effects
            .registry
            .equal_range(Container::Aura, None, code)
            .filter(|&effect| self.is_target_player(effect, playerid) && self.is_available(effect))
            .collect();
        if sort {
            effects.sort_by_key(|&effect| self.effects.id(effect));
        }
        effects
    }

    #[inline(always)]
    pub fn is_player_affected_by_effect(&self, playerid: u8, code: u32) -> Option<EffectId> {
        if !self.field_effects.registry.has_code(code) {
            return None;
        }
        self.is_player_affected_by_effect_lookup(playerid, code)
    }

    fn is_player_affected_by_effect_lookup(&self, playerid: u8, code: u32) -> Option<EffectId> {
        self.field_effects
            .registry
            .equal_range(Container::Aura, None, code)
            .find(|&effect| self.is_target_player(effect, playerid) && self.is_available(effect))
    }

    // The cards of `location1` (own side) and `location2` that pass, in zone order; returns whether it stopped at `count`.
    pub(crate) fn filter_matching_card(
        &self,
        filter: &mut dyn FnMut(&Duel, CardId) -> bool,
        own: u8,
        location1: u32,
        location2: u32,
        group: &mut CardSet,
        exception: &CardSet,
        count: usize,
        is_target: bool,
    ) -> bool {
        if own > 1 {
            return false;
        }
        let mut found = 0;
        let mut check = |duel: &Duel, card: CardId, extra: bool| {
            if !extra
                || exception.contains(card)
                || !filter(duel, card)
                || (is_target
                    && !duel
                        .core
                        .reason_effect
                        .get()
                        .is_some_and(|effect| duel.is_capable_be_effect_target(card, effect, duel.core.reason_player.get())))
            {
                return false;
            }
            found += 1;
            if count != 0 && found >= count {
                return true;
            }
            group.insert(card);
            false
        };
        let outer = usize::from(self.is_flag(DUEL_3_COLUMNS_FIELD));
        let mut player = own;
        for location in [location1, location2] {
            let info = &self.player[player];
            let monster = |card: CardId| !self.cards[card].get_status(STATUS_SUMMONING | STATUS_SUMMON_DISABLED | STATUS_SPSUMMON_STEP);
            let spell = |card: CardId| !self.cards[card].is_status(STATUS_ACTIVATE_DISABLED);
            let location = location as u16;
            let mut stops_in =
                |list: &[Option<CardId>], extra: &dyn Fn(CardId) -> bool| list.iter().flatten().any(|&card| check(self, card, extra(card)));
            let (mzone, szone) = (&info.list_mzone, &info.list_szone);
            if location & u16::from(LOCATION_MZONE) != 0 {
                if stops_in(mzone, &monster) {
                    return true;
                }
            } else {
                if location & LOCATION_MMZONE != 0 && stops_in(mzone.get(..5).unwrap_or_default(), &monster) {
                    return true;
                }
                if location & LOCATION_EMZONE != 0 && stops_in(mzone.get(5 + outer..7 - outer).unwrap_or_default(), &monster) {
                    return true;
                }
            }
            if location & u16::from(LOCATION_SZONE) != 0 {
                if stops_in(szone, &spell) {
                    return true;
                }
            } else {
                if location & LOCATION_STZONE != 0 && stops_in(szone.get(outer..5 - outer).unwrap_or_default(), &spell) {
                    return true;
                }
                if location & LOCATION_FZONE != 0 && stops_in(szone.get(5..6).unwrap_or_default(), &spell) {
                    return true;
                }
                let pzone = |card: CardId| self.cards[card].current.pzone && spell(card);
                let pzones = [self.get_pzone_index(0), self.get_pzone_index(1)].map(|index| szone.get(index as usize).copied().flatten());
                if location & LOCATION_PZONE != 0 && stops_in(&pzones, &pzone) {
                    return true;
                }
            }
            for bit in [LOCATION_DECK, LOCATION_EXTRA, LOCATION_HAND, LOCATION_GRAVE, LOCATION_REMOVED] {
                if location & u16::from(bit) != 0 && info.pile(bit).into_iter().flatten().any(|&card| check(self, card, true)) {
                    return true;
                }
            }
            player = 1 - player;
        }
        false
    }

    pub(crate) fn update_disable_check_list(&mut self, effect: EffectId) {
        for card in self.filter_affected_cards(effect) {
            self.add_to_disable_check_list(card);
        }
    }

    // Fills direct_attackable and returns the attackable monsters; atype 1: only be attacked, 2: only attack, 3: must attack, 4: any.
    pub(crate) fn field_get_attack_target(&mut self, card: CardId, chain_attack: bool, select_target: bool) -> Cards16 {
        self.field_get_attack_target_info(card, chain_attack, select_target).0
    }

    // The same with the type of attack and must_attack_map.
    pub(crate) fn field_get_attack_target_info(&mut self, card: CardId, chain_attack: bool, select_target: bool) -> (Cards16, AttackInfo) {
        self.cards[card].direct_attackable = 0;
        let p = self.cards[card].current.controler;
        let opponent = self.player[1 - p].list_mzone;
        let (mut auto_attack, mut only_attack, mut must_attack) = (Cards8::default(), Cards8::default(), Cards8::default());
        let mut must_attack_map = MustAttackMap::new();
        for target in opponent.into_iter().flatten() {
            if self.is_affected_by_effect(target, EFFECT_ONLY_BE_ATTACKED).is_some() {
                auto_attack.push(target);
            }
            if self
                .is_affected_by_effect_target(card, EFFECT_ONLY_ATTACK_MONSTER, target)
                .is_some()
            {
                only_attack.push(target);
            }
            if let Some(effect) = self.is_affected_by_effect_target(card, EFFECT_MUST_ATTACK_MONSTER, target) {
                must_attack.push(target);
                must_attack_map.push((effect, target));
            }
        }
        // The core keys the multimap by effect address, which the arena index stands in for.
        must_attack_map.sort_by_key(|&(effect, _)| effect);
        let mut info = AttackInfo { atype: 0, must_attack_map };
        let (atype, attack_tg) = if !auto_attack.is_empty() {
            info.atype = 1;
            if auto_attack.len() != 1 {
                return (Cards16::default(), info);
            }
            (1, auto_attack)
        } else if self.is_affected_by_effect(card, EFFECT_ONLY_ATTACK_MONSTER).is_some() {
            info.atype = 2;
            if only_attack.len() != 1 {
                return (Cards16::default(), info);
            }
            (2, only_attack)
        } else if self.is_affected_by_effect(card, EFFECT_MUST_ATTACK_MONSTER).is_some() {
            info.atype = 3;
            if must_attack.is_empty() {
                return (Cards16::default(), info);
            }
            (3, must_attack)
        } else {
            unported!(self, self.is_player_affected_by_effect(p, EFFECT_SELF_ATTACK));
            let attack_tg = opponent
                .into_iter()
                .flatten()
                .filter(|&target| Some(target) != self.core.attacker)
                .collect();
            info.atype = 4;
            (4, attack_tg)
        };
        let choosable = select_target && (atype == 2 || atype == 4);
        let extra_attacks = |duel: &Self, code| {
            duel.filter_effect(card, code, true)
                .into_iter()
                .map(|effect| duel.get_value(effect, card))
                .fold(0, i32::max)
        };
        let (extra_count, extra_count_m) = (
            extra_attacks(self, EFFECT_EXTRA_ATTACK),
            extra_attacks(self, EFFECT_EXTRA_ATTACK_MONSTER),
        );
        let announce_count = i32::from(self.cards[card].announce_count);
        if !chain_attack
            && announce_count > extra_count
            && (extra_count_m == 0 || announce_count > extra_count_m || self.find_announced_card(card, None) > 0)
        {
            let mut targets = Cards16::default();
            if let Some(effect) = self
                .is_affected_by_effect(card, EFFECT_ATTACK_ALL)
                .filter(|_| self.cards[card].attack_all_target != 0)
            {
                for target in attack_tg {
                    let value = self.get_value(effect, target);
                    if value == 0 || i32::from(self.find_announced_card(card, Some(target))) >= value {
                        continue;
                    }
                    if atype >= 2
                        && self
                            .is_affected_by_effect_target(target, EFFECT_IGNORE_BATTLE_TARGET, card)
                            .is_some()
                    {
                        continue;
                    }
                    if choosable
                        && (self
                            .is_affected_by_effect_target(target, EFFECT_CANNOT_BE_BATTLE_TARGET, card)
                            .is_some()
                            || self
                                .is_affected_by_effect_target(card, EFFECT_CANNOT_SELECT_BATTLE_TARGET, target)
                                .is_some())
                    {
                        continue;
                    }
                    targets.push(target);
                }
            }
            return (targets, info);
        }
        let mut mcount = 0;
        let mut targets = Cards16::default();
        for target in attack_tg {
            if atype >= 2
                && self
                    .is_affected_by_effect_target(target, EFFECT_IGNORE_BATTLE_TARGET, card)
                    .is_some()
            {
                continue;
            }
            if self.cards[target].current.controler != p {
                mcount += 1;
            }
            if chain_attack && self.core.chain_attack_target.is_some_and(|chain_target| chain_target != target) {
                continue;
            }
            if choosable
                && (self
                    .is_affected_by_effect_target(target, EFFECT_CANNOT_BE_BATTLE_TARGET, card)
                    .is_some()
                    || self
                        .is_affected_by_effect_target(card, EFFECT_CANNOT_SELECT_BATTLE_TARGET, target)
                        .is_some())
            {
                continue;
            }
            targets.push(target);
        }
        if atype <= 3 {
            return (targets, info);
        }
        if (mcount == 0 || self.is_affected_by_effect(card, EFFECT_DIRECT_ATTACK).is_some() || self.core.attack_player)
            && self.is_affected_by_effect(card, EFFECT_CANNOT_DIRECT_ATTACK).is_none()
            && !(!chain_attack && extra_count_m != 0 && announce_count > extra_count)
            && !(chain_attack && self.core.chain_attack_target.is_some())
        {
            self.cards[card].direct_attackable = 1;
        }
        (targets, info)
    }

    pub(crate) fn confirm_attack_target(&mut self) -> bool {
        let Some(attacker) = self.core.attacker else { return false };
        let targets = self.field_get_attack_target(attacker, self.core.chain_attack, false);
        match self.core.attack_target {
            Some(target) => targets.contains(&target),
            None => self.cards[attacker].direct_attackable != 0,
        }
    }

    pub(crate) fn attack_all_target_check(&mut self) {
        let Some(attacker) = self.core.attacker else { return };
        if self.core.attack_target.is_none() {
            self.cards[attacker].attack_all_target = 0;
            return;
        }
        if self
            .is_affected_by_effect(attacker, EFFECT_ATTACK_ALL)
            .is_some_and(|effect| self.get_value(effect, self.core.attack_target.unwrap_or_default()) == 0)
        {
            self.cards[attacker].attack_all_target = 0;
        }
    }

    pub(crate) fn add_announced_card(&mut self, attacker: CardId, target: Option<CardId>) {
        let fieldid = target.map_or(0, |target| self.cards[target].fieldid_r);
        match self
            .announced_cards
            .iter_mut()
            .find(|entry| entry.0 == attacker && entry.1 == fieldid)
        {
            Some(entry) => entry.2 += 1,
            None => self.announced_cards.push((attacker, fieldid, 1)),
        }
    }

    pub(crate) fn add_attacked_card(&mut self, attacker: CardId, target: Option<CardId>) {
        let fieldid = target.map_or(0, |target| self.cards[target].fieldid_r);
        if !self.attacked_cards.contains(&(attacker, fieldid)) {
            self.attacked_cards.push((attacker, fieldid));
        }
    }

    pub(crate) fn add_battled_card(&mut self, card: CardId, battled: Option<CardId>) {
        let fieldid = battled.map_or(0, |battled| self.cards[battled].fieldid_r);
        if !self.battled_cards.contains(&(card, fieldid)) {
            self.battled_cards.push((card, fieldid));
        }
    }

    pub(crate) fn find_announced_card(&self, attacker: CardId, target: Option<CardId>) -> u8 {
        let fieldid = target.map_or(0, |target| self.cards[target].fieldid_r);
        self.announced_cards
            .iter()
            .find(|entry| entry.0 == attacker && entry.1 == fieldid)
            .map_or(0, |entry| entry.2)
    }

    // The fieldid_r of the opponent's monsters, compared as core.opp_mzone.
    pub(crate) fn opponent_field_ids(&self) -> FieldIds {
        let opponent = self.player[1 - self.infos.turn_player].list_mzone;
        let mut ids: FieldIds = opponent.into_iter().flatten().map(|card| self.cards[card].fieldid_r).collect();
        ids.sort_unstable();
        ids
    }

    // core.unique_cards[controler]: the cards on the field with SetUniqueOnField whose rule covers that player's cards.
    pub(crate) fn unique_cards(&self, controler: u8) -> Vec<CardId> {
        let on_field = (0..2).flat_map(|p| {
            let player = &self.player[p];
            player.list_mzone.into_iter().chain(player.list_szone).flatten()
        });
        on_field
            .filter(|&card| {
                let target = &self.cards[card];
                target.unique.is_some_and(|unique| {
                    target.current.location & unique.location != 0 && unique.pos[u8::from(target.current.controler != controler)]
                })
            })
            .collect()
    }

    pub(crate) fn get_unique_target(&self, ucard: CardId, controler: u8, icard: Option<CardId>) -> CardSet {
        let mut cards = CardSet::default();
        let Some(unique) = self.cards[ucard].unique else { return cards };
        for p in 0..2 {
            if !unique.pos[p] {
                continue;
            }
            let player = &self.player[controler ^ p];
            let mzone = (unique.location & LOCATION_MZONE != 0)
                .then_some(player.list_mzone)
                .into_iter()
                .flatten()
                .flatten();
            let szone = (unique.location & LOCATION_SZONE != 0)
                .then_some(player.list_szone)
                .into_iter()
                .flatten()
                .flatten();
            let mzone = mzone.filter(|&card| !self.cards[card].get_status(STATUS_SPSUMMON_STEP));
            for card in mzone.chain(szone) {
                if Some(card) != icard && self.cards[card].is_position(POS_FACEUP) && (unique.filter)(card, self) {
                    cards.insert(card);
                }
            }
        }
        cards
    }

    // The check the core makes where a card gets a new fieldid: a card that now breaks a SetUniqueOnField rule is marked.
    pub(crate) fn mark_unique_violation(&mut self, card: CardId) {
        let current = self.cards[card].current;
        if self.check_unique_onfield(card, current.controler, current.location).is_some() {
            self.cards[card].unique_fieldid = u32::MAX;
        }
    }

    // field::check_unique_onfield: the effect of the SetUniqueOnField that forbids the card on the field, if any.
    pub(crate) fn check_unique_onfield(&self, card: CardId, controler: u8, location: u8) -> Option<EffectId> {
        self.check_unique_onfield_full(card, controler, location, None)
    }

    pub(crate) fn check_unique_onfield_full(&self, card: CardId, controler: u8, location: u8, icard: Option<CardId>) -> Option<EffectId> {
        for ucard in self.unique_cards(controler) {
            let (unique, target) = (self.cards[ucard].unique?, &self.cards[ucard]);
            if ucard != card
                && Some(ucard) != icard
                && target.is_position(POS_FACEUP)
                && target.get_status(STATUS_EFFECT_ENABLED)
                && !target.get_status(STATUS_DISABLED | STATUS_FORBIDDEN)
                && target.unique_fieldid != 0
                && (unique.filter)(card, self)
                && unique.location & location != 0
            {
                return Some(unique.effect);
            }
        }
        let unique = self.cards[card].unique?;
        if unique.location & location == 0 || self.cards[card].get_status(STATUS_DISABLED | STATUS_FORBIDDEN) {
            return None;
        }
        let mut cards = self.get_unique_target(card, controler, icard);
        if (unique.filter)(card, self) {
            cards.insert(card);
        }
        (cards.len() >= 2).then_some(unique.effect)
    }

    // Puts the cards the target may tribute into the sets, sets their release_param and returns the total; `mg` restricts them.
    pub(crate) fn get_summon_release_list(
        &mut self,
        target: CardId,
        release_list: &mut CardSet,
        ex_list: &mut CardSet,
        mg: Option<CardSet>,
        ex: bool,
        releasable: u32,
        pos: u32,
    ) -> u32 {
        let (rcount, params) = self.summon_release_list(target, release_list, ex_list, mg, ex, releasable, pos);
        for (card, param) in params {
            self.cards[card].release_param = param;
        }
        rcount
    }

    // get_summon_release_list without setting release_param, which only the tribute selection after it reads.
    pub(crate) fn summon_release_list(
        &self,
        target: CardId,
        release_list: &mut CardSet,
        ex_list: &mut CardSet,
        mg: Option<CardSet>,
        ex: bool,
        releasable: u32,
        pos: u32,
    ) -> (u32, List<(CardId, u32), 16>) {
        let p = self.cards[target].current.controler;
        let mut ex_tribute = CardSet::default();
        for effect in self.filter_effect(target, EFFECT_ADD_EXTRA_TRIBUTE, true) {
            if self.get_own_value(effect) as u32 & pos != 0 {
                self.filter_inrange_cards(effect, &mut ex_tribute);
            }
        }
        let release_param = |duel: &Self, card: CardId| {
            if duel.is_affected_by_effect_target(card, EFFECT_TRIPLE_TRIBUTE, target).is_some() {
                3
            } else if duel.is_affected_by_effect_target(card, EFFECT_DOUBLE_TRIBUTE, target).is_some() {
                2
            } else {
                1
            }
        };
        let (mut rcount, mut params) = (0, List::default());
        let in_mg = |card: CardId| mg.is_none_or(|mg| mg.contains(card));
        for card in self.player[p].list_mzone.into_iter().flatten() {
            if (releasable >> self.cards[card].current.sequence) & 1 != 0 && self.is_releasable_by_summon(card, p, target) {
                if !in_mg(card) {
                    continue;
                }
                release_list.insert(card);
                let param = release_param(self, card);
                params.push((card, param));
                rcount += param;
            }
        }
        for card in self.player[1 - p].list_mzone.into_iter().flatten() {
            if (releasable >> (self.cards[card].current.sequence + 16)) & 1 == 0 || !self.is_releasable_by_summon(card, p, target) {
                continue;
            }
            if !in_mg(card) {
                continue;
            }
            let param = release_param(self, card);
            params.push((card, param));
            if self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE).is_some() {
                ex_list.insert(card);
                rcount += param;
            } else if ex || ex_tribute.contains(card) {
                release_list.insert(card);
                rcount += param;
            } else {
                unported!(self, self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE_SUM));
            }
        }
        (rcount, params)
    }

    // Puts the cards the player may release for a non-summon reason into the set, matching `fun`.
    pub(crate) fn get_release_list(
        &self,
        playerid: u8,
        release_list: &mut CardSet,
        use_hand: bool,
        fun: Option<ReleaseFilter>,
        exc: Option<CardId>,
        exg: Option<CardSet>,
        use_oppo: bool,
        reason: u32,
    ) {
        let usable = |card: CardId| Some(card) != exc && !exg.is_some_and(|exg| exg.contains(card));
        let releasable = |card: CardId| self.is_releasable_by_nonsummon(card, playerid, reason) && fun.is_none_or(|fun| fun(card, self));
        for card in self.player[playerid].list_mzone.into_iter().flatten() {
            if usable(card) && releasable(card) {
                release_list.insert(card);
            }
        }
        if use_hand {
            for &card in &self.player[playerid].list_hand {
                if usable(card) && releasable(card) {
                    release_list.insert(card);
                }
            }
        }
        for card in self.player[1 - playerid].list_mzone.into_iter().flatten() {
            if !(usable(card) && (self.cards[card].is_position(POS_FACEUP) || fun.is_none()) && releasable(card)) {
                continue;
            }
            if use_oppo {
                release_list.insert(card);
            } else {
                unported!(
                    self,
                    self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE)
                        .or_else(|| self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE_NONSUM))
                );
            }
        }
    }

    // Whether the player can release at least `min` cards matching `fun` (see get_release_list).
    pub(crate) fn check_release_list(
        &self,
        playerid: u8,
        min: usize,
        fun: &dyn Fn(CardId, &Self) -> bool,
        options: &ReleaseOptions,
        reason: u32,
    ) -> bool {
        let mut relcard = CardSet::default();
        self.get_release_list(
            playerid,
            &mut relcard,
            options.use_hand,
            Some(fun),
            options.exception,
            options.exception_group,
            options.use_oppo,
            reason,
        );
        if options.check_field {
            let zone = options.zone
                & (0x1f
                    & self.get_forced_zones(
                        options.to_check,
                        playerid,
                        LOCATION_MZONE,
                        options.toplayer,
                        LOCATION_REASON_TOFIELD,
                    ));
            let (ct, _) = self.get_useable_count(
                options.to_check,
                playerid,
                LOCATION_MZONE,
                options.toplayer,
                LOCATION_REASON_TOFIELD,
                zone,
            );
            if ct < min as i32 && !relcard.iter().any(|card| self.is_release_in_zone(card, playerid, zone)) {
                return false;
            }
        }
        for card in self.core.must_select_cards.get().iter() {
            if !relcard.contains(card) {
                return false;
            }
            relcard.remove(card);
        }
        relcard.len() >= min
    }

    // Whether the card is a monster of the player in one of the zones.
    pub(crate) fn is_release_in_zone(&self, card: CardId, playerid: u8, zone: u32) -> bool {
        let current = self.cards[card].current;
        current.location == LOCATION_MZONE && current.controler == playerid && (zone >> current.sequence) & 1 != 0
    }

    pub(crate) fn get_summon_count_limit(&self, playerid: u8) -> i32 {
        if self.is_flag(DUEL_UNLIMITED_SUMMONS) {
            return i32::MAX - 100;
        }
        if !self.field_effects.registry.has_code(EFFECT_SET_SUMMON_COUNT_LIMIT) {
            return 1;
        }
        self.filter_player_effect(playerid, EFFECT_SET_SUMMON_COUNT_LIMIT, true)
            .into_iter()
            .map(|effect| self.get_own_value(effect))
            .fold(1, i32::max)
    }

    pub(crate) fn field_get_draw_count(&self, playerid: u8) -> i32 {
        let mut count = self.player[playerid].draw_count;
        for effect in self.filter_player_effect(playerid, EFFECT_DRAW_COUNT, true) {
            match self.get_own_value(effect) {
                0 => return 0,
                value => count = count.max(value),
            }
        }
        count
    }

    // Adds the cards in the ranges of a field effect that its target function accepts.
    fn filter_inrange_cards(&self, effect: EffectId, cards: &mut CardSet) {
        let peffect = &self.effects[effect];
        let own = self.get_handler_player(effect);
        if peffect.is_flag(EFFECT_FLAG_PLAYER_TARGET | EFFECT_FLAG_SPSUM_PARAM) || own == PLAYER_NONE {
            return;
        }
        unported!(
            self,
            ((peffect.s_range | peffect.o_range) & !u16::from(LOCATION_MZONE) != 0).then_some(effect)
        );
        for (player, range) in [(own, peffect.s_range), (1 - own, peffect.o_range)] {
            if range & u16::from(LOCATION_MZONE) != 0 {
                for card in self.player[player].list_mzone.into_iter().flatten() {
                    if self.is_fit_target_function(effect, card) {
                        cards.insert(card);
                    }
                }
            }
        }
    }

    // For the checks of the idle step; the Tributes are shared only while no effect enters them.
    pub(crate) fn open_idle_memo(&self) {
        let registry = &self.field_effects.registry;
        if !TRIBUTE_CODES.iter().any(|&code| registry.has_code(code)) {
            self.idle_memo.tributes.set(Shared::Open);
        }
        self.idle_memo.synchro_materials.set(Shared::Open);
    }

    pub(crate) fn field_check_tribute(
        &self,
        card: CardId,
        min: i32,
        max: i32,
        mg: Option<CardSet>,
        toplayer: u8,
        zone: u32,
        releasable: u32,
        pos: u32,
    ) -> bool {
        let sumplayer = self.cards[card].current.controler;
        let key = (sumplayer, toplayer, zone, releasable);
        let shared = self.idle_memo.tributes.get();
        if let Shared::Filled(tributes) = shared
            && mg.is_none()
            && tributes.key == key
        {
            return tributes.suffice(min, max);
        }
        // An open memo means no effect is looked up here: counting past a failed test runs nothing, and the counts fit any card.
        let share = mg.is_none() && matches!(shared, Shared::Open);
        let ex = toplayer == 1 - sumplayer;
        let (mut release_list, mut ex_list) = (CardSet::default(), CardSet::default());
        let count = self
            .summon_release_list(card, &mut release_list, &mut ex_list, mg, ex, releasable, pos)
            .0 as i32;
        if !share && min > max.min(count) {
            return false;
        }
        let zone = zone & (0x1f & self.get_forced_zones(Some(card), toplayer, LOCATION_MZONE, sumplayer, LOCATION_REASON_TOFIELD));
        let (free, _) = self.get_tofield_count(Some(card), toplayer, LOCATION_MZONE, sumplayer, LOCATION_REASON_TOFIELD, zone);
        if !share && free <= 0 && max.min(count) <= 0 {
            return false;
        }
        let to_check = if !ex_list.is_empty() && ex_list.len() as i32 >= min {
            ex_list
        } else {
            release_list
        };
        let (mut own, mut freed) = (0, 0);
        for tribute in to_check.iter() {
            let current = self.cards[tribute].current;
            if current.location == LOCATION_MZONE && current.controler == toplayer {
                own += 1;
                if (zone >> current.sequence) & 1 != 0 {
                    freed += 1;
                }
            }
        }
        if !share && free + freed <= 0 {
            return false;
        }
        let tributes = Tributes {
            key,
            count,
            extra: ex_list.len() as i32,
            free,
            own,
            freed,
            limit: self.get_mzone_limit(toplayer, sumplayer, LOCATION_REASON_TOFIELD),
        };
        if share {
            self.idle_memo.tributes.set(Shared::Filled(tributes));
        }
        tributes.suffice(min, max)
    }

    // A matching player effect without a target forbids the action, one with a target only for the cards it takes.
    #[inline(always)]
    fn is_player_can(&self, playerid: u8, code: u32, card: CardId) -> bool {
        self.is_player_can_for_reason(playerid, code, card, 0)
    }

    fn is_player_can_for_reason(&self, playerid: u8, code: u32, card: CardId, reason: u32) -> bool {
        if !self.field_effects.registry.has_code(code) {
            return true;
        }
        self.is_player_can_lookup(playerid, code, card, reason)
    }

    fn is_player_can_lookup(&self, playerid: u8, code: u32, card: CardId, reason: u32) -> bool {
        !self
            .filter_player_effect(playerid, code, true)
            .into_iter()
            .any(|effect| self.limit_targets(effect, card, playerid, Some(reason), None))
    }

    #[inline]
    fn limit_targets(
        &self,
        limit: EffectId,
        card: CardId,
        player: u8,
        reason: Option<u32>,
        summon: Option<(u32, u8, u8, Option<EffectId>)>,
    ) -> bool {
        match (self.effects[limit].target, reason, summon) {
            (Target::Card(target), _, _) => target(self, limit, card),
            (Target::PlayerCard(target), Some(reason), _) => target(self, limit, card, player, reason),
            (Target::SpSummon(target), _, Some((sumtype, first, second, effect))) => {
                target(self, limit, card, player, sumtype, first, second, effect)
            }
            _ => true,
        }
    }

    // EFFECT_CANNOT_SUMMON and EFFECT_CANNOT_FLIP_SUMMON, whose target may read the summoning player, type and target player.
    fn is_player_can_summon_target(&self, playerid: u8, code: u32, card: CardId, sumtype: u32, toplayer: u8) -> bool {
        if !self.field_effects.registry.has_code(code) {
            return true;
        }
        let summon = Some((sumtype, POS_FACEUP as u8, toplayer, None));
        !self
            .filter_player_effect(playerid, code, true)
            .into_iter()
            .any(|effect| self.limit_targets(effect, card, playerid, None, summon))
    }

    pub(crate) fn field_is_player_can_summon(&self, sumtype: u32, playerid: u8, card: CardId, toplayer: u8) -> bool {
        unported!(self, self.filter_player_effect(playerid, EFFECT_FORCE_NORMAL_SUMMON_POSITION, true));
        self.is_player_can_summon_target(playerid, EFFECT_CANNOT_SUMMON, card, sumtype | SUMMON_TYPE_NORMAL, toplayer)
    }

    // The core passes only the card and the player; cards read the nil parameters past them as the player.
    pub(crate) fn is_player_can_flipsummon(&self, playerid: u8, card: CardId) -> bool {
        self.is_player_can_summon_target(playerid, EFFECT_CANNOT_FLIP_SUMMON, card, 0, playerid)
    }

    // The core passes (e, c, player, sumtype | normal, toplayer, POS_FACEDOWN) to the target.
    pub(crate) fn is_player_can_mset(&self, sumtype: u32, playerid: u8, card: CardId, toplayer: u8) -> bool {
        if !self.field_effects.registry.has_code(EFFECT_CANNOT_MSET) {
            return true;
        }
        let summon = Some((sumtype | SUMMON_TYPE_NORMAL, toplayer, POS_FACEDOWN as u8, None));
        !self
            .filter_player_effect(playerid, EFFECT_CANNOT_MSET, true)
            .into_iter()
            .any(|effect| self.limit_targets(effect, card, playerid, Some(0), summon))
    }

    pub(crate) fn is_player_can_sset(&self, playerid: u8, card: CardId) -> bool {
        self.is_player_can(playerid, EFFECT_CANNOT_SSET, card)
    }

    pub(crate) fn field_is_player_can_release(&self, playerid: u8, card: CardId, reason: u32) -> bool {
        self.is_player_can_for_reason(playerid, EFFECT_CANNOT_RELEASE, card, reason)
    }

    pub(crate) fn is_player_can_send_to_grave(&self, playerid: u8, card: CardId, _reason: u32) -> bool {
        self.is_player_can(playerid, EFFECT_CANNOT_TO_GRAVE, card)
    }

    pub(crate) fn is_player_can_send_to_hand(&self, playerid: u8, card: CardId) -> bool {
        self.is_player_can(playerid, EFFECT_CANNOT_TO_HAND, card)
            && !(self.is_extra_deck_monster(card) && !self.is_player_can_send_to_deck(playerid, card))
    }

    pub(crate) fn is_player_can_send_to_deck(&self, playerid: u8, card: CardId) -> bool {
        self.is_player_can(playerid, EFFECT_CANNOT_TO_DECK, card)
    }

    pub(crate) fn field_is_player_can_remove(&self, playerid: u8, card: CardId, reason: u32) -> bool {
        self.is_player_can_for_reason(playerid, EFFECT_CANNOT_REMOVE, card, reason)
    }

    // field::is_player_can_action: no player effect of `actionlimit` without a target function applies.
    pub(crate) fn is_player_can_action(&self, playerid: u8, actionlimit: u32) -> bool {
        self.filter_player_effect(playerid, actionlimit, true)
            .into_iter()
            .all(|effect| !matches!(self.effects[effect].target, Target::None))
    }

    pub(crate) fn is_player_can_discard_hand(&self, playerid: u8, card: CardId) -> bool {
        self.cards[card].current.location == LOCATION_HAND && self.is_player_can(playerid, EFFECT_CANNOT_DISCARD_HAND, card)
    }

    pub(crate) fn is_player_can_spsummon(
        &self,
        effect: Option<EffectId>,
        sumtype: u32,
        sumpos: u8,
        playerid: u8,
        toplayer: u8,
        card: CardId,
    ) -> bool {
        let target = &self.cards[card];
        if self.is_affected_by_effect(card, EFFECT_CANNOT_SPECIAL_SUMMON).is_some() || target.is_status(STATUS_FORBIDDEN) {
            return false;
        }
        if target.data.card_type & TYPE_TOKEN != 0 && target.current.location & LOCATION_ONFIELD != 0 {
            return false;
        }
        let mut sumpos = sumpos;
        if target.data.card_type & TYPE_LINK != 0 && target.data.card_type & TYPE_MONSTER != 0 {
            sumpos &= POS_FACEUP_ATTACK as u8;
        }
        let sumtype = sumtype | SUMMON_TYPE_SPECIAL;
        if sumpos == 0 || !self.check_cost_condition(card, EFFECT_SPSUMMON_COST, playerid, sumtype) {
            return false;
        }
        if u32::from(sumpos) & POS_FACEDOWN != 0 && self.is_player_affected_by_effect(playerid, EFFECT_DEVINE_LIGHT).is_some() {
            sumpos = (sumpos & POS_FACEUP as u8) | ((sumpos & POS_FACEDOWN as u8) >> 1);
        }
        let forbidden = self
            .filter_player_effect(playerid, EFFECT_CANNOT_SPECIAL_SUMMON, true)
            .into_iter()
            .any(|limit| self.limit_targets(limit, card, playerid, None, Some((sumtype, sumpos, toplayer, effect))));
        if forbidden {
            return false;
        }
        let forcing = self.filter_player_effect(playerid, EFFECT_FORCE_SPSUMMON_POSITION, true);
        if self.force_spsummon_positions(&forcing, card, playerid, sumtype, sumpos, toplayer, effect) == 0 {
            return false;
        }
        self.check_spsummon_once(card, playerid) && self.check_spsummon_counter(playerid, 1)
    }

    // The positions the EFFECT_FORCE_SPSUMMON_POSITION effects leave of `positions`; the targets read `caller` as the summoner.
    pub(crate) fn force_spsummon_positions(
        &self,
        forcing: &EffectSet,
        card: CardId,
        caller: u8,
        sumtype: u32,
        positions: u8,
        toplayer: u8,
        effect: Option<EffectId>,
    ) -> u8 {
        forcing.into_iter().fold(positions, |positions, &forcing| {
            if self.limit_targets(forcing, card, caller, None, Some((sumtype, positions, toplayer, effect))) {
                positions & self.get_value(forcing, card) as u8
            } else {
                positions
            }
        })
    }

    pub(crate) fn check_spsummon_once(&self, card: CardId, playerid: u8) -> bool {
        let code = self.cards[card].spsummon_code;
        code == 0
            || self.core.spsummon_once_map[playerid]
                .iter()
                .all(|&(once, count)| once != code || count == 0)
    }

    // The ++spsummon_once_map[player][code] of the summons, which skip cards without a code to limit.
    pub(crate) fn add_spsummon_once(&mut self, playerid: u8, code: u32) {
        if code == 0 {
            return;
        }
        let map = &mut self.core.spsummon_once_map[playerid];
        match map.iter_mut().find(|(once, _)| *once == code) {
            Some((_, count)) => *count += 1,
            None => map.push((code, 1)),
        }
    }

    // The summons of a group count each code once: the core collects them in a set first.
    pub(crate) fn add_spsummon_once_codes(&mut self, playerid: u8, mut codes: Vec<u32>) {
        codes.sort_unstable();
        codes.dedup();
        for code in codes {
            self.add_spsummon_once(playerid, code);
        }
    }

    // field::set_spsummon_counter without DUEL_CANNOT_SUMMON_OATH_OLD.
    pub(crate) fn set_spsummon_counter(&mut self, playerid: u8) {
        self.core.spsummon_state_count[playerid] += 1;
        for effect in self.field_effects.spsummon_count_eff.iter().copied().collect::<Vec<_>>() {
            let handler = self.get_handler(effect);
            let peffect = &self.effects[effect];
            let range = if playerid == self.cards[handler].current.controler {
                peffect.s_range
            } else {
                peffect.o_range
            };
            if self.is_available(effect) && range != 0 {
                self.cards[handler].spsummon_counter[playerid] += 1;
            }
        }
    }

    pub(crate) fn check_spsummon_counter(&self, playerid: u8, count: u16) -> bool {
        self.field_effects.spsummon_count_eff.iter().all(|&effect| {
            let handler = self.get_handler(effect);
            !self.is_available(effect) || self.cards[handler].spsummon_counter[playerid] + count <= self.effects[effect].int_value() as u16
        })
    }

    // field::check_card_counter: the activity's counters still 0 for the player count once a card of `cards` fails their filter.
    pub(crate) fn check_card_counter(&mut self, cards: CardSet, activity: u8, playerid: u8) {
        for index in 0..self.core.custom_counters.len() {
            let Some(counter) = self.core.custom_counters.get(index).copied() else {
                break;
            };
            let CounterFilter::Card(filter) = counter.filter else { continue };
            if counter.activity != activity || counter.amount.get(usize::from(playerid)) != Some(&0) {
                continue;
            }
            if cards.iter().any(|card| !filter(self, card))
                && let Some(amount) = self
                    .core
                    .custom_counters
                    .get_mut(index)
                    .and_then(|counter| counter.amount.get_mut(usize::from(playerid)))
            {
                *amount += 1;
            }
        }
    }

    pub(crate) fn is_player_can_spsummon_count(&self, playerid: u8, count: u32) -> bool {
        let reason_effect = self.core.reason_effect.get();
        for effect in self.filter_player_effect(playerid, EFFECT_LEFT_SPSUMMON_COUNT, true) {
            if self.get_value_spsummon_count(effect, reason_effect, playerid) < count as i32 {
                return false;
            }
        }
        self.check_spsummon_counter(playerid, count as u8 as u16)
    }

    pub(crate) fn field_is_player_can_draw(&self, playerid: u8) -> bool {
        self.is_player_affected_by_effect(playerid, EFFECT_CANNOT_DRAW).is_none()
    }

    pub(crate) fn adjust_instant(&mut self) {
        self.infos.event_id += 1;
        self.adjust_disable_check_list();
        self.adjust_self_destroy_set();
    }

    pub(crate) fn adjust_all(&mut self) {
        self.infos.event_id += 1;
        self.core.readjust_map.clear();
        self.emplace_process(Adjust::default());
    }

    pub(crate) fn add_to_disable_check_list(&mut self, card: CardId) {
        self.field_effects.disable_check_set.insert(card);
    }

    pub(crate) fn adjust_disable_check_list(&mut self) {
        let cards = std::mem::take(&mut self.field_effects.disable_check_set);
        for card in cards.iter() {
            if !self.cards[card].is_status(STATUS_TO_ENABLE | STATUS_TO_DISABLE) {
                let pre_disable = self.cards[card].get_status(STATUS_DISABLED | STATUS_FORBIDDEN);
                self.refresh_disable_status(card);
                let new_disable = self.cards[card].get_status(STATUS_DISABLED | STATUS_FORBIDDEN);
                if pre_disable != new_disable && self.cards[card].is_status(STATUS_EFFECT_ENABLED) {
                    self.filter_disable_related_cards(card);
                    self.cards[card].set_status(if pre_disable { STATUS_TO_ENABLE } else { STATUS_TO_DISABLE }, true);
                }
            }
            let target = &self.cards[card];
            if target.is_status(STATUS_DISABLED) && target.is_status(STATUS_TO_DISABLE) && !target.is_status(STATUS_TO_ENABLE) {
                self.reset(card, RESET_DISABLE, RESET_EVENT);
            }
            self.cards.set_status(card, STATUS_TO_ENABLE | STATUS_TO_DISABLE, false);
        }
    }

    pub(crate) fn adjust_self_destroy_set(&mut self) {
        if self.core.selfdes_disabled || !self.core.unique_destroy_set.is_empty() || !self.core.self_destroy_set.is_empty() {
            return;
        }
        let mut p = self.infos.turn_player;
        for _ in 0..2 {
            let mut uniq_set = Vec::new();
            for ucard in self.unique_cards(p) {
                let target = &self.cards[ucard];
                if !target.is_position(POS_FACEUP)
                    || !target.get_status(STATUS_EFFECT_ENABLED)
                    || target.get_status(STATUS_DISABLED | STATUS_FORBIDDEN)
                {
                    continue;
                }
                let targets = self.get_unique_target(ucard, p, None);
                if targets.len() > 1 {
                    uniq_set.push(ucard);
                } else {
                    self.cards[ucard].unique_fieldid = targets.get_first().map_or(0, |card| self.cards[card].fieldid);
                }
            }
            uniq_set.sort_by_key(|&card| self.cards[card].fieldid);
            for ucard in uniq_set {
                self.emplace_process(SelfDestroyUnique {
                    step: 0,
                    unique_card: ucard,
                    playerid: p,
                });
                self.core.unique_destroy_set.insert(ucard);
            }
            p = 1 - p;
        }
        let registry = &self.field_effects.registry;
        if !(registry.has_code(EFFECT_SELF_DESTROY) || registry.has_code(EFFECT_SELF_TOGRAVE)) {
            return;
        }
        let mut cards = CardSet::default();
        for p in 0..2 {
            for card in self.player[p].list_mzone.into_iter().flatten() {
                if self.cards[card].is_position(POS_FACEUP) && !self.cards[card].is_status(STATUS_BATTLE_DESTROYED) {
                    cards.insert(card);
                }
            }
            for card in self.player[p].list_szone.into_iter().flatten() {
                if self.cards[card].is_position(POS_FACEUP) {
                    cards.insert(card);
                }
            }
        }
        for card in cards.iter() {
            if self.is_affected_by_effect(card, EFFECT_SELF_DESTROY).is_some() {
                self.core.self_destroy_set.insert(card);
            }
        }
        if self.core.global_flag & GLOBALFLAG_SELF_TOGRAVE != 0 {
            for card in cards.iter() {
                unported!(self, self.is_affected_by_effect(card, EFFECT_SELF_TOGRAVE));
            }
        }
        if !self.core.self_destroy_set.is_empty() {
            self.emplace_process(SelfDestroy::default());
        }
    }

    pub(crate) fn field_is_chain_negatable(&self, chaincount: u8) -> bool {
        let Some(chain) = self.chain_at(chaincount) else { return false };
        if self.effects[chain.triggering_effect].is_flag(EFFECT_FLAG_CANNOT_INACTIVATE) {
            return false;
        }
        !self
            .filter_field_effect(EFFECT_CANNOT_INACTIVATE)
            .into_iter()
            .any(|peff| self.check_value_chain(peff, chaincount))
    }

    pub(crate) fn field_is_chain_disablable(&self, chaincount: u8) -> bool {
        let Some(chain) = self.chain_at(chaincount) else { return false };
        let effect = chain.triggering_effect;
        if !self.cards[self.get_handler(effect)].get_status(STATUS_FORBIDDEN) {
            if self.effects[effect].is_flag(EFFECT_FLAG_CANNOT_DISABLE) {
                return false;
            }
            return !self
                .filter_field_effect(EFFECT_CANNOT_DISEFFECT)
                .into_iter()
                .any(|peff| self.check_value_chain(peff, chaincount));
        }
        true
    }

    pub(crate) fn is_chain_disabled(&mut self, chaincount: u8) -> bool {
        let Some(chain) = self.chain_at(chaincount) else { return false };
        if chain.flag & CHAIN_DISABLE_EFFECT != 0 {
            return true;
        }
        let card = self.get_handler(chain.triggering_effect);
        let chain_id = i32::from(chain.chain_id);
        let disabled: EffectSet = self
            .field_effects
            .registry
            .equal_range(Container::Single, Some(card), EFFECT_DISABLE_CHAIN)
            .collect();
        for effect in disabled {
            if self.get_value(effect, card) == chain_id {
                self.effects[effect].reset_flag |= RESET_CHAIN;
                return true;
            }
        }
        false
    }

    // The chain link `chaincount` (1-based), or the newest for 0; None past the chain.
    pub(crate) fn chain_at(&self, chaincount: u8) -> Option<&Chain> {
        if usize::from(chaincount) > self.core.current_chain.len() {
            return None;
        }
        if chaincount == 0 {
            self.core.current_chain.last()
        } else {
            self.core.current_chain.get(usize::from(chaincount) - 1)
        }
    }

    // field::get_chain: the solving continuous effect's chain for 0, else as chain_at but clamping counts past the chain.
    pub(crate) fn get_chain(&mut self, chaincount: u8) -> Option<&mut Chain> {
        match self.chain_slot(chaincount)? {
            (true, _) => self.core.continuous_chain.last_mut(),
            (false, index) => self.core.current_chain.get_mut(index),
        }
    }

    pub(crate) fn get_chain_ref(&self, chaincount: u8) -> Option<&Chain> {
        match self.chain_slot(chaincount)? {
            (true, _) => self.core.continuous_chain.last(),
            (false, index) => self.core.current_chain.get(index),
        }
    }

    // Whether get_chain reads the continuous chain, else the index into the current chain.
    fn chain_slot(&self, chaincount: u8) -> Option<(bool, usize)> {
        let continuous = self
            .core
            .reason_effect
            .get()
            .is_some_and(|effect| self.effects[effect].effect_type & EFFECT_TYPE_CONTINUOUS != 0);
        if chaincount == 0 && continuous && !self.core.continuous_chain.is_empty() {
            return Some((true, 0));
        }
        let len = self.core.current_chain.len();
        let index = if chaincount == 0 || usize::from(chaincount) > len {
            len
        } else {
            usize::from(chaincount)
        };
        Some((false, index.checked_sub(1)?))
    }

    // field::get_cteffect: whether a plain continuous trap activation could also apply a trigger or quick effect; `store` collects them.
    pub(crate) fn get_cteffect(&mut self, effect: EffectId, playerid: u8, store: bool) -> bool {
        if !self.is_plain_continuous_trap_activation(effect) {
            return false;
        }
        if store {
            self.core.select_chains.clear();
            self.core.select_options.clear();
        }
        for (code, feffect) in self.cteffect_candidates(effect) {
            for event in self.cteffect_events(code) {
                if self.get_cteffect_evt(feffect, playerid, event, store) && !store {
                    return true;
                }
            }
        }
        store && !self.core.select_chains.is_empty()
    }

    fn is_plain_continuous_trap_activation(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        self.cards[self.get_handler(effect)].data.card_type == TYPE_TRAP | TYPE_CONTINUOUS
            && peffect.effect_type & EFFECT_TYPE_ACTIVATE != 0
            && peffect.code == EVENT_FREE_CHAIN
            && peffect.cost.is_none()
            && matches!(peffect.target, Target::None)
            && peffect.operation.is_none()
    }

    // The trigger and quick effects of the trap's field container that are in range, with their codes.
    fn cteffect_candidates(&self, effect: EffectId) -> List<(u32, EffectId), 4> {
        let handler = self.get_handler(effect);
        let actions = EFFECT_TYPE_TRIGGER_F | EFFECT_TYPE_TRIGGER_O | EFFECT_TYPE_QUICK_O;
        self.field_effects
            .registry
            .all(Container::Field, Some(handler))
            .filter(|&(_, feffect)| self.effects[feffect].effect_type & actions != 0 && self.in_range(feffect, handler))
            .collect()
    }

    // The events a candidate of `code` is checked against: the free chain or the current phase, else the pending ones.
    fn cteffect_events(&mut self, code: u32) -> List<Event, 4> {
        if code == EVENT_FREE_CHAIN || code == EVENT_PHASE + u32::from(self.infos.phase) {
            self.nil_event.event_code = code;
            return List::from_iter([self.nil_event]);
        }
        let core = &self.core;
        core.point_event
            .iter()
            .chain(&core.instant_event)
            .filter(|event| event.event_code == code)
            .copied()
            .collect()
    }

    fn get_cteffect_evt(&mut self, feffect: EffectId, playerid: u8, event: Event, store: bool) -> bool {
        if !self.is_activateable(feffect, playerid, &event, false, false, false, false, true) {
            return false;
        }
        if store {
            let mut chain = Chain::new(feffect);
            chain.evt = event;
            self.core.select_chains.push(chain);
            let description = self.effects[feffect].description;
            self.core.select_options.push(description);
        }
        true
    }

    pub(crate) fn check_cteffect_hint(&mut self, effect: EffectId, playerid: u8) -> bool {
        if !self.is_plain_continuous_trap_activation(effect) {
            return false;
        }
        for (code, feffect) in self.cteffect_candidates(effect) {
            for event in self.cteffect_events(code) {
                if self.get_cteffect_evt(feffect, playerid, event, false) && (code != EVENT_FREE_CHAIN || self.check_hint_timing(feffect)) {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn check_hint_timing(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        let p = self.get_handler_player(effect);
        let (own, other) = if p == 0 {
            (self.core.hint_timing[0], self.core.hint_timing[1])
        } else {
            (self.core.hint_timing[1], self.core.hint_timing[0])
        };
        peffect.hint_timing[0] & own != 0 || peffect.hint_timing[1] & other != 0
    }

    pub(crate) fn check_nonpublic_trigger(&mut self, chain: &mut Chain) -> bool {
        let effect = chain.triggering_effect;
        let phandler = self.get_handler(effect);
        let peffect = &self.effects[effect];
        let hand_or_deck = u16::from(LOCATION_HAND | LOCATION_DECK);
        if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
            && ((peffect.effect_type & EFFECT_TYPE_SINGLE != 0
                && !peffect.is_flag(EFFECT_FLAG_SINGLE_RANGE)
                && self.is_has_relation(phandler, effect, chain.chain_id)
                && chain.triggering_location & hand_or_deck != 0)
                || peffect.range & hand_or_deck != 0)
        {
            chain.flag |= CHAIN_HAND_TRIGGER;
            self.core.new_ochain_h.push(*chain);
            let range = peffect.range;
            if !self.is_flag(DUEL_TCG_SEGOC_NONPUBLIC)
                && ((chain.triggering_location == u16::from(LOCATION_HAND) && self.cards[phandler].is_position(POS_FACEDOWN))
                    || chain.triggering_location == u16::from(LOCATION_DECK)
                    || (range != 0 && !self.in_range_chain(effect, chain)))
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn check_trigger_effect(&self, chain: &Chain) -> bool {
        let effect = chain.triggering_effect;
        let phandler = self.get_handler(effect);
        let has_relation = self.is_has_relation(phandler, effect, chain.chain_id);
        if self.is_flag(DUEL_TRIGGER_ONLY_IN_LOCATION) {
            return has_relation;
        }
        let peffect = &self.effects[effect];
        if peffect.effect_type & EFFECT_TYPE_FIELD != 0 && !has_relation {
            return false;
        }
        if peffect.code == EVENT_FLIP && self.infos.phase == PHASE_DAMAGE {
            return true;
        }
        if self.is_flag(DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE) {
            return true;
        }
        let card = &self.cards[phandler];
        let hidden = u16::from(LOCATION_DECK | LOCATION_HAND | LOCATION_EXTRA);
        if card.current.location & LOCATION_DECK != 0 && chain.triggering_location != u16::from(LOCATION_DECK) {
            return false;
        }
        if chain.triggering_location & hidden != 0 && u32::from(chain.triggering_position) & POS_FACEDOWN != 0 {
            return true;
        }
        u16::from(card.current.location) & hidden == 0 || card.is_position(POS_FACEUP)
    }

    pub(crate) fn check_spself_from_hand_trigger(&self, chain: &Chain) -> bool {
        if self.effects.status(chain.triggering_effect) & EFFECT_STATUS_SUMMON_SELF != 0 && chain.flag & CHAIN_HAND_TRIGGER != 0 {
            let tp = chain.triggering_player;
            return !self.core.current_chain.iter().any(|other| {
                other.triggering_player == tp
                    && self.effects.status(other.triggering_effect) & EFFECT_STATUS_SUMMON_SELF != 0
                    && other.flag & CHAIN_HAND_TRIGGER != 0
            });
        }
        true
    }
}
