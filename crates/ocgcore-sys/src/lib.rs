use std::ffi::{c_char, c_int, c_void};

pub type Duel = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CardData {
    pub code: u32,
    pub alias: u32,
    pub setcodes: *mut u16,
    pub card_type: u32,
    pub level: u32,
    pub attribute: u32,
    pub race: u64,
    pub attack: i32,
    pub defense: i32,
    pub lscale: u32,
    pub rscale: u32,
    pub link_marker: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub starting_lp: u32,
    pub starting_draw_count: u32,
    pub draw_count_per_turn: u32,
}

pub type DataReader = Option<unsafe extern "C" fn(payload: *mut c_void, code: u32, data: *mut CardData)>;
pub type DataReaderDone = Option<unsafe extern "C" fn(payload: *mut c_void, data: *mut CardData)>;
pub type ScriptReader = Option<unsafe extern "C" fn(payload: *mut c_void, duel: Duel, name: *const c_char) -> c_int>;
pub type LogHandler = Option<unsafe extern "C" fn(payload: *mut c_void, string: *const c_char, log_type: c_int)>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DuelOptions {
    pub seed: [u64; 4],
    pub flags: u64,
    pub team1: Player,
    pub team2: Player,
    pub card_reader: DataReader,
    pub card_reader_payload: *mut c_void,
    pub script_reader: ScriptReader,
    pub script_reader_payload: *mut c_void,
    pub log_handler: LogHandler,
    pub log_handler_payload: *mut c_void,
    pub card_reader_done: DataReaderDone,
    pub card_reader_done_payload: *mut c_void,
    pub enable_unsafe_libraries: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NewCardInfo {
    pub team: u8,
    pub duelist: u8,
    pub code: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub pos: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QueryInfo {
    pub flags: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub overlay_seq: u32,
}

pub const DUEL_CREATION_SUCCESS: c_int = 0;
pub const DUEL_STATUS_END: c_int = 0;
pub const DUEL_STATUS_AWAITING: c_int = 1;
pub const DUEL_STATUS_CONTINUE: c_int = 2;
pub const LOG_TYPE_ERROR: c_int = 0;
pub const LOG_TYPE_FOR_DEBUG: c_int = 2;

pub const LOCATION_DECK: u32 = 0x01;
pub const LOCATION_MZONE: u8 = 0x04;
pub const LOCATION_SZONE: u8 = 0x08;
pub const LOCATION_EXTRA: u32 = 0x40;
pub const POS_FACEDOWN_DEFENSE: u32 = 0x8;

pub const TYPE_MONSTER: u32 = 0x1;
pub const TYPE_FUSION: u32 = 0x40;
pub const TYPE_SYNCHRO: u32 = 0x2000;
pub const TYPE_TOKEN: u32 = 0x4000;
pub const TYPE_XYZ: u32 = 0x80_0000;
pub const TYPE_LINK: u32 = 0x400_0000;
pub const TYPE_EXTRA: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

// DUEL_PZONE | DUEL_EMZONE | DUEL_FSX_MMZONE | DUEL_TRAP_MONSTERS_NOT_USE_ZONE | DUEL_TRIGGER_ONLY_IN_LOCATION
pub const DUEL_MODE_MR5: u64 = 0x800 | 0x2000 | 0x4000 | 0x8000 | 0x2_0000;

pub const MSG_RETRY: u8 = 1;
pub const MSG_WIN: u8 = 5;
pub const MSG_SELECT_BATTLECMD: u8 = 10;
pub const MSG_SELECT_IDLECMD: u8 = 11;
pub const MSG_SELECT_EFFECTYN: u8 = 12;
pub const MSG_SELECT_YESNO: u8 = 13;
pub const MSG_SELECT_OPTION: u8 = 14;
pub const MSG_SELECT_CARD: u8 = 15;
pub const MSG_SELECT_CHAIN: u8 = 16;
pub const MSG_SELECT_PLACE: u8 = 18;
pub const MSG_SELECT_POSITION: u8 = 19;
pub const MSG_SELECT_TRIBUTE: u8 = 20;
pub const MSG_SORT_CHAIN: u8 = 21;
pub const MSG_SELECT_COUNTER: u8 = 22;
pub const MSG_SELECT_SUM: u8 = 23;
pub const MSG_SELECT_DISFIELD: u8 = 24;
pub const MSG_SORT_CARD: u8 = 25;
pub const MSG_SELECT_UNSELECT_CARD: u8 = 26;
pub const MSG_NEW_TURN: u8 = 40;
pub const MSG_ROCK_PAPER_SCISSORS: u8 = 132;
pub const MSG_ANNOUNCE_RACE: u8 = 140;
pub const MSG_ANNOUNCE_ATTRIB: u8 = 141;
pub const MSG_ANNOUNCE_CARD: u8 = 142;
pub const MSG_ANNOUNCE_NUMBER: u8 = 143;

pub const OPCODE_ADD: u64 = 0x4000_0000_0000_0000;
pub const OPCODE_SUB: u64 = 0x4000_0001_0000_0000;
pub const OPCODE_MUL: u64 = 0x4000_0002_0000_0000;
pub const OPCODE_DIV: u64 = 0x4000_0003_0000_0000;
pub const OPCODE_AND: u64 = 0x4000_0004_0000_0000;
pub const OPCODE_OR: u64 = 0x4000_0005_0000_0000;
pub const OPCODE_NEG: u64 = 0x4000_0006_0000_0000;
pub const OPCODE_NOT: u64 = 0x4000_0007_0000_0000;
pub const OPCODE_BAND: u64 = 0x4000_0008_0000_0000;
pub const OPCODE_BOR: u64 = 0x4000_0009_0000_0000;
pub const OPCODE_BNOT: u64 = 0x4000_0010_0000_0000;
pub const OPCODE_BXOR: u64 = 0x4000_0011_0000_0000;
pub const OPCODE_LSHIFT: u64 = 0x4000_0012_0000_0000;
pub const OPCODE_RSHIFT: u64 = 0x4000_0013_0000_0000;
pub const OPCODE_ALLOW_ALIASES: u64 = 0x4000_0014_0000_0000;
pub const OPCODE_ALLOW_TOKENS: u64 = 0x4000_0015_0000_0000;
pub const OPCODE_ISCODE: u64 = 0x4000_0100_0000_0000;
pub const OPCODE_ISSETCARD: u64 = 0x4000_0101_0000_0000;
pub const OPCODE_ISTYPE: u64 = 0x4000_0102_0000_0000;
pub const OPCODE_ISRACE: u64 = 0x4000_0103_0000_0000;
pub const OPCODE_ISATTRIBUTE: u64 = 0x4000_0104_0000_0000;
pub const OPCODE_GETCODE: u64 = 0x4000_0105_0000_0000;
pub const OPCODE_GETTYPE: u64 = 0x4000_0107_0000_0000;
pub const OPCODE_GETRACE: u64 = 0x4000_0108_0000_0000;
pub const OPCODE_GETATTRIBUTE: u64 = 0x4000_0109_0000_0000;

unsafe extern "C" {
    pub fn OCG_GetVersion(major: *mut c_int, minor: *mut c_int);
    pub fn OCG_CreateDuel(out_duel: *mut Duel, options: *const DuelOptions) -> c_int;
    pub fn OCG_DestroyDuel(duel: Duel);
    pub fn OCG_DuelNewCard(duel: Duel, info: *const NewCardInfo);
    pub fn OCG_StartDuel(duel: Duel);
    pub fn OCG_DuelProcess(duel: Duel) -> c_int;
    pub fn OCG_DuelGetMessage(duel: Duel, length: *mut u32) -> *mut c_void;
    pub fn OCG_DuelSetResponse(duel: Duel, buffer: *const c_void, length: u32);
    pub fn OCG_LoadScript(duel: Duel, buffer: *const c_char, length: u32, name: *const c_char) -> c_int;
    pub fn OCG_DuelQueryCount(duel: Duel, team: u8, loc: u32) -> u32;
    pub fn OCG_DuelQuery(duel: Duel, length: *mut u32, info: *const QueryInfo) -> *mut c_void;
    pub fn OCG_DuelQueryLocation(duel: Duel, length: *mut u32, info: *const QueryInfo) -> *mut c_void;
    pub fn OCG_DuelQueryField(duel: Duel, length: *mut u32) -> *mut c_void;
}
