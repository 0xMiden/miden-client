// Copyright 2020 Google LLC. All rights reserved.
import createSymbolsBackend from './SymbolsBackend.js';

// Copyright 2020 Google LLC. All rights reserved.
class MemorySlice {
    begin;
    buffer;
    constructor(buffer, begin) {
        this.begin = begin;
        this.buffer = buffer;
    }
    merge(other) {
        if (other.begin < this.begin) {
            return other.merge(this);
        }
        if (other.begin > this.end) {
            throw new Error('Slices are not contiguous');
        }
        if (other.end <= this.end) {
            return this;
        }
        const newBuffer = new Uint8Array(other.end - this.begin);
        newBuffer.set(new Uint8Array(this.buffer), 0);
        newBuffer.set(new Uint8Array(other.buffer, this.end - other.begin), this.length);
        return new MemorySlice(newBuffer.buffer, this.begin);
    }
    contains(offset) {
        return this.begin <= offset && offset < this.end;
    }
    get length() {
        return this.buffer.byteLength;
    }
    get end() {
        return this.length + this.begin;
    }
    view(begin, length) {
        return new DataView(this.buffer, begin - this.begin, length);
    }
}
class PageStore {
    slices = [];
    // Returns the highest index |i| such that |slices[i].start <= offset|, or -1 if there is no such |i|.
    findSliceIndex(offset) {
        let begin = 0;
        let end = this.slices.length;
        while (begin < end) {
            const idx = Math.floor((end + begin) / 2);
            const pivot = this.slices[idx];
            if (offset < pivot.begin) {
                end = idx;
            }
            else {
                begin = idx + 1;
            }
        }
        return begin - 1;
    }
    findSlice(offset) {
        return this.getSlice(this.findSliceIndex(offset), offset);
    }
    getSlice(index, offset) {
        if (index < 0) {
            return null;
        }
        const candidate = this.slices[index];
        return candidate?.contains(offset) ? candidate : null;
    }
    addSlice(buffer, begin) {
        let slice = new MemorySlice(Array.isArray(buffer) ? new Uint8Array(buffer).buffer : buffer, begin);
        let leftPosition = this.findSliceIndex(slice.begin - 1);
        const leftOverlap = this.getSlice(leftPosition, slice.begin - 1);
        if (leftOverlap) {
            slice = slice.merge(leftOverlap);
        }
        else {
            leftPosition++;
        }
        const rightPosition = this.findSliceIndex(slice.end);
        const rightOverlap = this.getSlice(rightPosition, slice.end);
        if (rightOverlap) {
            slice = slice.merge(rightOverlap);
        }
        this.slices.splice(leftPosition, // Insert to the right if no overlap
        rightPosition - leftPosition + 1, // Delete one additional slice if overlapping on the left
        slice);
        return slice;
    }
}
class WasmMemoryView {
    wasm;
    pages = new PageStore();
    static PAGE_SIZE = 4096;
    constructor(wasm) {
        this.wasm = wasm;
    }
    page(byteOffset, byteLength) {
        const mask = WasmMemoryView.PAGE_SIZE - 1;
        const offset = byteOffset & mask;
        const page = byteOffset - offset;
        const rangeEnd = byteOffset + byteLength;
        const count = 1 + Math.ceil((rangeEnd - (rangeEnd & mask) - page) / WasmMemoryView.PAGE_SIZE);
        return { page, offset, count };
    }
    getPages(page, count) {
        if (page & (WasmMemoryView.PAGE_SIZE - 1)) {
            throw new Error('Not a valid page');
        }
        let slice = this.pages.findSlice(page);
        const size = WasmMemoryView.PAGE_SIZE * count;
        if (!slice || slice.length < count * WasmMemoryView.PAGE_SIZE) {
            const data = this.wasm.readMemory(page, size);
            if (data.byteOffset !== 0 || data.byteLength !== data.buffer.byteLength) {
                throw new Error('Did not expect a partial memory view');
            }
            slice = this.pages.addSlice(data.buffer, page);
        }
        return slice.view(page, size);
    }
    getFloat32(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 4);
        const view = this.getPages(page, count);
        return view.getFloat32(offset, littleEndian);
    }
    getFloat64(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 8);
        const view = this.getPages(page, count);
        return view.getFloat64(offset, littleEndian);
    }
    getInt8(byteOffset) {
        const { offset, page, count } = this.page(byteOffset, 1);
        const view = this.getPages(page, count);
        return view.getInt8(offset);
    }
    getInt16(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 2);
        const view = this.getPages(page, count);
        return view.getInt16(offset, littleEndian);
    }
    getInt32(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 4);
        const view = this.getPages(page, count);
        return view.getInt32(offset, littleEndian);
    }
    getUint8(byteOffset) {
        const { offset, page, count } = this.page(byteOffset, 1);
        const view = this.getPages(page, count);
        return view.getUint8(offset);
    }
    getUint16(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 2);
        const view = this.getPages(page, count);
        return view.getUint16(offset, littleEndian);
    }
    getUint32(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 4);
        const view = this.getPages(page, count);
        return view.getUint32(offset, littleEndian);
    }
    getBigInt64(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 8);
        const view = this.getPages(page, count);
        return view.getBigInt64(offset, littleEndian);
    }
    getBigUint64(byteOffset, littleEndian) {
        const { offset, page, count } = this.page(byteOffset, 8);
        const view = this.getPages(page, count);
        return view.getBigUint64(offset, littleEndian);
    }
    asDataView(byteOffset, byteLength) {
        const { offset, page, count } = this.page(byteOffset, byteLength);
        const view = this.getPages(page, count);
        return new DataView(view.buffer, view.byteOffset + offset, byteLength);
    }
}
class CXXValue {
    location;
    type;
    data;
    memoryOrDataView;
    wasm;
    typeMap;
    memoryView;
    membersMap;
    objectStore;
    objectId;
    displayValue;
    memoryAddress;
    constructor(objectStore, wasm, memoryView, location, type, typeMap, data, displayValue, memoryAddress) {
        if (!location && !data) {
            throw new Error('Cannot represent nullptr');
        }
        this.data = data;
        this.location = location;
        this.type = type;
        this.typeMap = typeMap;
        this.wasm = wasm;
        this.memoryOrDataView = data ? new DataView(new Uint8Array(data).buffer) : memoryView;
        if (data && data.length !== type.size) {
            throw new Error('Invalid data size');
        }
        this.memoryView = memoryView;
        this.objectStore = objectStore;
        this.objectId = objectStore.store(this);
        this.displayValue = displayValue;
        this.memoryAddress = memoryAddress;
    }
    static create(objectStore, wasm, memoryView, typeInfo) {
        const typeMap = new Map();
        for (const info of typeInfo.typeInfos) {
            typeMap.set(info.typeId, info);
        }
        const { location, root, data, displayValue, memoryAddress } = typeInfo;
        return new CXXValue(objectStore, wasm, memoryView, location ?? 0, root, typeMap, data, displayValue, memoryAddress);
    }
    get members() {
        if (!this.membersMap) {
            this.membersMap = new Map();
            for (const member of this.type.members) {
                const memberType = this.typeMap.get(member.typeId);
                if (memberType && member.name) {
                    const memberLocation = member.name === '*' ? this.memoryOrDataView.getUint32(this.location, true) :
                        this.location + member.offset;
                    this.membersMap.set(member.name, { location: memberLocation, type: memberType });
                }
            }
        }
        return this.membersMap;
    }
    getArrayElement(index) {
        const data = this.members.has('*') ? undefined : this.data;
        const element = this.members.get('*') || this.members.get('0');
        if (!element) {
            throw new Error(`Incomplete type information for array or pointer type '${this.typeNames}'`);
        }
        // FIXME handle alignment
        return new CXXValue(this.objectStore, this.wasm, this.memoryView, element.location + index * element.type.size, element.type, this.typeMap, data);
    }
    async getProperties() {
        const properties = [];
        // FIXME implement bucketing
        if (this.type.arraySize > 0) {
            for (let index = 0; index < this.type.arraySize; ++index) {
                properties.push({ name: `${index}`, property: await this.getArrayElement(index) });
            }
        }
        else {
            const members = await this.members;
            const data = members.has('*') ? undefined : this.data;
            for (const [name, { location, type }] of members) {
                const property = new CXXValue(this.objectStore, this.wasm, this.memoryView, location, type, this.typeMap, data);
                properties.push({ name, property });
            }
        }
        return properties;
    }
    async asRemoteObject() {
        if (this.type.hasValue && this.type.arraySize === 0) {
            const formatter = CustomFormatters.get(this.type);
            if (!formatter) {
                const type = 'undefined';
                const description = '<not displayable>';
                return { type, description, hasChildren: false };
            }
            if (this.location === undefined || (!this.data && this.location === 0xffffffff)) {
                const type = 'undefined';
                const description = '<optimized out>';
                return { type, description, hasChildren: false };
            }
            const value = new CXXValue(this.objectStore, this.wasm, this.memoryView, this.location, this.type, this.typeMap, this.data);
            try {
                const formattedValue = await formatter.format(this.wasm, value);
                return lazyObjectFromAny(formattedValue, this.objectStore, this.type, this.displayValue, this.memoryAddress)
                    .asRemoteObject();
            }
            catch (e) {
                // Fallthrough
            }
        }
        const type = (this.type.arraySize > 0 ? 'array' : 'object');
        const { objectId } = this;
        return {
            type,
            description: this.type.typeNames[0],
            hasChildren: this.type.members.length > 0,
            linearMemoryAddress: this.memoryAddress,
            linearMemorySize: this.type.size,
            objectId,
        };
    }
    get typeNames() {
        return this.type.typeNames;
    }
    get size() {
        return this.type.size;
    }
    asInt8() {
        return this.memoryOrDataView.getInt8(this.location);
    }
    asInt16() {
        return this.memoryOrDataView.getInt16(this.location, true);
    }
    asInt32() {
        return this.memoryOrDataView.getInt32(this.location, true);
    }
    asInt64() {
        return this.memoryOrDataView.getBigInt64(this.location, true);
    }
    asUint8() {
        return this.memoryOrDataView.getUint8(this.location);
    }
    asUint16() {
        return this.memoryOrDataView.getUint16(this.location, true);
    }
    asUint32() {
        return this.memoryOrDataView.getUint32(this.location, true);
    }
    asUint64() {
        return this.memoryOrDataView.getBigUint64(this.location, true);
    }
    asFloat32() {
        return this.memoryOrDataView.getFloat32(this.location, true);
    }
    asFloat64() {
        return this.memoryOrDataView.getFloat64(this.location, true);
    }
    asDataView(offset, size) {
        offset = this.location + (offset ?? 0);
        size = size ?? this.size;
        if (this.memoryOrDataView instanceof DataView) {
            size = Math.min(size - offset, this.memoryOrDataView.byteLength - offset - this.location);
            if (size < 0) {
                throw new RangeError('Size exceeds the buffer range');
            }
            return new DataView(this.memoryOrDataView.buffer, this.memoryOrDataView.byteOffset + this.location + offset, size);
        }
        return this.memoryView.asDataView(offset, size);
    }
    $(selector) {
        const data = this.members.has('*') ? undefined : this.data;
        if (typeof selector === 'number') {
            return this.getArrayElement(selector);
        }
        const dot = selector.indexOf('.');
        const memberName = dot >= 0 ? selector.substring(0, dot) : selector;
        selector = selector.substring(memberName.length + 1);
        const member = this.members.get(memberName);
        if (!member) {
            throw new Error(`Type ${this.typeNames[0] || '<anonymous>'} has no member '${memberName}'. Available members are: ${Array.from(this.members.keys())}`);
        }
        const memberValue = new CXXValue(this.objectStore, this.wasm, this.memoryView, member.location, member.type, this.typeMap, data);
        if (selector.length === 0) {
            return memberValue;
        }
        return memberValue.$(selector);
    }
    getMembers() {
        return Array.from(this.members.keys());
    }
}
function primitiveObject(value, description, linearMemoryAddress, type) {
    if (['number', 'string', 'boolean', 'bigint', 'undefined'].includes(typeof value)) {
        if (typeof value === 'bigint' || typeof value === 'number') {
            const enumerator = type?.enumerators?.find(e => e.value === BigInt(value));
            if (enumerator) {
                description = enumerator.name;
            }
        }
        return new PrimitiveLazyObject(typeof value, value, description, linearMemoryAddress, type?.size);
    }
    return null;
}
function lazyObjectFromAny(value, objectStore, type, description, linearMemoryAddress) {
    const primitive = primitiveObject(value, description, linearMemoryAddress, type);
    if (primitive) {
        return primitive;
    }
    if (value instanceof CXXValue) {
        return value;
    }
    if (typeof value === 'object') {
        if (value === null) {
            return new PrimitiveLazyObject('null', value, description, linearMemoryAddress);
        }
        return new LocalLazyObject(value, objectStore, type, linearMemoryAddress);
    }
    if (typeof value === 'function') {
        return value();
    }
    throw new Error('Value type is not formattable');
}
class LazyObjectStore {
    nextObjectId = 0;
    objects = new Map();
    store(lazyObject) {
        const objectId = `${this.nextObjectId++}`;
        this.objects.set(objectId, lazyObject);
        return objectId;
    }
    get(objectId) {
        return this.objects.get(objectId);
    }
    release(objectId) {
        this.objects.delete(objectId);
    }
    clear() {
        this.objects.clear();
    }
}
class PrimitiveLazyObject {
    type;
    value;
    description;
    linearMemoryAddress;
    linearMemorySize;
    constructor(type, value, description, linearMemoryAddress, linearMemorySize) {
        this.type = type;
        this.value = value;
        this.description = description ?? `${value}`;
        this.linearMemoryAddress = linearMemoryAddress;
        this.linearMemorySize = linearMemorySize;
    }
    async getProperties() {
        return [];
    }
    async asRemoteObject() {
        const { type, value, description, linearMemoryAddress, linearMemorySize } = this;
        return { type, hasChildren: false, value, description, linearMemoryAddress, linearMemorySize };
    }
}
class LocalLazyObject {
    value;
    objectId;
    objectStore;
    type;
    linearMemoryAddress;
    constructor(value, objectStore, type, linearMemoryAddress) {
        this.value = value;
        this.objectStore = objectStore;
        this.objectId = objectStore.store(this);
        this.type = type;
        this.linearMemoryAddress = linearMemoryAddress;
    }
    async getProperties() {
        return Object
            .keys(this.value)
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            .map(name => ({ name, property: lazyObjectFromAny(this.value[name], this.objectStore) }));
    }
    async asRemoteObject() {
        const type = (Array.isArray(this.value) ? 'array' : 'object');
        const { objectId, type: valueType, linearMemoryAddress } = this;
        return {
            type,
            objectId,
            description: valueType?.typeNames[0],
            hasChildren: Object.keys(this.value).length > 0,
            linearMemorySize: valueType?.size,
            linearMemoryAddress,
        };
    }
}
class HostWasmInterface {
    hostInterface;
    stopId;
    view;
    constructor(hostInterface, stopId) {
        this.hostInterface = hostInterface;
        this.stopId = stopId;
        this.view = new WasmMemoryView(this);
    }
    readMemory(offset, length) {
        return new Uint8Array(this.hostInterface.getWasmLinearMemory(offset, length, this.stopId));
    }
    getOp(op) {
        return this.hostInterface.getWasmOp(op, this.stopId);
    }
    getLocal(local) {
        return this.hostInterface.getWasmLocal(local, this.stopId);
    }
    getGlobal(global) {
        return this.hostInterface.getWasmGlobal(global, this.stopId);
    }
}
class DebuggerProxy {
    wasm;
    target;
    constructor(wasm, target) {
        this.wasm = wasm;
        this.target = target;
    }
    readMemory(src, dst, length) {
        const data = this.wasm.view.asDataView(src, length);
        this.target.HEAP8.set(new Uint8Array(data.buffer, data.byteOffset, length), dst);
        return data.byteLength;
    }
    getLocal(index) {
        return this.wasm.getLocal(index);
    }
    getGlobal(index) {
        return this.wasm.getGlobal(index);
    }
    getOperand(index) {
        return this.wasm.getOp(index);
    }
}
class CustomFormatters {
    static formatters = new Map();
    static genericFormatters = [];
    static addFormatter(formatter) {
        if (Array.isArray(formatter.types)) {
            for (const type of formatter.types) {
                CustomFormatters.formatters.set(type, formatter);
            }
        }
        else {
            CustomFormatters.genericFormatters.push(formatter);
        }
    }
    static get(type) {
        for (const name of type.typeNames) {
            const formatter = CustomFormatters.formatters.get(name);
            if (formatter) {
                return formatter;
            }
        }
        for (const t of type.typeNames) {
            const CONST_PREFIX = 'const ';
            if (t.startsWith(CONST_PREFIX)) {
                const formatter = CustomFormatters.formatters.get(t.substr(CONST_PREFIX.length));
                if (formatter) {
                    return formatter;
                }
            }
        }
        for (const formatter of CustomFormatters.genericFormatters) {
            if (formatter.types instanceof Function) {
                if (formatter.types(type)) {
                    return formatter;
                }
            }
        }
        return null;
    }
}

