#!/usr/bin/env python3
"""
make_xex.py – Xbox 360 Executable (XEX2) Generator for Homebrew (libxenon / JTAG / RGH)
RustCraft Project

Packages an input binary or ELF into a valid, standard uncompressed, unencrypted
Xbox 360 XEX2 executable (default.xex) ready to launch on any exploited Xbox 360 console
(RGH 1.2, RGH 2.0, RGH 3.0, JTAG) via Aurora, FreestyleDash, XeXMenu, DashLaunch, or Xenia emulator.
"""

import sys
import os
import struct
import hashlib
import argparse

# Magic constants
XEX2_MAGIC = b'XEX2'
MODULE_FLAG_TITLE = 0x00000001

# Optional header keys
XEX_HEADER_FILE_FORMAT_INFO = 0x000003FF
XEX_HEADER_ORIGINAL_BASE_ADDRESS = 0x00010001
XEX_HEADER_ENTRY_POINT = 0x00010100
XEX_HEADER_IMAGE_BASE_ADDRESS = 0x00010201
XEX_HEADER_ORIGINAL_PE_NAME = 0x000183FF
XEX_HEADER_DEFAULT_STACK_SIZE = 0x00020200
XEX_HEADER_DEFAULT_HEAP_SIZE = 0x00020401
XEX_HEADER_SYSTEM_FLAGS = 0x00030000
XEX_HEADER_EXECUTION_INFO = 0x00040006
XEX_HEADER_TITLE_WORKSPACE_SIZE = 0x00040201

# Encryption and compression
XEX_ENCRYPTION_NONE = 0
XEX_COMPRESSION_NONE = 0

# Media flags (All media allowed: HDD, USB Mass Storage, DVD, Flash)
XEX_MEDIA_ALL = 0xFFFFFFFF
XEX_REGION_ALL = 0xFFFFFFFF

DEFAULT_TITLE_ID = 0xFFFE07D1  # Standard Universal Xbox 360 Homebrew Title ID (XeXMenu / XellLaunch)
DEFAULT_BASE_ADDRESS = 0x82000000
DEFAULT_ENTRY_POINT = 0x82001000


# PE / COFF Constants for Xbox 360
IMAGE_DOS_SIGNATURE = b'MZ'
IMAGE_NT_SIGNATURE = b'PE\x00\x00'
IMAGE_FILE_MACHINE_POWERPCBE = 0x01F2  # Xbox 360 Xenon PowerPC Big-Endian
IMAGE_FILE_EXECUTABLE_IMAGE = 0x0002
IMAGE_FILE_32BIT_MACHINE = 0x0100
IMAGE_NT_OPTIONAL_HDR32_MAGIC = 0x010B
IMAGE_SUBSYSTEM_XBOX = 14  # Xbox Subsystem
IMAGE_SIZEOF_NT_OPTIONAL_HEADER = 224

PE_SECTION_TEXT_FLAGS = 0x60000020  # CODE | EXECUTE | READ
PE_SECTION_DATA_FLAGS = 0xC0000040  # INITIALIZED_DATA | READ | WRITE


def align_up(val, alignment):
    return (val + alignment - 1) & ~(alignment - 1)


def parse_elf_segments(elf_data):
    """Parses an ELF32 Big-Endian binary and extracts loadable segments."""
    if len(elf_data) < 52 or elf_data[:4] != b'\x7fELF':
        return None  # Not an ELF file

    is_be = (elf_data[5] == 2)
    fmt = '>H' if is_be else '<H'
    fmt_i = '>I' if is_be else '<I'

    e_entry = struct.unpack_from(fmt_i, elf_data, 24)[0]
    ph_off = struct.unpack_from(fmt_i, elf_data, 28)[0]
    phent_size = struct.unpack_from(fmt, elf_data, 42)[0]
    ph_num = struct.unpack_from(fmt, elf_data, 44)[0]

    segments = []
    for i in range(ph_num):
        off = ph_off + i * phent_size
        p_type = struct.unpack_from(fmt_i, elf_data, off)[0]
        if p_type == 1:  # PT_LOAD
            p_offset = struct.unpack_from(fmt_i, elf_data, off + 4)[0]
            p_vaddr = struct.unpack_from(fmt_i, elf_data, off + 8)[0]
            p_filesz = struct.unpack_from(fmt_i, elf_data, off + 16)[0]
            p_memsz = struct.unpack_from(fmt_i, elf_data, off + 20)[0]
            p_flags = struct.unpack_from(fmt_i, elf_data, off + 24)[0]
            seg_data = elf_data[p_offset:p_offset + p_filesz]
            # Only consider segments in the Xbox 360 title space (>= 0x82000000)
            if p_vaddr >= DEFAULT_BASE_ADDRESS:
                segments.append({
                    'vaddr': p_vaddr,
                    'filesz': p_filesz,
                    'memsz': p_memsz,
                    'flags': p_flags,
                    'data': seg_data,
                })
    return segments, e_entry


