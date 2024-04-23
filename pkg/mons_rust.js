let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder, TextEncoder } = require(`util`);

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getUint32Memory0();
    for (let i = 0; i < array.length; i++) {
        mem[ptr / 4 + i] = addHeapObject(array[i]);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}
/**
* @param {string} fen_w
* @param {string} fen_b
* @param {string} flat_moves_string_w
* @param {string} flat_moves_string_b
* @returns {string}
*/
module.exports.winner = function(fen_w, fen_b, flat_moves_string_w, flat_moves_string_b) {
    let deferred5_0;
    let deferred5_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(fen_w, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(fen_b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(flat_moves_string_w, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(flat_moves_string_b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        wasm.winner(retptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        deferred5_0 = r0;
        deferred5_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
};

/**
*/
module.exports.AvailableMoveKind = Object.freeze({ MonMove:0,"0":"MonMove",ManaMove:1,"1":"ManaMove",Action:2,"2":"Action",Potion:3,"3":"Potion", });
/**
*/
module.exports.SquareModelKind = Object.freeze({ Regular:0,"0":"Regular",ConsumableBase:1,"1":"ConsumableBase",SupermanaBase:2,"2":"SupermanaBase",ManaBase:3,"3":"ManaBase",ManaPool:4,"4":"ManaPool",MonBase:5,"5":"MonBase", });
/**
*/
module.exports.OutputModelKind = Object.freeze({ InvalidInput:0,"0":"InvalidInput",LocationsToStartFrom:1,"1":"LocationsToStartFrom",NextInputOptions:2,"2":"NextInputOptions",Events:3,"3":"Events", });
/**
*/
module.exports.Modifier = Object.freeze({ SelectPotion:0,"0":"SelectPotion",SelectBomb:1,"1":"SelectBomb",Cancel:2,"2":"Cancel", });
/**
*/
module.exports.EventModelKind = Object.freeze({ MonMove:0,"0":"MonMove",ManaMove:1,"1":"ManaMove",ManaScored:2,"2":"ManaScored",MysticAction:3,"3":"MysticAction",DemonAction:4,"4":"DemonAction",DemonAdditionalStep:5,"5":"DemonAdditionalStep",SpiritTargetMove:6,"6":"SpiritTargetMove",PickupBomb:7,"7":"PickupBomb",PickupPotion:8,"8":"PickupPotion",PickupMana:9,"9":"PickupMana",MonFainted:10,"10":"MonFainted",ManaDropped:11,"11":"ManaDropped",SupermanaBackToBase:12,"12":"SupermanaBackToBase",BombAttack:13,"13":"BombAttack",MonAwake:14,"14":"MonAwake",BombExplosion:15,"15":"BombExplosion",NextTurn:16,"16":"NextTurn",GameOver:17,"17":"GameOver", });
/**
*/
module.exports.Color = Object.freeze({ White:0,"0":"White",Black:1,"1":"Black", });
/**
*/
module.exports.ItemModelKind = Object.freeze({ Mon:0,"0":"Mon",Mana:1,"1":"Mana",MonWithMana:2,"2":"MonWithMana",MonWithConsumable:3,"3":"MonWithConsumable",Consumable:4,"4":"Consumable", });
/**
*/
module.exports.MonKind = Object.freeze({ Demon:0,"0":"Demon",Drainer:1,"1":"Drainer",Angel:2,"2":"Angel",Spirit:3,"3":"Spirit",Mystic:4,"4":"Mystic", });
/**
*/
module.exports.NextInputKind = Object.freeze({ MonMove:0,"0":"MonMove",ManaMove:1,"1":"ManaMove",MysticAction:2,"2":"MysticAction",DemonAction:3,"3":"DemonAction",DemonAdditionalStep:4,"4":"DemonAdditionalStep",SpiritTargetCapture:5,"5":"SpiritTargetCapture",SpiritTargetMove:6,"6":"SpiritTargetMove",SelectConsumable:7,"7":"SelectConsumable",BombAttack:8,"8":"BombAttack", });
/**
*/
module.exports.Consumable = Object.freeze({ Potion:0,"0":"Potion",Bomb:1,"1":"Bomb",BombOrPotion:2,"2":"BombOrPotion", });
/**
*/
module.exports.ManaKind = Object.freeze({ Regular:0,"0":"Regular",Supermana:1,"1":"Supermana", });

const EventModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_eventmodel_free(ptr >>> 0));
/**
*/
class EventModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(EventModel.prototype);
        obj.__wbg_ptr = ptr;
        EventModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EventModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_eventmodel_free(ptr);
    }
}
module.exports.EventModel = EventModel;

const ItemModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_itemmodel_free(ptr >>> 0));
/**
*/
class ItemModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ItemModel.prototype);
        obj.__wbg_ptr = ptr;
        ItemModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ItemModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_itemmodel_free(ptr);
    }
}
module.exports.ItemModel = ItemModel;

const LocationFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_location_free(ptr >>> 0));
/**
*/
class Location {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Location.prototype);
        obj.__wbg_ptr = ptr;
        LocationFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof Location)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LocationFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_location_free(ptr);
    }
    /**
    * @returns {number}
    */
    get i() {
        const ret = wasm.__wbg_get_location_i(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set i(arg0) {
        wasm.__wbg_set_location_i(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get j() {
        const ret = wasm.__wbg_get_location_j(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set j(arg0) {
        wasm.__wbg_set_location_j(this.__wbg_ptr, arg0);
    }
    /**
    * @param {number} i
    * @param {number} j
    */
    constructor(i, j) {
        const ret = wasm.location_new(i, j);
        this.__wbg_ptr = ret >>> 0;
        return this;
    }
}
module.exports.Location = Location;

const ManaModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_manamodel_free(ptr >>> 0));
/**
*/
class ManaModel {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ManaModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_manamodel_free(ptr);
    }
    /**
    * @returns {ManaKind}
    */
    get kind() {
        const ret = wasm.__wbg_get_manamodel_kind(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {ManaKind} arg0
    */
    set kind(arg0) {
        wasm.__wbg_set_manamodel_kind(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {Color}
    */
    get color() {
        const ret = wasm.__wbg_get_manamodel_color(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {Color} arg0
    */
    set color(arg0) {
        wasm.__wbg_set_manamodel_color(this.__wbg_ptr, arg0);
    }
}
module.exports.ManaModel = ManaModel;

const MonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mon_free(ptr >>> 0));
/**
*/
class Mon {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Mon.prototype);
        obj.__wbg_ptr = ptr;
        MonFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mon_free(ptr);
    }
    /**
    * @returns {MonKind}
    */
    get kind() {
        const ret = wasm.__wbg_get_mon_kind(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {MonKind} arg0
    */
    set kind(arg0) {
        wasm.__wbg_set_mon_kind(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {Color}
    */
    get color() {
        const ret = wasm.__wbg_get_mon_color(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {Color} arg0
    */
    set color(arg0) {
        wasm.__wbg_set_mon_color(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get cooldown() {
        const ret = wasm.__wbg_get_mon_cooldown(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set cooldown(arg0) {
        wasm.__wbg_set_mon_cooldown(this.__wbg_ptr, arg0);
    }
    /**
    * @param {MonKind} kind
    * @param {Color} color
    * @param {number} cooldown
    * @returns {Mon}
    */
    static new(kind, color, cooldown) {
        const ret = wasm.mon_new(kind, color, cooldown);
        return Mon.__wrap(ret);
    }
    /**
    * @returns {boolean}
    */
    is_fainted() {
        const ret = wasm.mon_is_fainted(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
    */
    faint() {
        wasm.mon_faint(this.__wbg_ptr);
    }
    /**
    */
    decrease_cooldown() {
        wasm.mon_decrease_cooldown(this.__wbg_ptr);
    }
}
module.exports.Mon = Mon;

const MonsGameModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_monsgamemodel_free(ptr >>> 0));
/**
*/
class MonsGameModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MonsGameModel.prototype);
        obj.__wbg_ptr = ptr;
        MonsGameModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MonsGameModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_monsgamemodel_free(ptr);
    }
    /**
    * @returns {MonsGameModel}
    */
    static new() {
        const ret = wasm.monsgamemodel_new();
        return MonsGameModel.__wrap(ret);
    }
    /**
    * @param {string} fen
    * @returns {MonsGameModel | undefined}
    */
    static from_fen(fen) {
        const ptr0 = passStringToWasm0(fen, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.monsgamemodel_from_fen(ptr0, len0);
        return ret === 0 ? undefined : MonsGameModel.__wrap(ret);
    }
    /**
    * @returns {string}
    */
    fen() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.monsgamemodel_fen(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
    * @param {(Location)[]} locations
    * @param {Modifier | undefined} [modifier]
    * @returns {OutputModel}
    */
    process_input(locations, modifier) {
        const ptr0 = passArrayJsValueToWasm0(locations, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.monsgamemodel_process_input(this.__wbg_ptr, ptr0, len0, isLikeNone(modifier) ? 3 : modifier);
        return OutputModel.__wrap(ret);
    }
    /**
    * @param {string} input_fen
    * @returns {OutputModel}
    */
    process_input_fen(input_fen) {
        const ptr0 = passStringToWasm0(input_fen, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.monsgamemodel_process_input_fen(this.__wbg_ptr, ptr0, len0);
        return OutputModel.__wrap(ret);
    }
    /**
    * @param {Location} at
    * @returns {ItemModel | undefined}
    */
    item(at) {
        _assertClass(at, Location);
        var ptr0 = at.__destroy_into_raw();
        const ret = wasm.monsgamemodel_item(this.__wbg_ptr, ptr0);
        return ret === 0 ? undefined : ItemModel.__wrap(ret);
    }
    /**
    * @param {Location} at
    * @returns {SquareModel}
    */
    square(at) {
        _assertClass(at, Location);
        var ptr0 = at.__destroy_into_raw();
        const ret = wasm.monsgamemodel_square(this.__wbg_ptr, ptr0);
        return SquareModel.__wrap(ret);
    }
    /**
    * @param {string} other_fen
    * @returns {boolean}
    */
    is_later_than(other_fen) {
        const ptr0 = passStringToWasm0(other_fen, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.monsgamemodel_is_later_than(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
    * @returns {Color}
    */
    active_color() {
        const ret = wasm.monsgamemodel_active_color(this.__wbg_ptr);
        return ret;
    }
    /**
    * @returns {Color | undefined}
    */
    winner_color() {
        const ret = wasm.monsgamemodel_winner_color(this.__wbg_ptr);
        return ret === 2 ? undefined : ret;
    }
    /**
    * @returns {number}
    */
    black_score() {
        const ret = wasm.monsgamemodel_black_score(this.__wbg_ptr);
        return ret;
    }
    /**
    * @returns {number}
    */
    white_score() {
        const ret = wasm.monsgamemodel_white_score(this.__wbg_ptr);
        return ret;
    }
    /**
    * @returns {Int32Array}
    */
    available_move_kinds() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.monsgamemodel_available_move_kinds(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayI32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {(Location)[]}
    */
    locations_with_content() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.monsgamemodel_locations_with_content(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.MonsGameModel = MonsGameModel;

const NextInputModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_nextinputmodel_free(ptr >>> 0));
/**
*/
class NextInputModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NextInputModel.prototype);
        obj.__wbg_ptr = ptr;
        NextInputModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NextInputModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_nextinputmodel_free(ptr);
    }
    /**
    * @returns {Location | undefined}
    */
    get location() {
        const ret = wasm.__wbg_get_nextinputmodel_location(this.__wbg_ptr);
        return ret === 0 ? undefined : Location.__wrap(ret);
    }
    /**
    * @param {Location | undefined} [arg0]
    */
    set location(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, Location);
            ptr0 = arg0.__destroy_into_raw();
        }
        wasm.__wbg_set_nextinputmodel_location(this.__wbg_ptr, ptr0);
    }
    /**
    * @returns {Modifier | undefined}
    */
    get modifier() {
        const ret = wasm.__wbg_get_nextinputmodel_modifier(this.__wbg_ptr);
        return ret === 3 ? undefined : ret;
    }
    /**
    * @param {Modifier | undefined} [arg0]
    */
    set modifier(arg0) {
        wasm.__wbg_set_nextinputmodel_modifier(this.__wbg_ptr, isLikeNone(arg0) ? 3 : arg0);
    }
    /**
    * @returns {NextInputKind}
    */
    get kind() {
        const ret = wasm.__wbg_get_nextinputmodel_kind(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {NextInputKind} arg0
    */
    set kind(arg0) {
        wasm.__wbg_set_nextinputmodel_kind(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {ItemModel | undefined}
    */
    get actor_mon_item() {
        const ret = wasm.__wbg_get_nextinputmodel_actor_mon_item(this.__wbg_ptr);
        return ret === 0 ? undefined : ItemModel.__wrap(ret);
    }
    /**
    * @param {ItemModel | undefined} [arg0]
    */
    set actor_mon_item(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, ItemModel);
            ptr0 = arg0.__destroy_into_raw();
        }
        wasm.__wbg_set_nextinputmodel_actor_mon_item(this.__wbg_ptr, ptr0);
    }
}
module.exports.NextInputModel = NextInputModel;

const OutputModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_outputmodel_free(ptr >>> 0));
/**
*/
class OutputModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OutputModel.prototype);
        obj.__wbg_ptr = ptr;
        OutputModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OutputModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_outputmodel_free(ptr);
    }
    /**
    * @returns {OutputModelKind}
    */
    get kind() {
        const ret = wasm.__wbg_get_outputmodel_kind(this.__wbg_ptr);
        return ret;
    }
    /**
    * @param {OutputModelKind} arg0
    */
    set kind(arg0) {
        wasm.__wbg_set_outputmodel_kind(this.__wbg_ptr, arg0);
    }
    /**
    * @returns {(Location)[]}
    */
    locations() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.outputmodel_locations(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {(NextInputModel)[]}
    */
    next_inputs() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.outputmodel_next_inputs(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {(EventModel)[]}
    */
    events() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.outputmodel_events(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {string}
    */
    input_fen() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.outputmodel_input_fen(retptr, this.__wbg_ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
module.exports.OutputModel = OutputModel;

const SquareModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_squaremodel_free(ptr >>> 0));
/**
*/
class SquareModel {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SquareModel.prototype);
        obj.__wbg_ptr = ptr;
        SquareModelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SquareModelFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_squaremodel_free(ptr);
    }
}
module.exports.SquareModel = SquareModel;

module.exports.__wbg_nextinputmodel_new = function(arg0) {
    const ret = NextInputModel.__wrap(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_eventmodel_new = function(arg0) {
    const ret = EventModel.__wrap(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_location_new = function(arg0) {
    const ret = Location.__wrap(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_location_unwrap = function(arg0) {
    const ret = Location.__unwrap(takeObject(arg0));
    return ret;
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'mons_rust_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