// Copyright 2020 Google LLC. All rights reserved.
/*
 * Numbers
 */
CustomFormatters.addFormatter({ types: ['bool'], format: (wasm, value) => value.asUint8() > 0 });
CustomFormatters.addFormatter({ types: ['uint16_t'], format: (wasm, value) => value.asUint16() });
CustomFormatters.addFormatter({ types: ['uint32_t'], format: (wasm, value) => value.asUint32() });
CustomFormatters.addFormatter({ types: ['uint64_t'], format: (wasm, value) => value.asUint64() });
CustomFormatters.addFormatter({ types: ['int16_t'], format: (wasm, value) => value.asInt16() });
CustomFormatters.addFormatter({ types: ['int32_t'], format: (wasm, value) => value.asInt32() });
CustomFormatters.addFormatter({ types: ['int64_t'], format: (wasm, value) => value.asInt64() });
CustomFormatters.addFormatter({ types: ['float'], format: (wasm, value) => value.asFloat32() });
CustomFormatters.addFormatter({ types: ['double'], format: (wasm, value) => value.asFloat64() });
function formatVoid() {
    return () => new PrimitiveLazyObject('undefined', undefined, '<void>');
}
CustomFormatters.addFormatter({ types: ['void'], format: formatVoid });
CustomFormatters.addFormatter({ types: ['uint8_t', 'int8_t'], format: formatChar });
function formatChar(wasm, value) {
    const char = value.typeNames.includes('int8_t') ? Math.abs(value.asInt8()) : value.asUint8();
    switch (char) {
        case 0x0:
            return '\'\\0\'';
        case 0x7:
            return '\'\\a\'';
        case 0x8:
            return '\'\\b\'';
        case 0x9:
            return '\'\\t\'';
        case 0xA:
            return '\'\\n\'';
        case 0xB:
            return '\'\\v\'';
        case 0xC:
            return '\'\\f\'';
        case 0xD:
            return '\'\\r\'';
    }
    if (char < 0x20 || char > 0x7e) {
        return `'\\x${char.toString(16).padStart(2, '0')}'`;
    }
    return `'${String.fromCharCode(value.asInt8())}'`;
}
CustomFormatters.addFormatter({
    types: ['wchar_t', 'char32_t', 'char16_t'],
    format: (wasm, value) => {
        const codepoint = value.size === 2 ? value.asUint16() : value.asUint32();
        try {
            return String.fromCodePoint(codepoint);
        }
        catch {
            return `U+${codepoint.toString(16).padStart(value.size * 2, '0')}`;
        }
    },
});
/*
 * STL
 */
