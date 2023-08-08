// Parse the custom name section of the WASM module according to this spec:
// https://webassembly.github.io/spec/core/appendix/custom.html#custom-sections
export function parseNameSection(nameSection: ArrayBuffer): Map<number, string> {
    const nameSectionView = new DataView(nameSection);
    const fnNameMap = new Map<number, string>;
    let offset = 0;
    try {
        while (offset < nameSection.byteLength) {
            const subsectionId = readByte(nameSectionView, offset);
            offset += subsectionId.bytesRead;

            const subsectionLength = readLEB128(nameSectionView, offset);
            offset += subsectionLength.bytesRead;
            const subsectionEnd = offset + subsectionLength.value;

            // function name subsection is subsection id 1
            if (subsectionId.value === 0x01) {
                const nameMapLength = readLEB128(nameSectionView, offset);
                offset += nameMapLength.bytesRead;

                // process namemap
                for (let nameMapCount = 0; nameMapCount < nameMapLength.value; nameMapCount++) {
                    const nameIdx = readLEB128(nameSectionView, offset);
                    offset += nameIdx.bytesRead;

                    const nameLength = readLEB128(nameSectionView, offset);
                    offset += nameLength.bytesRead;

                    const fnName = new TextDecoder().decode(nameSection.slice(offset, offset + nameLength.value));
                    offset += nameLength.value;

                    fnNameMap.set(nameIdx.value, fnName);
                }
                if (offset < subsectionEnd) {
                    console.warn("suspicious: at end of name map, but not its name subsection");
                    offset = subsectionEnd;
                }
            } else {
                // skip this subsection
                offset = subsectionEnd;
            }
        }
    } catch (error) {
        // WASM probably has a corrupt name section, log the error and return what we got
        console.error(error);
    }
    return fnNameMap;
}

function readLEB128(view: DataView, offset: number): { value: number, bytesRead: number } {
    let result = 0;
    let bytesRead = 0;
    let shift = 0;
    let byte;

    do {
        byte = view.getUint8(offset + bytesRead);
        result |= (byte & 0x7F) << shift;
        shift += 7;
        bytesRead++;
    } while (byte & 0x80);
    return { value: result, bytesRead };
}

function readByte(view: DataView, offset: number): { value: number, bytesRead: number } {
    const byte = view.getUint8(offset);
    return { value: byte, bytesRead: 1 };
}