def extract_elf_symbols(data):
    """Extracts virtual addresses of symbols from ELF32 binary."""
    if len(data) < 52 or data[:4] != b'\x7fELF':
        return {}
    is_be = (data[5] == 2)
    endian = ">" if is_be else "<"
    sh_off = struct.unpack_from(endian + "I", data, 32)[0]
    shent_size = struct.unpack_from(endian + "H", data, 46)[0]
    sh_num = struct.unpack_from(endian + "H", data, 48)[0]

    sections = []
    for i in range(sh_num):
        off = sh_off + i * shent_size
        s_name, s_type, s_flags, s_addr, s_offset, s_size, s_link, s_info = struct.unpack_from(endian + "8I", data, off)
        sections.append({"type": s_type, "offset": s_offset, "size": s_size, "link": s_link})

    symtab = next((s for s in sections if s["type"] == 2), None)
    if not symtab or symtab["link"] >= len(sections):
        return {}
    strtab = sections[symtab["link"]]
    strings = data[strtab["offset"]:strtab["offset"] + strtab["size"]]

    sym_data = data[symtab["offset"]:symtab["offset"] + symtab["size"]]
    syms = {}
    for i in range(0, len(sym_data), 16):
        st_name, st_value, st_size = struct.unpack_from(endian + "3I", sym_data, i)
        if st_name < len(strings):
            name = strings[st_name:strings.find(b"\x00", st_name)].decode("ascii", "replace")
            if name:
                syms[name] = st_value
    return syms