function formatLibCXXString(wasm, value, charType, decode) {
    const shortString = value.$('__r_.__value_.<union>.__s');
    const size = shortString.getMembers().includes('<union>') ? shortString.$('<union>.__size_').asUint8() :
        shortString.$('__size_').asUint8();
    const isLong = 0 < (size & 0x80);
    const charSize = charType.BYTES_PER_ELEMENT;
    if (isLong) {
        const longString = value.$('__r_.__value_.<union>.__l');
        const data = longString.$('__data_').asUint32();
        const stringSize = longString.$('__size_').asUint32();
        const copyLen = Math.min(stringSize * charSize, 268435440 /* Constants.MAX_STRING_LEN */);
        const bytes = wasm.readMemory(data, copyLen);
        const text = new charType(bytes.buffer, bytes.byteOffset, stringSize);
        return { size: stringSize, string: decode(text) };
    }
    const bytes = shortString.$('__data_').asDataView(0, size * charSize);
    const text = new charType(bytes.buffer, bytes.byteOffset, size);
    return { size, string: decode(text) };
}
function formatLibCXX8String(wasm, value) {
    return formatLibCXXString(wasm, value, Uint8Array, str => new TextDecoder().decode(str));
}
function formatLibCXX16String(wasm, value) {
    return formatLibCXXString(wasm, value, Uint16Array, str => new TextDecoder('utf-16le').decode(str));
}
function formatLibCXX32String(wasm, value) {
    // emscripten's wchar is 4 byte
    return formatLibCXXString(wasm, value, Uint32Array, str => Array.from(str).map(v => String.fromCodePoint(v)).join(''));
}
CustomFormatters.addFormatter({
    types: [
        'std::__2::string',
        'std::__2::basic_string<char, std::__2::char_traits<char>, std::__2::allocator<char> >',
        'std::__2::u8string',
        'std::__2::basic_string<char8_t, std::__2::char_traits<char8_t>, std::__2::allocator<char8_t> >',
    ],
    format: formatLibCXX8String,
});
CustomFormatters.addFormatter({
    types: [
        'std::__2::u16string',
        'std::__2::basic_string<char16_t, std::__2::char_traits<char16_t>, std::__2::allocator<char16_t> >',
    ],
    format: formatLibCXX16String,
});
CustomFormatters.addFormatter({
    types: [
        'std::__2::wstring',
        'std::__2::basic_string<wchar_t, std::__2::char_traits<wchar_t>, std::__2::allocator<wchar_t> >',
        'std::__2::u32string',
        'std::__2::basic_string<char32_t, std::__2::char_traits<char32_t>, std::__2::allocator<char32_t> >',
    ],
    format: formatLibCXX32String,
});
function formatRawString(wasm, value, charType, decode) {
    const address = value.asUint32();
    if (address < 1024 /* Constants.SAFE_HEAP_START */) {
        return formatPointerOrReference(wasm, value);
    }
    const charSize = charType.BYTES_PER_ELEMENT;
    const slices = [];
    const deref = value.$('*');
    for (let bufferSize = 0; bufferSize < 268435440 /* Constants.MAX_STRING_LEN */; bufferSize += 4096 /* Constants.PAGE_SIZE */) {
        // Copy PAGE_SIZE bytes
        const buffer = deref.asDataView(bufferSize, 4096 /* Constants.PAGE_SIZE */);
        // Convert to charType
        const substr = new charType(buffer.buffer, buffer.byteOffset, buffer.byteLength / charSize);
        const strlen = substr.indexOf(0);
        if (strlen >= 0) {
            // buffer size is in bytes, strlen in characters
            const str = new charType(bufferSize / charSize + strlen);
            for (let i = 0; i < slices.length; ++i) {
                str.set(new charType(slices[i].buffer, slices[i].byteOffset, slices[i].byteLength / charSize), i * 4096 /* Constants.PAGE_SIZE */ / charSize);
            }
            str.set(substr.subarray(0, strlen), bufferSize / charSize);
            return decode(str);
        }
        slices.push(buffer);
    }
    return formatPointerOrReference(wasm, value);
}
function formatCString(wasm, value) {
    return formatRawString(wasm, value, Uint8Array, str => new TextDecoder().decode(str));
}
function formatU16CString(wasm, value) {
    return formatRawString(wasm, value, Uint16Array, str => new TextDecoder('utf-16le').decode(str));
}
function formatCWString(wasm, value) {
    // emscripten's wchar is 4 byte
    return formatRawString(wasm, value, Uint32Array, str => Array.from(str).map(v => String.fromCodePoint(v)).join(''));
}
// Register with higher precedence than the generic pointer handler.
CustomFormatters.addFormatter({ types: ['char *', 'char8_t *'], format: formatCString });
CustomFormatters.addFormatter({ types: ['char16_t *'], format: formatU16CString });
CustomFormatters.addFormatter({ types: ['wchar_t *', 'char32_t *'], format: formatCWString });
function formatVector(wasm, value) {
    const begin = value.$('__begin_');
    const end = value.$('__end_');
    const size = (end.asUint32() - begin.asUint32()) / begin.$('*').size;
    const elements = [];
    for (let i = 0; i < size; ++i) {
        elements.push(begin.$(i));
    }
    return elements;
}
function reMatch(...exprs) {
    return (type) => {
        for (const expr of exprs) {
            for (const name of type.typeNames) {
                if (expr.exec(name)) {
                    return true;
                }
            }
        }
        for (const expr of exprs) {
            for (const name of type.typeNames) {
                if (name.startsWith('const ')) {
                    if (expr.exec(name.substring(6))) {
                        return true;
                    }
                }
            }
        }
        return false;
    };
}
CustomFormatters.addFormatter({ types: reMatch(/^std::vector<.+>$/), format: formatVector });
function formatPointerOrReference(wasm, value) {
    const address = value.asUint32();
    if (address === 0) {
        return { '0x0': null };
    }
    return { [`0x${address.toString(16)}`]: value.$('*') };
}
CustomFormatters.addFormatter({ types: type => type.isPointer, format: formatPointerOrReference });
function formatDynamicArray(wasm, value) {
    return { [`0x${value.location.toString(16)}`]: value.$(0) };
}
CustomFormatters.addFormatter({ types: reMatch(/^.+\[\]$/), format: formatDynamicArray });
function formatUInt128(wasm, value) {
    const view = value.asDataView();
    return (view.getBigUint64(8, true) << BigInt(64)) + (view.getBigUint64(0, true));
}
CustomFormatters.addFormatter({ types: ['unsigned __int128'], format: formatUInt128 });
function formatInt128(wasm, value) {
    const view = value.asDataView();
    return (view.getBigInt64(8, true) << BigInt(64)) | (view.getBigUint64(0, true));
}
CustomFormatters.addFormatter({ types: ['__int128'], format: formatInt128 });

