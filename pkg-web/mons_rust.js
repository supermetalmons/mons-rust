let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

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

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

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
/**
* @param {string} fen_w
* @param {string} fen_b
* @param {string} flat_moves_string_w
* @param {string} flat_moves_string_b
* @returns {string}
*/
export function winner(fen_w, fen_b, flat_moves_string_w, flat_moves_string_b) {
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
}

/**
*/
export const AvailableMoveKind = Object.freeze({ MonMove:0,"0":"MonMove",ManaMove:1,"1":"ManaMove",Action:2,"2":"Action",Potion:3,"3":"Potion", });
/**
*/
export const MonKind = Object.freeze({ Demon:0,"0":"Demon",Drainer:1,"1":"Drainer",Angel:2,"2":"Angel",Spirit:3,"3":"Spirit",Mystic:4,"4":"Mystic", });
/**
*/
export const Color = Object.freeze({ White:0,"0":"White",Black:1,"1":"Black", });
/**
*/
export const Modifier = Object.freeze({ SelectPotion:0,"0":"SelectPotion",SelectBomb:1,"1":"SelectBomb",Cancel:2,"2":"Cancel", });
/**
*/
export const Consumable = Object.freeze({ Potion:0,"0":"Potion",Bomb:1,"1":"Bomb",BombOrPotion:2,"2":"BombOrPotion", });
/**
*/
export const NextInputKind = Object.freeze({ MonMove:0,"0":"MonMove",ManaMove:1,"1":"ManaMove",MysticAction:2,"2":"MysticAction",DemonAction:3,"3":"DemonAction",DemonAdditionalStep:4,"4":"DemonAdditionalStep",SpiritTargetCapture:5,"5":"SpiritTargetCapture",SpiritTargetMove:6,"6":"SpiritTargetMove",SelectConsumable:7,"7":"SelectConsumable",BombAttack:8,"8":"BombAttack", });

const LocationFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_location_free(ptr >>> 0));
/**
*/
export class Location {

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

const MonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mon_free(ptr >>> 0));
/**
*/
export class Mon {

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

const MonsGameModelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_monsgamemodel_free(ptr >>> 0));
/**
*/
export class MonsGameModel {

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
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, maybe_memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedInt32Memory0 = null;
    cachedUint8Memory0 = null;


    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;

    if (typeof input === 'undefined') {
        input = new URL('mons_rust_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await input, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync }
export default __wbg_init;