def create_xbox_pe(payload, entry_point=DEFAULT_ENTRY_POINT, base_address=DEFAULT_BASE_ADDRESS):
    """
    Wraps raw PowerPC executable code or ELF segments into a valid Xbox 360 PE32 binary
    with IMAGE_FILE_MACHINE_POWERPCBE (0x01F2) and IMAGE_SUBSYSTEM_XBOX (14),
    matching Xenia's XexModule::ReadPEHeaders() and Xbox 360 kernel specifications.
    """
    parsed = parse_elf_segments(payload)
    if parsed:
        elf_segs, elf_entry = parsed
        if elf_entry >= base_address:
            entry_point = elf_entry

        max_file_end = 0x2000
        max_mem_end = 0x3000
        code_end = 0x2000
        for seg in elf_segs:
            rva = seg['vaddr'] - base_address
            f_end = rva + seg['filesz']
            m_end = rva + max(seg['filesz'], seg['memsz'])
            if seg['flags'] & 1:  # Executable (PF_X)
                if f_end > code_end:
                    code_end = f_end
            if f_end > max_file_end:
                max_file_end = f_end
            if m_end > max_mem_end:
                max_mem_end = m_end

        code_end_aligned = align_up(code_end, 4096)
        text_size = code_end_aligned - 0x1000
        data_rva = code_end_aligned
        data_virtual_size = align_up(max_mem_end, 4096) - data_rva
        data_raw_size = align_up(max(0, max_file_end - data_rva), 512)
        total_file_size = align_up(data_rva + data_raw_size, 512)
        image_size = align_up(max_mem_end, 4096)

        pe_image = bytearray(total_file_size)
        for seg in elf_segs:
            rva = seg['vaddr'] - base_address
            if rva + len(seg['data']) <= len(pe_image):
                pe_image[rva:rva + len(seg['data'])] = seg['data']
            elif rva < len(pe_image):
                avail = len(pe_image) - rva
                pe_image[rva:rva + avail] = seg['data'][:avail]
        code_data = pe_image[0x1000:]
    else:
        # Check if already a PE (starts with MZ)
        if len(payload) > 64 and payload[:2] == IMAGE_DOS_SIGNATURE:
            return payload, entry_point, 1
        code_data = payload
        text_size = 0x1000
        data_rva = 0x2000
        data_raw_size = len(code_data) if len(code_data) > 0x1000 else 0x1000
        data_virtual_size = data_raw_size
        code_end_aligned = 0x2000
        image_size = align_up(0x1000 + len(code_data), 4096)

    aligned_code = code_data
    code_size = len(aligned_code)

    # 1. DOS Header (64 bytes, Little-Endian standard for PE)
    dos_header = bytearray(64)
    dos_header[0:4] = b'MZ\x90\x00'
    struct.pack_into('<H', dos_header, 4, 0x0003)   # e_cp
    struct.pack_into('<H', dos_header, 8, 0x0004)   # e_cparhdr (64 bytes = 4 paras)
    struct.pack_into('<H', dos_header, 10, 0xFFFF)  # e_minalloc
    struct.pack_into('<H', dos_header, 14, 0x0000)  # e_sp
    struct.pack_into('<H', dos_header, 20, 0x0000)  # e_ip
    struct.pack_into('<H', dos_header, 24, 0x0040)  # e_lfarlc
    struct.pack_into('<I', dos_header, 60, 0x00000080) # e_lfanew = 0x80

    # DOS Stub (64 bytes)
    dos_stub = b'RustCraft Xbox 360 Xenon (c) 2026 Homebrew Project\r\n\x24\x00\x00\x00'
    dos_stub = dos_stub.ljust(64, b'\x00')

    # 2. NT Header (PE\0\0) at offset 0x80
    nt_signature = IMAGE_NT_SIGNATURE

    # IMAGE_FILE_HEADER (20 bytes, Little-Endian)
    file_header = struct.pack(
        '<HHIIIHH',
        IMAGE_FILE_MACHINE_POWERPCBE,  # Machine = 0x01F2
        2,                             # NumberOfSections = 2 (.text, .data)
        0x67000000,                    # TimeDateStamp
        0,                             # PointerToSymbolTable
        0,                             # NumberOfSymbols
        IMAGE_SIZEOF_NT_OPTIONAL_HEADER, # SizeOfOptionalHeader = 224
        IMAGE_FILE_EXECUTABLE_IMAGE | IMAGE_FILE_32BIT_MACHINE  # 0x0102
    )

    # IMAGE_OPTIONAL_HEADER32 (224 bytes, Little-Endian)
    entry_rva = (entry_point - base_address) if entry_point >= base_address else 0x1000
    opt_header = struct.pack(
        '<HBBIIIIII IIIHHHHHH IIIIHHIIIIII',
        IMAGE_NT_OPTIONAL_HDR32_MAGIC, # 0x010B
        1, 0,                          # MajorLinker, MinorLinker
        text_size,                     # SizeOfCode
        data_raw_size,                 # SizeOfInitializedData
        data_virtual_size - data_raw_size, # SizeOfUninitializedData
        entry_rva,                     # AddressOfEntryPoint (RVA)
        0x1000,                        # BaseOfCode
        data_rva,                      # BaseOfData
        base_address,                  # ImageBase (0x82000000)
        4096,                          # SectionAlignment
        512,                           # FileAlignment
        1, 0,                          # OS Major, Minor
        1, 0,                          # Image Major, Minor
        1, 0,                          # Subsystem Major, Minor
        0,                             # Win32Version
        image_size,                    # SizeOfImage
        0x1000,                        # SizeOfHeaders (4096)
        0,                             # Checksum
        IMAGE_SUBSYSTEM_XBOX,          # Subsystem = 14
        0,                             # DllCharacteristics
        0x00100000,                    # SizeOfStackReserve (1 MB)
        0x00010000,                    # SizeOfStackCommit (64 KB)
        0x04000000,                    # SizeOfHeapReserve (64 MB)
        0x00100000,                    # SizeOfHeapCommit (1 MB)
        0,                             # LoaderFlags
        16                             # NumberOfRvaAndSizes
    )
    data_dirs = b'\x00' * (16 * 8)
    full_opt_header = opt_header + data_dirs

    # Section 1: .text (40 bytes, Little-Endian)
    text_section = struct.pack(
        '<8sIIIIIIHHI',
        b'.text\x00\x00\x00',
        text_size,                     # VirtualSize
        0x1000,                        # VirtualAddress (RVA)
        align_up(text_size, 512),      # SizeOfRawData
        0x1000,                        # PointerToRawData
        0, 0, 0, 0,
        PE_SECTION_TEXT_FLAGS
    )

    # Section 2: .data (40 bytes, Little-Endian)
    data_section = struct.pack(
        '<8sIIIIIIHHI',
        b'.data\x00\x00\x00',
        data_virtual_size,             # VirtualSize (including BSS in RAM)
        data_rva,                      # VirtualAddress (RVA)
        data_raw_size,                 # SizeOfRawData (only initialized file bytes)
        data_rva,                      # PointerToRawData
        0, 0, 0, 0,
        PE_SECTION_DATA_FLAGS
    )

    headers = bytearray()
    headers.extend(dos_header)
    headers.extend(dos_stub)
    headers.extend(nt_signature)
    headers.extend(file_header)
    headers.extend(full_opt_header)
    headers.extend(text_section)
    headers.extend(data_section)

    if len(headers) < 0x1000:
        headers.extend(b'\x00' * (0x1000 - len(headers)))

    pe_file = bytearray()
    pe_file.extend(headers)
    pe_file.extend(aligned_code)
    code_end_val = code_end_aligned if parsed else 0x2000
    return bytes(pe_file), entry_point, code_end_val