// Copyright 2020 Google LLC. All rights reserved.
function globToRegExp(glob) {
    let re = '^';
    for (let i = 0; i < glob.length; ++i) {
        const c = glob.charCodeAt(i);
        if (c === 0x2a) {
            if (i + 2 < glob.length && glob.charCodeAt(i + 1) === 0x2a && glob.charCodeAt(i + 2) === 0x2f) {
                // Compile '**/' to match everything including slashes.
                re += '.*';
                i += 2;
            }
            else {
                // Compile '*' to match everything except slashes.
                re += '[^/]*';
            }
        }
        else {
            // Just escape everything else, so we don't need to
            // worry about special characters like ., +, $, etc.
            re += `\\u${c.toString(16).padStart(4, '0')}`;
        }
    }
    re += '$';
    return new RegExp(re, 'i');
}
/**
 * Performs a glob-style pattern matching.
 *
 * The following special characters are supported for the `pattern`:
 *
 * - '*' matches every sequence of characters, except for slash ('/').
 * - '**' plus '/' matches every sequence of characters, including slash ('/').
 *
 * If the `pattern` doesn't contain a slash ('/'), only the last path
 * component of the `subject` (its basename) will be matched against
 * the `pattern`. Otherwise if at least one slash is found in `pattern`
 * the full `subject` is matched against the `pattern`.
 *
 * @param pattern the wildcard pattern
 * @param subject the subject URL to test against
 * @return whether the `subject` matches the given `pattern`.
 */
function globMatch(pattern, subject) {
    const regexp = globToRegExp(pattern);
    if (!pattern.includes('/')) {
        subject = subject.slice(subject.lastIndexOf('/') + 1);
    }
    return regexp.test(subject);
}

// Copyright 2020 Google LLC. All rights reserved.
/**
 * Resolve a source path (as stored in DWARF debugging information) to an absolute URL.
 *
 * Note that we treat "." specially as a pattern, since LLDB normalizes paths before
 * returning them from the DWARF parser. Our logic replicates the logic found in the
 * LLDB frontend in `PathMappingList::RemapPath()` inside `Target/PathMappingList.cpp`
 * (http://cs/github/llvm/llvm-project/lldb/source/Target/PathMappingList.cpp?l=157-185).
 *
 * @param pathSubstitutions possible substitutions to apply to the {@param sourcePath}, applies the first match.
 * @param sourcePath the source path as found in the debugging information.
 * @param baseURL the URL of the WebAssembly module, which is used to resolve relative source paths.
 * @return an absolute `file:`-URI or a URL relative to the {@param baseURL}.
 */
function resolveSourcePathToURL(pathSubstitutions, sourcePath, baseURL) {
    // Normalize '\' to '/' in sourcePath first.
    let resolvedSourcePath = sourcePath.replace(/\\/g, '/');
    // Apply source path substitutions first.
    for (const { from, to } of pathSubstitutions) {
        if (resolvedSourcePath.startsWith(from)) {
            resolvedSourcePath = to + resolvedSourcePath.slice(from.length);
            break;
        }
        // Relative paths won't have a leading "./" in them unless "." is the only
        // thing in the relative path so we need to work around "." carefully.
        if (from === '.') {
            // We need to figure whether sourcePath can be considered a relative path,
            // ruling out absolute POSIX and Windows paths, as well as file:, http: and
            // https: URLs.
            if (!resolvedSourcePath.startsWith('/') && !/^([A-Z]|file|https?):/i.test(resolvedSourcePath)) {
                resolvedSourcePath = `${to}/${resolvedSourcePath}`;
                break;
            }
        }
    }
    if (resolvedSourcePath.startsWith('/')) {
        if (resolvedSourcePath.startsWith('//')) {
            return new URL(`file:${resolvedSourcePath}`);
        }
        return new URL(`file://${resolvedSourcePath}`);
    }
    if (/^[A-Z]:/i.test(resolvedSourcePath)) {
        return new URL(`file:/${resolvedSourcePath}`);
    }
    return new URL(resolvedSourcePath, baseURL.href);
}
/**
 * Locate the configuration for a given `something.wasm` module file name.
 *
 * @param moduleConfigurations list of module configurations to scan.
 * @param moduleName the URL of the module to lookup.
 * @return the matching module configuration or the default fallback.
 */
function findModuleConfiguration(moduleConfigurations, moduleURL) {
    let defaultModuleConfiguration = { pathSubstitutions: [] };
    for (const moduleConfiguration of moduleConfigurations) {
        // The idea here is that module configurations will have at most
        // one default configuration, so picking the last here is fine.
        if (moduleConfiguration.name === undefined) {
            defaultModuleConfiguration = moduleConfiguration;
            continue;
        }
        // Perform wildcard pattern matching on the full URL.
        if (globMatch(moduleConfiguration.name, moduleURL.href)) {
            return moduleConfiguration;
        }
    }
    return defaultModuleConfiguration;
}
const DEFAULT_MODULE_CONFIGURATIONS = [{ pathSubstitutions: [] }];