def build_xex(input_data, output_path, title_id=DEFAULT_TITLE_ID, title_name="RustCraft"):
    # 1. Wrap raw payload or ELF into valid Xbox 360 PE32 image
    pe_payload, entry_point, code_end = create_xbox_pe(input_data, entry_point=DEFAULT_ENTRY_POINT, base_address=DEFAULT_BASE_ADDRESS)

    # Xbox 360 default MMU page size is 64 KB (65536 bytes, 0x10000).
    PAGE_SIZE_XEX = 65536
    payload_size = len(pe_payload)
    aligned_payload_size = align_up(payload_size, PAGE_SIZE_XEX)
    if aligned_payload_size > payload_size:
        pe_payload += b'\x00' * (aligned_payload_size - payload_size)

    page_count = max(1, aligned_payload_size // PAGE_SIZE_XEX)
    code_pages = min(page_count, max(1, align_up(code_end, PAGE_SIZE_XEX) // PAGE_SIZE_XEX))
    data_pages = page_count - code_pages

    payload_sha1 = hashlib.sha1(pe_payload).digest()

    # 2. Prepare opt-header data pool
    pool = bytearray()

    # File Format Info (16 bytes)
    # struct xex2_opt_file_format_info: info_size(u32), enc_type(u16), comp_type(u16), compression_info
    off_file_format = len(pool)
    pool.extend(struct.pack('>IIII', 16, (XEX_ENCRYPTION_NONE << 16) | XEX_COMPRESSION_NONE, 0, 0))

    # Execution Info (24 bytes)
    # media_id(4), version(4), base_version(4), title_id(4), platform(1), exec_table(1), disc_num(1), disc_count(1), savegame_id(4)
    off_exec_info = len(pool)
    pool.extend(struct.pack(
        '>IIIIBBBB I',
        0x00000000,      # media_id
        0x00010000,      # version 1.0.0.0
        0x00010000,      # base_version
        title_id,        # title_id
        1,               # platform = Xbox 360
        1,               # executable_table = Title
        1,               # disc_number = 1
        1,               # disc_count = 1
        0x00000000       # savegame_id
    ))

    # Original PE Name (len u32, null-terminated string)
    pe_name = f"{title_name.lower()}.exe".encode('ascii') + b'\x00'
    pe_name_aligned = pe_name + b'\x00' * (align_up(len(pe_name), 4) - len(pe_name))
    off_pe_name = len(pool)
    pool.extend(struct.pack('>I', len(pe_name_aligned) + 4))
    pool.extend(pe_name_aligned)

    # 2b. Import Libraries Header (XEX_HEADER_IMPORT_LIBRARIES = 0x000103FF)
    # Allows Xbox 360 kernel (xboxkrnl.exe) and dashboard (xam.xex) to patch ordinal imports.
    syms = extract_elf_symbols(input_data)

    XBOXKRNL_EXPORTS = [
        (3,   "DbgPrint"),
        (25,  "ExTerminateThread"),
        (40,  "HalReturnToFirmware"),
        (90,  "KeDelayExecutionThread"),
        (207, "NtClose"),
        (210, "NtCreateFile"),
        (223, "NtOpenFile"),
        (232, "NtQueryInformationFile"),
        (240, "NtReadFile"),
        (247, "NtSetInformationFile"),
        (255, "NtWriteFile"),
        (300, "RtlInitAnsiString"),
    ]

    XAM_EXPORTS = [
        (401, "XamInputGetState"),
    ]

    def pad_str(s):
        b = s.encode('ascii') + b'\x00'
        pad = (4 - (len(b) % 4)) % 4
        return b + b'\x00' * pad

    str_krnl = pad_str("xboxkrnl.exe")
    str_xam = pad_str("xam.xex")
    str_table_data = str_krnl + str_xam
    str_table_struct = struct.pack('>II', len(str_table_data), 2) + str_table_data

    # Library 0: xboxkrnl.exe
    krnl_slots = []
    for ord_val, name in XBOXKRNL_EXPORTS:
        desc_addr = syms.get(f"imp_desc_{name}", 0)
        thunk_addr = syms.get(name, 0)
        krnl_slots.append(desc_addr)
        krnl_slots.append(thunk_addr)

    lib_krnl_size = 0x28 + len(krnl_slots) * 4
    lib_krnl = bytearray()
    lib_krnl.extend(struct.pack('>I', lib_krnl_size))
    lib_krnl.extend(b'\x00' * 20)
    lib_krnl.extend(struct.pack('>III', 0, 0, 0))
    lib_krnl.extend(struct.pack('>HH', 0, len(krnl_slots)))
    for addr in krnl_slots:
        lib_krnl.extend(struct.pack('>I', addr))

    # Library 1: xam.xex
    xam_slots = []
    for ord_val, name in XAM_EXPORTS:
        desc_addr = syms.get(f"imp_desc_{name}", 0)
        thunk_addr = syms.get(name, 0)
        xam_slots.append(desc_addr)
        xam_slots.append(thunk_addr)

    lib_xam_size = 0x28 + len(xam_slots) * 4
    lib_xam = bytearray()
    lib_xam.extend(struct.pack('>I', lib_xam_size))
    lib_xam.extend(b'\x00' * 20)
    lib_xam.extend(struct.pack('>III', 0, 0, 0))
    lib_xam.extend(struct.pack('>HH', 1, len(xam_slots)))
    for addr in xam_slots:
        lib_xam.extend(struct.pack('>I', addr))

    total_import_size = 4 + len(str_table_struct) + len(lib_krnl) + len(lib_xam)
    import_libraries_blob = struct.pack('>I', total_import_size) + str_table_struct + lib_krnl + lib_xam

    off_import_libraries = len(pool)
    pool.extend(import_libraries_blob)

    # 3. Security Info Structure (xex2_security_info)
    # Generate page descriptors with authentic SHA1 digest for each segment
    code_split = code_pages * PAGE_SIZE_XEX
    code_bytes = pe_payload[:code_split]
    code_sha1 = hashlib.sha1(code_bytes).digest()

    if data_pages > 0:
        data_bytes = pe_payload[code_split:]
        data_sha1 = hashlib.sha1(data_bytes).digest()
        code_desc = struct.pack('>I', (code_pages << 4) | 1) + code_sha1
        data_desc = struct.pack('>I', (data_pages << 4) | 2) + data_sha1
        page_desc = code_desc + data_desc
        page_desc_count = 2
    else:
        page_desc = struct.pack('>I', (page_count << 4) | 1) + code_sha1
        page_desc_count = 1

    sec_info_size = 0x184 + len(page_desc)

    sec_info = bytearray()
    sec_info.extend(struct.pack('>II', sec_info_size, aligned_payload_size))
    sec_info.extend(b'\x00' * 256)       # RSA signature
    sec_info.extend(struct.pack('>III', 0x174, 0, DEFAULT_BASE_ADDRESS)) # unk_108 = 0x174, image_flags = 0 (64KB pages), load_addr
    sec_info.extend(payload_sha1)        # section_digest
    sec_info.extend(struct.pack('>I', 0)) # import_table_count
    sec_info.extend(b'\x00' * 20)        # import_table_digest
    sec_info.extend(b'\x00' * 16)        # xgd2_media_id
    sec_info.extend(b'\x00' * 16)        # aes_key (unencrypted)
    sec_info.extend(struct.pack('>I', 0)) # export_table
    sec_info.extend(b'\x00' * 20)        # header_digest
    sec_info.extend(struct.pack('>III', XEX_REGION_ALL, XEX_MEDIA_ALL, page_desc_count))
    sec_info.extend(page_desc)

    # 4. Assemble Optional Headers table
    XEX_HEADER_IMPORT_LIBRARIES = 0x000103FF
    opt_headers = [
        (XEX_HEADER_SYSTEM_FLAGS, 0x00000001),             # NO_FORCED_REBOOT
        (XEX_HEADER_ENTRY_POINT, entry_point),
        (XEX_HEADER_IMAGE_BASE_ADDRESS, DEFAULT_BASE_ADDRESS),
        (XEX_HEADER_ORIGINAL_BASE_ADDRESS, DEFAULT_BASE_ADDRESS),
        (XEX_HEADER_DEFAULT_STACK_SIZE, 0x00100000),       # 1 MB Stack
        (XEX_HEADER_DEFAULT_HEAP_SIZE, 0x02000000),        # 32 MB Heap
        (XEX_HEADER_TITLE_WORKSPACE_SIZE, 0x00100000),     # 1 MB Workspace
    ]

    header_count = len(opt_headers) + 4 # + file_format, exec_info, pe_name, import_libraries
    opt_table_size = header_count * 8
    pool_offset_in_header = 24 + opt_table_size

    opt_headers.append((XEX_HEADER_FILE_FORMAT_INFO, pool_offset_in_header + off_file_format))
    opt_headers.append((XEX_HEADER_EXECUTION_INFO, pool_offset_in_header + off_exec_info))
    opt_headers.append((XEX_HEADER_ORIGINAL_PE_NAME, pool_offset_in_header + off_pe_name))
    opt_headers.append((XEX_HEADER_IMPORT_LIBRARIES, pool_offset_in_header + off_import_libraries))

    sec_info_offset_in_header = pool_offset_in_header + len(pool)
    raw_header_size = sec_info_offset_in_header + len(sec_info)
    total_header_size = align_up(raw_header_size, 4096)

    # 5. Pack Main Header
    main_header = struct.pack(
        '>4sIIIII',
        XEX2_MAGIC,
        MODULE_FLAG_TITLE,           # Title module (0x00000001)
        total_header_size,
        0,                           # reserved
        sec_info_offset_in_header,   # security_offset
        len(opt_headers)             # header_count
    )

    # Pack Opt Headers table
    opt_table = bytearray()
    for key, val in opt_headers:
        opt_table.extend(struct.pack('>II', key, val))

    # Assemble full file
    xex_file = bytearray()
    xex_file.extend(main_header)
    xex_file.extend(opt_table)
    xex_file.extend(pool)
    xex_file.extend(sec_info)

    # Pad to total_header_size
    if len(xex_file) < total_header_size:
        xex_file.extend(b'\x00' * (total_header_size - len(xex_file)))

    # Append executable payload (PE32)
    xex_file.extend(pe_payload)

    # Write output file
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, 'wb') as f:
        f.write(xex_file)

    print(f"✓ XEX2 binary successfully generated: {output_path}")
    print(f"  Header size: {total_header_size} bytes (0x{total_header_size:X})")
    print(f"  Payload size: {aligned_payload_size} bytes (0x{aligned_payload_size:X})")
    print(f"  Total XEX size: {len(xex_file)} bytes (0x{len(xex_file):X})")
    print(f"  Title ID: 0x{title_id:08X} | Base: 0x{DEFAULT_BASE_ADDRESS:08X} | Entry: 0x{DEFAULT_ENTRY_POINT:08X}")
    print(f"  Exploit Compatibility: RGH 1.2 / 2.0 / 3.0, JTAG, Aurora, XeXMenu, DashLaunch, Xenia")


def main():
    parser = argparse.ArgumentParser(description="Generate Xbox 360 XEX2 executable from binary/ELF")
    parser.add_argument("input", help="Path to input binary or ELF")
    parser.add_argument("-o", "--output", default="default.xex", help="Path to output .xex file (default: default.xex)")
    parser.add_argument("--title-id", default="58580864", help="Hex Title ID (default: 58580864)")
    parser.add_argument("--title-name", default="RustCraft", help="Title Name (default: RustCraft)")

    args = parser.parse_args()

    title_id = int(args.title_id, 16)

    if not os.path.exists(args.input):
        print(f"Error: Input file '{args.input}' not found.", file=sys.stderr)
        sys.exit(1)

    with open(args.input, "rb") as f:
        data = f.read()

    build_xex(data, args.output, title_id=title_id, title_name=args.title_name)


if __name__ == "__main__":
    main()