// Copyright 2020 Google LLC. All rights reserved.
function mapVector(vector, callback) {
    const elements = [];
    for (let i = 0; i < vector.size(); ++i) {
        const element = vector.get(i);
        elements.push(callback(element));
    }
    return elements;
}
function mapEnumerator(apiEnumerator) {
    return { typeId: apiEnumerator.typeId, value: apiEnumerator.value, name: apiEnumerator.name };
}
function mapFieldInfo(apiFieldInfo) {
    return { typeId: apiFieldInfo.typeId, offset: apiFieldInfo.offset, name: apiFieldInfo.name };
}
class ModuleInfo {
    symbolsUrl;
    symbolsFileName;
    symbolsDwpFileName;
    backend;
    fileNameToUrl;
    urlToFileName;
    dwarfSymbolsPlugin;
    constructor(symbolsUrl, symbolsFileName, symbolsDwpFileName, backend) {
        this.symbolsUrl = symbolsUrl;
        this.symbolsFileName = symbolsFileName;
        this.symbolsDwpFileName = symbolsDwpFileName;
        this.backend = backend;
        this.fileNameToUrl = new Map();
        this.urlToFileName = new Map();
        this.dwarfSymbolsPlugin = new backend.DWARFSymbolsPlugin();
    }
    stringifyScope(scope) {
        switch (scope) {
            case this.backend.VariableScope.GLOBAL:
                return 'GLOBAL';
            case this.backend.VariableScope.LOCAL:
                return 'LOCAL';
            case this.backend.VariableScope.PARAMETER:
                return 'PARAMETER';
        }
        throw new Error(`InternalError: Invalid scope ${scope}`);
    }
    stringifyErrorCode(errorCode) {
        switch (errorCode) {
            case this.backend.ErrorCode.PROTOCOL_ERROR:
                return 'ProtocolError:';
            case this.backend.ErrorCode.MODULE_NOT_FOUND_ERROR:
                return 'ModuleNotFoundError:';
            case this.backend.ErrorCode.INTERNAL_ERROR:
                return 'InternalError';
            case this.backend.ErrorCode.EVAL_ERROR:
                return 'EvalError';
        }
        throw new Error(`InternalError: Invalid error code ${errorCode}`);
    }
}
function createEmbindPool() {
    class EmbindObjectPool {
        objectPool = [];
        flush() {
            for (const object of this.objectPool.reverse()) {
                object.delete();
            }
            this.objectPool = [];
        }
        manage(object) {
            if (typeof object !== 'undefined') {
                this.objectPool.push(object);
            }
            return object;
        }
        unmanage(object) {
            const index = this.objectPool.indexOf(object);
            if (index > -1) {
                this.objectPool.splice(index, 1);
                object.delete();
                return true;
            }
            return false;
        }
    }
    const pool = new EmbindObjectPool();
    const manage = pool.manage.bind(pool);
    const unmanage = pool.unmanage.bind(pool);
    const flush = pool.flush.bind(pool);
    return { manage, unmanage, flush };
}
// Cache the underlying WebAssembly module after the first instantiation
// so that subsequent calls to `createSymbolsBackend()` are faster, which
// greatly speeds up the test suite.
let symbolsBackendModulePromise;
function instantiateWasm(imports, callback, resourceLoader) {
    if (!symbolsBackendModulePromise) {
        symbolsBackendModulePromise = resourceLoader.createSymbolsBackendModulePromise();
    }
    symbolsBackendModulePromise.then(module => WebAssembly.instantiate(module, imports))
        .then(callback)
        .catch(console.error);
    return [];
}
class DWARFLanguageExtensionPlugin {
    moduleConfigurations;
    resourceLoader;
    hostInterface;
    moduleInfos = new Map();
    lazyObjects = new LazyObjectStore();
    constructor(moduleConfigurations, resourceLoader, hostInterface) {
        this.moduleConfigurations = moduleConfigurations;
        this.resourceLoader = resourceLoader;
        this.hostInterface = hostInterface;
        this.moduleConfigurations = moduleConfigurations;
    }
    getTypeInfo(_expression, _context) {
        throw new Error('Method not implemented.');
    }
    getFormatter(_expressionOrField, _context) {
        throw new Error('Method not implemented.');
    }
    getInspectableAddress(_field) {
        throw new Error('Method not implemented.');
    }
    async newModuleInfo(rawModuleId, symbolsHint, rawModule) {
        const { flush, manage } = createEmbindPool();
        try {
            const rawModuleURL = new URL(rawModule.url);
            const { pathSubstitutions } = findModuleConfiguration(this.moduleConfigurations, rawModuleURL);
            const symbolsURL = symbolsHint ? resolveSourcePathToURL([], symbolsHint, rawModuleURL) : rawModuleURL;
            const instantiateWasmWrapper = (imports, callback) => {
                // Emscripten type definitions are incorrect, we're getting passed a WebAssembly.Imports object here.
                return instantiateWasm(imports, callback, this.resourceLoader);
            };
            const backend = await createSymbolsBackend({ instantiateWasm: instantiateWasmWrapper });
            const { symbolsFileName, symbolsDwpFileName } = await this.resourceLoader.loadSymbols(rawModuleId, rawModule, symbolsURL, backend.FS);
            const moduleInfo = new ModuleInfo(symbolsURL.href, symbolsFileName, symbolsDwpFileName, backend);
            const addRawModuleResponse = manage(moduleInfo.dwarfSymbolsPlugin.AddRawModule(rawModuleId, symbolsFileName));
            mapVector(manage(addRawModuleResponse.sources), fileName => {
                const fileURL = resolveSourcePathToURL(pathSubstitutions, fileName, symbolsURL);
                moduleInfo.fileNameToUrl.set(fileName, fileURL.href);
                moduleInfo.urlToFileName.set(fileURL.href, fileName);
            });
            // Set up lazy dwo files if we are running on a worker
            if (typeof global === 'undefined' && typeof importScripts === 'function' &&
                typeof XMLHttpRequest !== 'undefined') {
                mapVector(manage(addRawModuleResponse.dwos), dwoFile => {
                    const absolutePath = dwoFile.startsWith('/') ? dwoFile : '/' + dwoFile;
                    const pathSplit = absolutePath.split('/');
                    const fileName = pathSplit.pop();
                    const parentDirectory = pathSplit.join('/');
                    // Sometimes these stick around.
                    try {
                        backend.FS.unlink(absolutePath);
                    }
                    catch (_) {
                    }
                    // Ensure directory exists
                    if (parentDirectory.length > 1) {
                        // TypeScript doesn't know about createPath
                        // @ts-ignore
                        backend.FS.createPath('/', parentDirectory.substring(1), true, true);
                    }
                    const node = backend.FS.createLazyFile(parentDirectory, fileName, new URL(dwoFile, symbolsURL).href, true, false);
                    const oldget = node.node_ops.getattr;
                    const wrapper = (n) => {
                        try {
                            return oldget(n);
                        }
                        catch (_) {
                            // Rethrow any error fetching the content as errno 44 (EEXIST)
                            // TypeScript doesn't know about the ErrnoError constructor
                            // @ts-ignore
                            throw new backend.FS.ErrnoError(44);
                        }
                    };
                    if (oldget.toString() !== wrapper.toString()) {
                        node.node_ops.getattr = wrapper;
                    }
                });
            }
            return moduleInfo;
        }
        finally {
            flush();
        }
    }
    async addRawModule(rawModuleId, symbolsUrl, rawModule) {
        // This complex logic makes sure that addRawModule / removeRawModule calls are
        // handled sequentially for the same rawModuleId, and thus this looks symmetrical
        // to the removeRawModule() method below. The idea is that we chain our operation
        // on any previous operation for the same rawModuleId, and thereby end up with a
        // single sequence of events.
        const originalPromise = Promise.resolve(this.moduleInfos.get(rawModuleId));
        const moduleInfoPromise = originalPromise.then(moduleInfo => {
            if (moduleInfo) {
                throw new Error(`InternalError: Duplicate module with ID '${rawModuleId}'`);
            }
            return this.newModuleInfo(rawModuleId, symbolsUrl, rawModule);
        });
        // This looks a bit odd, but it's important that the operation is chained via
        // the `_moduleInfos` map *and* at the same time resolves to it's original
        // value in case of an error (i.e. if someone tried to add the same rawModuleId
        // twice, this will retain the original value in that case instead of having all
        // users get the internal error).
        this.moduleInfos.set(rawModuleId, moduleInfoPromise.catch(() => originalPromise));
        const moduleInfo = await moduleInfoPromise;
        return [...moduleInfo.urlToFileName.keys()];
    }
    async getModuleInfo(rawModuleId) {
        const moduleInfo = await this.moduleInfos.get(rawModuleId);
        if (!moduleInfo) {
            throw new Error(`InternalError: Unknown module with raw module ID ${rawModuleId}`);
        }
        return moduleInfo;
    }
    async removeRawModule(rawModuleId) {
        const originalPromise = Promise.resolve(this.moduleInfos.get(rawModuleId));
        const moduleInfoPromise = originalPromise.then(moduleInfo => {
            if (!moduleInfo) {
                throw new Error(`InternalError: No module with ID '${rawModuleId}'`);
            }
            return undefined;
        });
        this.moduleInfos.set(rawModuleId, moduleInfoPromise.catch(() => originalPromise));
        await moduleInfoPromise;
    }
    async sourceLocationToRawLocation(sourceLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(sourceLocation.rawModuleId);
        const sourceFile = moduleInfo.urlToFileName.get(sourceLocation.sourceFileURL);
        if (!sourceFile) {
            throw new Error(`InternalError: Unknown URL ${sourceLocation.sourceFileURL}`);
        }
        try {
            const rawLocations = manage(moduleInfo.dwarfSymbolsPlugin.SourceLocationToRawLocation(sourceLocation.rawModuleId, sourceFile, sourceLocation.lineNumber, sourceLocation.columnNumber));
            const error = manage(rawLocations.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const locations = mapVector(manage(rawLocations.rawLocationRanges), rawLocation => {
                const { rawModuleId, startOffset, endOffset } = manage(rawLocation);
                return { rawModuleId, startOffset, endOffset };
            });
            return locations;
        }
        finally {
            flush();
        }
    }
    async rawLocationToSourceLocation(rawLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawLocation.rawModuleId);
        try {
            const sourceLocations = moduleInfo.dwarfSymbolsPlugin.RawLocationToSourceLocation(rawLocation.rawModuleId, rawLocation.codeOffset, rawLocation.inlineFrameIndex || 0);
            const error = manage(sourceLocations.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const locations = mapVector(manage(sourceLocations.sourceLocation), sourceLocation => {
                const sourceFileURL = moduleInfo.fileNameToUrl.get(sourceLocation.sourceFile);
                if (!sourceFileURL) {
                    throw new Error(`InternalError: Unknown source file ${sourceLocation.sourceFile}`);
                }
                const { rawModuleId, lineNumber, columnNumber } = manage(sourceLocation);
                return {
                    rawModuleId,
                    sourceFileURL,
                    lineNumber,
                    columnNumber,
                };
            });
            return locations;
        }
        finally {
            flush();
        }
    }
    async getScopeInfo(type) {
        switch (type) {
            case 'GLOBAL':
                return {
                    type,
                    typeName: 'Global',
                    icon: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTIiIGhlaWdodD0iMTIiIHZpZXdCb3g9IjAgMCAxMiAxMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3QgeD0iMC41IiB5PSIwLjUiIHdpZHRoPSIxMSIgaGVpZ2h0PSIxMSIgcng9IjEuNSIgZmlsbD0iIzFBNzNFOCIgc3Ryb2tlPSIjMTQ1NkFEIi8+CjxwYXRoIGQ9Ik04Ljg1MTU2IDguNjAxNTZDOC41ODc4OSA4LjkxNzMyIDguMjE1MTcgOS4xNjMwOSA3LjczMzQgOS4zMzg4N0M3LjI1MTYzIDkuNTExMzkgNi43MTc3NyA5LjU5NzY2IDYuMTMxODQgOS41OTc2NkM1LjUxNjYgOS41OTc2NiA0Ljk3NjI0IDkuNDY0MTkgNC41MTA3NCA5LjE5NzI3QzQuMDQ4NSA4LjkyNzA4IDMuNjkwNDMgOC41MzY0NiAzLjQzNjUyIDguMDI1MzlDMy4xODU4NyA3LjUxNDMyIDMuMDU3MjkgNi45MTM3NCAzLjA1MDc4IDYuMjIzNjNWNS43NDAyM0MzLjA1MDc4IDUuMDMwNiAzLjE2OTYgNC40MTY5OSAzLjQwNzIzIDMuODk5NDFDMy42NDgxMSAzLjM3ODU4IDMuOTkzMTYgMi45ODE0NSA0LjQ0MjM4IDIuNzA4MDFDNC44OTQ4NiAyLjQzMTMyIDUuNDIzODMgMi4yOTI5NyA2LjAyOTMgMi4yOTI5N0M2Ljg3MjQgMi4yOTI5NyA3LjUzMTU4IDIuNDk0NzkgOC4wMDY4NCAyLjg5ODQ0QzguNDgyMSAzLjI5ODgzIDguNzYzNjcgMy44ODMxNCA4Ljg1MTU2IDQuNjUxMzdINy40MjU3OEM3LjM2MDY4IDQuMjQ0NDcgNy4yMTU4MiAzLjk0NjYxIDYuOTkxMjEgMy43NTc4MUM2Ljc2OTg2IDMuNTY5MDEgNi40NjM4NyAzLjQ3NDYxIDYuMDczMjQgMy40NzQ2MUM1LjU3NTIgMy40NzQ2MSA1LjE5NTk2IDMuNjYxNzggNC45MzU1NSA0LjAzNjEzQzQuNjc1MTMgNC40MTA0OCA0LjU0MzI5IDQuOTY3MTIgNC41NDAwNCA1LjcwNjA1VjYuMTYwMTZDNC41NDAwNCA2LjkwNTYgNC42ODE2NCA3LjQ2ODc1IDQuOTY0ODQgNy44NDk2MUM1LjI0ODA1IDguMjMwNDcgNS42NjMwOSA4LjQyMDkgNi4yMDk5NiA4LjQyMDlDNi43NjAwOSA4LjQyMDkgNy4xNTIzNCA4LjMwMzcxIDcuMzg2NzIgOC4wNjkzNFY2Ljg0Mzc1SDYuMDUzNzFWNS43NjQ2NUg4Ljg1MTU2VjguNjAxNTZaIiBmaWxsPSJ3aGl0ZSIvPgo8L3N2Zz4K',
                };
            case 'LOCAL':
                return {
                    type,
                    typeName: 'Local',
                    icon: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTIiIGhlaWdodD0iMTIiIHZpZXdCb3g9IjAgMCAxMiAxMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3QgeD0iMC41IiB5PSIwLjUiIHdpZHRoPSIxMSIgaGVpZ2h0PSIxMSIgcng9IjEuNSIgZmlsbD0iIzFBNzNFOCIgc3Ryb2tlPSIjMTQ1NkFEIi8+CjxwYXRoIGQ9Ik01LjM4OTY1IDguMzIzMjRIOC41VjkuNUgzLjkyNDhWMi4zOTA2Mkg1LjM4OTY1VjguMzIzMjRaIiBmaWxsPSJ3aGl0ZSIvPgo8L3N2Zz4K',
                };
            case 'PARAMETER':
                return {
                    type,
                    typeName: 'Parameter',
                    icon: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTIiIGhlaWdodD0iMTIiIHZpZXdCb3g9IjAgMCAxMiAxMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3QgeD0iMC41IiB5PSIwLjUiIHdpZHRoPSIxMSIgaGVpZ2h0PSIxMSIgcng9IjEuNSIgZmlsbD0iIzFBNzNFOCIgc3Ryb2tlPSIjMTQ1NkFEIi8+CjxwYXRoIGQ9Ik03LjMyODEyIDQuNjAxNzFINi41NDI5N0w1Ljk5NjA5IDguMTkxNTZDNS45Mzg4IDguNjc1OTMgNS43NzQ3NCA5LjA1MDkzIDUuNTAzOTEgOS4zMTY1NkM1LjIzMzA3IDkuNTg0NzkgNC44Nzc2IDkuNzE4OSA0LjQzNzUgOS43MTg5QzQuMjU3ODEgOS43MTg5IDQuMDY1MSA5LjY5Njc2IDMuODU5MzggOS42NTI0OUwzLjk4MDQ3IDguNzQ2MjRDNC4xMDI4NiA4Ljc4NTMxIDQuMjE4NzUgOC44MDQ4NCA0LjMyODEyIDguODA0ODRDNC42MzU0MiA4LjgwNDg0IDQuODIwMzEgOC42MDA0MSA0Ljg4MjgxIDguMTkxNTZMNS40Mzc1IDQuNjAxNzFINC44MjgxMkw0Ljk1NzAzIDMuNzczNTlMNS41NjY0MSAzLjc2OTY4TDUuNjA5MzggMy40MDY0QzUuNjY0MDYgMi45MzI0NCA1Ljg0MTE1IDIuNTYzOTUgNi4xNDA2MiAyLjMwMDkzQzYuNDQwMSAyLjAzNTMxIDYuODI0MjIgMS45MDUxIDcuMjkyOTcgMS45MTAzMUM3LjQzNjIgMS45MTAzMSA3LjY0ODQ0IDEuOTM3NjUgNy45Mjk2OSAxLjk5MjM0TDcuNzczNDQgMi44NzUxNUM3LjY0MzIzIDIuODQxMyA3LjUxOTUzIDIuODI0MzcgNy40MDIzNCAyLjgyNDM3QzcuMjE0ODQgMi44MjQzNyA3LjA2MjUgMi44NzY0NSA2Ljk0NTMxIDIuOTgwNjJDNi44MzA3MyAzLjA4NDc5IDYuNzU3ODEgMy4yMjY3MSA2LjcyNjU2IDMuNDA2NEw2LjY3MTg4IDMuNzczNTlINy40NTcwM0w3LjMyODEyIDQuNjAxNzFaIiBmaWxsPSJ3aGl0ZSIvPgo8L3N2Zz4K',
                };
        }
        throw new Error(`InternalError: Invalid scope type '${type}`);
    }
    async listVariablesInScope(rawLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawLocation.rawModuleId);
        try {
            const variables = manage(moduleInfo.dwarfSymbolsPlugin.ListVariablesInScope(rawLocation.rawModuleId, rawLocation.codeOffset, rawLocation.inlineFrameIndex || 0));
            const error = manage(variables.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const apiVariables = mapVector(manage(variables.variable), variable => {
                const { scope, name, type } = manage(variable);
                return { scope: moduleInfo.stringifyScope(scope), name, type, nestedName: name.split('::') };
            });
            return apiVariables;
        }
        finally {
            flush();
        }
    }
    async getFunctionInfo(rawLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawLocation.rawModuleId);
        try {
            const functionInfo = manage(moduleInfo.dwarfSymbolsPlugin.GetFunctionInfo(rawLocation.rawModuleId, rawLocation.codeOffset));
            const error = manage(functionInfo.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const apiFunctionInfos = mapVector(manage(functionInfo.functionNames), functionName => {
                return { name: functionName };
            });
            let apiMissingSymbolFiles = mapVector(manage(functionInfo.missingSymbolFiles), x => x);
            if (apiMissingSymbolFiles.length && this.resourceLoader.possiblyMissingSymbols) {
                apiMissingSymbolFiles = apiMissingSymbolFiles.concat(this.resourceLoader.possiblyMissingSymbols);
            }
            return { frames: apiFunctionInfos, missingSymbolFiles: apiMissingSymbolFiles };
        }
        finally {
            flush();
        }
    }
    async getInlinedFunctionRanges(rawLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawLocation.rawModuleId);
        try {
            const rawLocations = manage(moduleInfo.dwarfSymbolsPlugin.GetInlinedFunctionRanges(rawLocation.rawModuleId, rawLocation.codeOffset));
            const error = manage(rawLocations.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const locations = mapVector(manage(rawLocations.rawLocationRanges), rawLocation => {
                const { rawModuleId, startOffset, endOffset } = manage(rawLocation);
                return { rawModuleId, startOffset, endOffset };
            });
            return locations;
        }
        finally {
            flush();
        }
    }
    async getInlinedCalleesRanges(rawLocation) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawLocation.rawModuleId);
        try {
            const rawLocations = manage(moduleInfo.dwarfSymbolsPlugin.GetInlinedCalleesRanges(rawLocation.rawModuleId, rawLocation.codeOffset));
            const error = manage(rawLocations.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const locations = mapVector(manage(rawLocations.rawLocationRanges), rawLocation => {
                const { rawModuleId, startOffset, endOffset } = manage(rawLocation);
                return { rawModuleId, startOffset, endOffset };
            });
            return locations;
        }
        finally {
            flush();
        }
    }
    async getValueInfo(expression, context, stopId) {
        const { manage, unmanage, flush } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(context.rawModuleId);
        try {
            const apiRawLocation = manage(new moduleInfo.backend.RawLocation());
            apiRawLocation.rawModuleId = context.rawModuleId;
            apiRawLocation.codeOffset = context.codeOffset;
            apiRawLocation.inlineFrameIndex = context.inlineFrameIndex || 0;
            const wasm = new HostWasmInterface(this.hostInterface, stopId);
            const proxy = new DebuggerProxy(wasm, moduleInfo.backend);
            const typeInfoResult = manage(moduleInfo.dwarfSymbolsPlugin.EvaluateExpression(apiRawLocation, expression, proxy));
            const error = manage(typeInfoResult.error);
            if (error) {
                if (error.code === moduleInfo.backend.ErrorCode.MODULE_NOT_FOUND_ERROR) {
                    // Let's not throw when the module gets unloaded - that is quite common path that
                    // we hit when the source-scope pane still keeps asynchronously updating while we
                    // unload the wasm module.
                    return null;
                }
                // TODO(crbug.com/1271147) Instead of throwing, we whould create an AST error node with the message
                // so that it is properly surfaced to the user. This should then make the special handling of
                // MODULE_NOT_FOUND_ERROR unnecessary.
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const typeInfos = mapVector(manage(typeInfoResult.typeInfos), typeInfo => fromApiTypeInfo(manage(typeInfo)));
            const root = fromApiTypeInfo(manage(typeInfoResult.root));
            const { location, displayValue, memoryAddress } = typeInfoResult;
            const data = typeInfoResult.data ? mapVector(manage(typeInfoResult.data), n => n) : undefined;
            return { typeInfos, root, location, data, displayValue, memoryAddress };
            function fromApiTypeInfo(apiTypeInfo) {
                const apiMembers = manage(apiTypeInfo.members);
                const members = mapVector(apiMembers, fieldInfo => mapFieldInfo(manage(fieldInfo)));
                const apiEnumerators = manage(apiTypeInfo.enumerators);
                const enumerators = mapVector(apiEnumerators, enumerator => mapEnumerator(manage(enumerator)));
                unmanage(apiEnumerators);
                const typeNames = mapVector(manage(apiTypeInfo.typeNames), e => e);
                unmanage(apiMembers);
                const { typeId, size, arraySize, alignment, canExpand, isPointer, hasValue } = apiTypeInfo;
                const formatter = CustomFormatters.get({
                    typeNames,
                    typeId,
                    size,
                    alignment,
                    isPointer,
                    canExpand,
                    arraySize: arraySize ?? 0,
                    hasValue,
                    members,
                    enumerators,
                });
                return {
                    typeNames,
                    isPointer,
                    typeId,
                    size,
                    alignment,
                    canExpand: canExpand && !formatter,
                    arraySize: arraySize ?? 0,
                    hasValue: hasValue || Boolean(formatter),
                    members,
                    enumerators,
                };
            }
        }
        finally {
            flush();
        }
    }
    async getMappedLines(rawModuleId, sourceFileURL) {
        const { flush, manage } = createEmbindPool();
        const moduleInfo = await this.getModuleInfo(rawModuleId);
        const sourceFile = moduleInfo.urlToFileName.get(sourceFileURL);
        if (!sourceFile) {
            throw new Error(`InternalError: Unknown URL ${sourceFileURL}`);
        }
        try {
            const mappedLines = manage(moduleInfo.dwarfSymbolsPlugin.GetMappedLines(rawModuleId, sourceFile));
            const error = manage(mappedLines.error);
            if (error) {
                throw new Error(`${moduleInfo.stringifyErrorCode(error.code)}: ${error.message}`);
            }
            const lines = mapVector(manage(mappedLines.MappedLines), l => l);
            return lines;
        }
        finally {
            flush();
        }
    }
    async evaluate(expression, context, stopId) {
        const valueInfo = await this.getValueInfo(expression, context, stopId);
        if (!valueInfo) {
            return null;
        }
        const wasm = new HostWasmInterface(this.hostInterface, stopId);
        const cxxObject = await CXXValue.create(this.lazyObjects, wasm, wasm.view, valueInfo);
        if (!cxxObject) {
            return {
                type: 'undefined',
                hasChildren: false,
                description: '<optimized out>',
            };
        }
        return cxxObject.asRemoteObject();
    }
    async getProperties(objectId) {
        const remoteObject = this.lazyObjects.get(objectId);
        if (!remoteObject) {
            return [];
        }
        const properties = await remoteObject.getProperties();
        const descriptors = [];
        for (const { name, property } of properties) {
            descriptors.push({ name, value: await property.asRemoteObject() });
        }
        return descriptors;
    }
    async releaseObject(objectId) {
        this.lazyObjects.release(objectId);
    }
}
async function createPlugin(hostInterface, resourceLoader, moduleConfigurations = DEFAULT_MODULE_CONFIGURATIONS, logPluginApiCalls = false) {
    const plugin = new DWARFLanguageExtensionPlugin(moduleConfigurations, resourceLoader, hostInterface);
    if (logPluginApiCalls) {
        const pluginLoggingProxy = {
            get: function (target, key) {
                if (typeof target[key] === 'function') {
                    return function () {
                        const args = [...arguments];
                        const jsonArgs = args.map(x => {
                            try {
                                return JSON.stringify(x);
                            }
                            catch {
                                return x.toString();
                            }
                        })
                            .join(', ');
                        // eslint-disable-next-line no-console
                        console.info(`${key}(${jsonArgs})`);
                        return target[key].apply(target, arguments);
                    };
                }
                return Reflect.get(target, key);
            },
        };
        return new Proxy(plugin, pluginLoggingProxy);
    }
    return plugin;
}

// Copyright 2022 Google LLC. All rights reserved.
function serializeWasmValue(value, buffer) {
    if (value instanceof ArrayBuffer) {
        const data = new Uint8Array(value);
        new Uint8Array(buffer).set(data);
        return data.byteLength || -1;
    }
    const view = new DataView(buffer);
    switch (value.type) {
        case 'i32':
            view.setInt32(0, value.value, true);
            return 1 /* SerializedWasmType.i32 */;
        case 'i64':
            view.setBigInt64(0, value.value, true);
            return 2 /* SerializedWasmType.i64 */;
        case 'f32':
            view.setFloat32(0, value.value, true);
            return 3 /* SerializedWasmType.f32 */;
        case 'f64':
            view.setFloat64(0, value.value, true);
            return 4 /* SerializedWasmType.f64 */;
        case 'v128':
            const [enc, a, b, c, d] = value.value.split(' ');
            view.setInt32(0, Number(a), true);
            view.setInt32(4, Number(b), true);
            view.setInt32(8, Number(c), true);
            view.setInt32(12, Number(d), true);
            return 5 /* SerializedWasmType.v128 */;
        default:
            throw new Error('cannot serialize non-numerical wasm type');
    }
}
function deserializeWasmMemory(buffer) {
    const result = new Uint8Array(buffer.byteLength);
    result.set(new Uint8Array(buffer));
    return result.buffer;
}
function deserializeWasmValue(buffer, type) {
    const view = new DataView(buffer);
    switch (type) {
        case 1 /* SerializedWasmType.i32 */:
            return { type: 'i32', value: view.getInt32(0, true) };
        case 2 /* SerializedWasmType.i64 */:
            return { type: 'i64', value: view.getBigInt64(0, true) };
        case 3 /* SerializedWasmType.f32 */:
            return { type: 'f32', value: view.getFloat32(0, true) };
        case 4 /* SerializedWasmType.f64 */:
            return { type: 'f64', value: view.getFloat64(0, true) };
        case 5 /* SerializedWasmType.v128 */:
            const a = view.getUint32(0, true);
            const b = view.getUint32(4, true);
            const c = view.getUint32(8, true);
            const d = view.getUint32(12, true);
            return {
                type: 'v128',
                value: `i32x4 0x${a.toString(16).padStart(8, '0')} 0x${b.toString(16).padStart(8, '0')} 0x${c.toString(16).padStart(8, '0')} 0x${d.toString(16).padStart(8, '0')}`
            };
    }
    // @ts-expect-error
    throw new Error('Invalid primitive wasm type');
}
const kMaxWasmValueSize = 4 + 4 + 4 * 10;

// Copyright 2022 Google LLC. All rights reserved.
class SynchronousIOMessage {
    buffer;
    constructor(bufferSize) {
        this.buffer = new SharedArrayBuffer(bufferSize);
    }
    static serialize(value, buffer) {
        return serializeWasmValue(value, buffer);
    }
}
/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
class WorkerRPC {
    nextRequestId = 0;
    channel;
    localHandler;
    requests = new Map();
    semaphore;
    constructor(channel, localHandler) {
        this.channel = channel;
        this.channel.onmessage = this.onmessage.bind(this);
        this.localHandler = localHandler;
        this.semaphore = new Int32Array(new SharedArrayBuffer(4));
    }
    sendMessage(method, ...params) {
        const requestId = this.nextRequestId++;
        const promise = new Promise((resolve, reject) => {
            this.requests.set(requestId, { resolve, reject });
        });
        this.channel.postMessage({ requestId, request: { method, params } });
        return promise;
    }
    sendMessageSync(message, method, ...params) {
        const requestId = this.nextRequestId++;
        Atomics.store(this.semaphore, 0, 0);
        this.channel.postMessage({
            requestId,
            sync_request: {
                request: { method, params },
                io_buffer: { semaphore: this.semaphore.buffer, data: message.buffer },
            },
        });
        while (Atomics.wait(this.semaphore, 0, 0) !== 'not-equal') {
        }
        const [response] = this.semaphore;
        return message.deserialize(response);
    }
    async onmessage(event) {
        if ('request' in event.data) {
            const { requestId, request } = event.data;
            try {
                const response = await this.localHandler[request.method](...request.params);
                this.channel.postMessage({ requestId, response });
            }
            catch (error) {
                this.channel.postMessage({ requestId, error: error }); // FIXME type?
            }
        }
        else if ('sync_request' in event.data) {
            /* eslint-disable-next-line @typescript-eslint/naming-convention */
            const { sync_request: { request, io_buffer } } = event.data;
            let signal = -1;
            try {
                const response = await this.localHandler[request.method](...request.params);
                signal = SynchronousIOMessage.serialize(response, io_buffer.data);
            }
            catch (error) {
                throw error;
            }
            finally {
                const semaphore = new Int32Array(io_buffer.semaphore);
                Atomics.store(semaphore, 0, signal);
                Atomics.notify(semaphore, 0);
            }
        }
        else {
            const { requestId } = event.data;
            const callbacks = this.requests.get(requestId);
            if (callbacks) {
                const { resolve, reject } = callbacks;
                if ('error' in event.data) {
                    reject(new Error(event.data.error));
                }
                else {
                    resolve(event.data.response);
                }
            }
        }
    }
}

// Copyright 2022 Google LLC. All rights reserved.
class SynchronousLinearMemoryMessage extends SynchronousIOMessage {
    deserialize(length) {
        if (length !== this.buffer.byteLength) {
            throw new Error('Expected length to match the internal buffer size');
        }
        return deserializeWasmMemory(this.buffer);
    }
}
class SynchronousWasmValueMessage extends SynchronousIOMessage {
    deserialize(type) {
        return deserializeWasmValue(this.buffer, type);
    }
}
class RPCInterface {
    rpc;
    #plugin;
    resourceLoader;
    get plugin() {
        if (!this.#plugin) {
            throw new Error('Worker is not yet initialized');
        }
        return this.#plugin;
    }
    constructor(port, resourceLoader) {
        this.rpc = new WorkerRPC(port, this);
        this.resourceLoader = resourceLoader;
    }
    getTypeInfo(_expression, _context) {
        throw new Error('Method not implemented.');
    }
    getFormatter(_expressionOrField, _context) {
        throw new Error('Method not implemented.');
    }
    getInspectableAddress(_field) {
        throw new Error('Method not implemented.');
    }
    getWasmLinearMemory(offset, length, stopId) {
        return this.rpc.sendMessageSync(new SynchronousLinearMemoryMessage(length), 'getWasmLinearMemory', offset, length, stopId);
    }
    getWasmLocal(local, stopId) {
        return this.rpc.sendMessageSync(new SynchronousWasmValueMessage(kMaxWasmValueSize), 'getWasmLocal', local, stopId);
    }
    getWasmGlobal(global, stopId) {
        return this.rpc.sendMessageSync(new SynchronousWasmValueMessage(kMaxWasmValueSize), 'getWasmGlobal', global, stopId);
    }
    getWasmOp(op, stopId) {
        return this.rpc.sendMessageSync(new SynchronousWasmValueMessage(kMaxWasmValueSize), 'getWasmOp', op, stopId);
    }
    evaluate(expression, context, stopId) {
        if (this.plugin.evaluate) {
            return this.plugin.evaluate(expression, context, stopId);
        }
        return Promise.resolve(null);
    }
    getProperties(objectId) {
        if (this.plugin.getProperties) {
            return this.plugin.getProperties(objectId);
        }
        return Promise.resolve([]);
    }
    releaseObject(objectId) {
        if (this.plugin.releaseObject) {
            return this.plugin.releaseObject(objectId);
        }
        return Promise.resolve();
    }
    addRawModule(rawModuleId, symbolsURL, rawModule) {
        return this.plugin.addRawModule(rawModuleId, symbolsURL, rawModule);
    }
    sourceLocationToRawLocation(sourceLocation) {
        return this.plugin.sourceLocationToRawLocation(sourceLocation);
    }
    rawLocationToSourceLocation(rawLocation) {
        return this.plugin.rawLocationToSourceLocation(rawLocation);
    }
    getScopeInfo(type) {
        return this.plugin.getScopeInfo(type);
    }
    listVariablesInScope(rawLocation) {
        return this.plugin.listVariablesInScope(rawLocation);
    }
    removeRawModule(rawModuleId) {
        return this.plugin.removeRawModule(rawModuleId);
    }
    getFunctionInfo(rawLocation) {
        return this.plugin.getFunctionInfo(rawLocation);
    }
    getInlinedFunctionRanges(rawLocation) {
        return this.plugin.getInlinedFunctionRanges(rawLocation);
    }
    getInlinedCalleesRanges(rawLocation) {
        return this.plugin.getInlinedCalleesRanges(rawLocation);
    }
    getMappedLines(rawModuleId, sourceFileURL) {
        return this.plugin.getMappedLines(rawModuleId, sourceFileURL);
    }
    async hello(moduleConfigurations, logPluginApiCalls) {
        this.#plugin = await createPlugin(this, this.resourceLoader, moduleConfigurations, logPluginApiCalls);
    }
}

// Copyright 2021 Google LLC. All rights reserved.
class ResourceLoader {
    async fetchSymbolsData(rawModule, url) {
        if (rawModule.code) {
            return { symbolsData: rawModule.code, symbolsDwpData: rawModule.dwp };
        }
        const symbolsResponse = await fetch(url.href, { mode: 'no-cors' });
        if (symbolsResponse.ok) {
            let symbolsDwpResponse = undefined;
            try {
                symbolsDwpResponse = await fetch(`${url.href}.dwp`, { mode: 'no-cors' });
            }
            catch (e) {
                // Unclear if this ever happens; usually if the file isn't there we
                // get a 404 response.
                console.error(`Failed to fetch dwp file: ${e}`);
            }
            if (!(symbolsDwpResponse && symbolsDwpResponse.ok)) {
                // Often this won't exist, but remember the missing file because if
                // we can't find symbol information later it is likely because this
                // file was missing.
                this.possiblyMissingSymbols = [`${url.pathname}.dwp`];
            }
            const [symbolsData, symbolsDwpData] = await Promise.all([
                symbolsResponse.arrayBuffer(),
                symbolsDwpResponse && symbolsDwpResponse.ok ? symbolsDwpResponse.arrayBuffer() : undefined,
            ]);
            return { symbolsData, symbolsDwpData };
        }
        const statusText = symbolsResponse.statusText || `status code ${symbolsResponse.status}`;
        if (rawModule.url !== url.href) {
            throw new Error(`NotFoundError: Unable to load debug symbols from '${url}' for the WebAssembly module '${rawModule.url}' (${statusText}), double-check the parameter to -gseparate-dwarf in your Emscripten link step`);
        }
        throw new Error(`NotFoundError: Unable to load debug symbols from '${url}' (${statusText})`);
    }
    getModuleFileName(rawModuleId) {
        return `${self.btoa(rawModuleId)}.wasm`.replace(/\//g, '_');
    }
    async loadSymbols(rawModuleId, rawModule, symbolsURL, fileSystem) {
        const { symbolsData, symbolsDwpData } = await this.fetchSymbolsData(rawModule, symbolsURL);
        const symbolsFileName = this.getModuleFileName(rawModuleId);
        const symbolsDwpFileName = symbolsDwpData && `${symbolsFileName}.dwp`;
        // This file is sometimes preserved on reload, causing problems.
        try {
            fileSystem.unlink('/' + symbolsFileName);
        }
        catch (_) {
        }
        fileSystem.createDataFile('/', symbolsFileName, new Uint8Array(symbolsData), true /* canRead */, false /* canWrite */, true /* canOwn */);
        if (symbolsDwpData && symbolsDwpFileName) {
            fileSystem.createDataFile('/', symbolsDwpFileName, new Uint8Array(symbolsDwpData), true /* canRead */, false /* canWrite */, true /* canOwn */);
        }
        return { symbolsFileName, symbolsDwpFileName };
    }
    createSymbolsBackendModulePromise() {
        const url = new URL('SymbolsBackend.wasm', import.meta.url);
        return fetch(url.href, { credentials: 'same-origin' }).then(response => {
            if (!response.ok) {
                throw new Error(response.statusText);
            }
            return WebAssembly.compileStreaming(response);
        });
    }
    possiblyMissingSymbols;
}

// Copyright 2022 Google LLC. All rights reserved.
new RPCInterface(globalThis, new ResourceLoader());
