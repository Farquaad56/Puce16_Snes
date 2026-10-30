#!/usr/bin/env python
"""ppu_ref.py - INDEPENDENT reference renderer for the Puce16 PPU + scene generator (maintenance tool of Claude).

The reference is written from docs/specs/05_PPU.md and 05b_PPU_RENDER.md (and the refinements of tasks E05-xx),
NOT from the Rust code.  Task E05-19 compares the Rust PPU with the frame hashes computed here.

    python docs/tools/ppu_ref.py selftest          # hand-checkable cases of the reference itself
    python docs/tools/ppu_ref.py generate          # write docs/specs/data/ppu_scenes.txt and ppu_scenes.expected
    python docs/tools/ppu_ref.py verify            # regenerate in memory and compare with the files (exit 1 if different)
    python docs/tools/ppu_ref.py images [DIR]      # write <scene>.ppm for every scene (default: tests/out/ppu_ref)
    python docs/tools/ppu_ref.py list

Scene file format (see task E05-19):  scene/vram/cgram/oam/reg/at/end directives, values in hex except cgram index,
oam address (decimal) and the line of `at` (decimal).  The reference executes the TEXT file (parse -> simulate), so the
file is the single source of truth.  Integers only, no external module (Python 3.8+).
"""
import os
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "docs" / "specs" / "data"
SCENES_TXT = DATA / "ppu_scenes.txt"
EXPECTED = DATA / "ppu_scenes.expected"

BACKDROP = 5
BPP = {0: [2, 2, 2, 2], 1: [4, 4, 2, 0], 2: [4, 4, 0, 0], 3: [8, 4, 0, 0], 4: [8, 2, 0, 0], 5: [4, 2, 0, 0], 6: [4, 0, 0, 0], 7: [0, 0, 0, 0]}
OBJ_SIZES = {0: ((8, 8), (16, 16)), 1: ((8, 8), (32, 32)), 2: ((8, 8), (64, 64)), 3: ((16, 16), (32, 32)),
             4: ((16, 16), (64, 64)), 5: ((32, 32), (64, 64)), 6: ((16, 32), (32, 64)), 7: ((16, 32), (32, 32))}


def sext13(x):
    x &= 0x1FFF
    return x - 0x2000 if x & 0x1000 else x


def sext16(x):
    x &= 0xFFFF
    return x - 0x10000 if x & 0x8000 else x


def fnv64(words32):
    h = 0xCBF29CE484222325
    for w in words32:
        for b in struct.pack("<I", w):
            h ^= b
            h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def frame_hash(width, height, pixels):
    return fnv64([width, height] + list(pixels))


# =====================================================================================================
#  Reference PPU
# =====================================================================================================

class RefPpu:
    def __init__(self):
        self.vram = [0] * 0x8000
        self.cgram = [0] * 256
        self.oam = [0] * 544
        self.v = 0
        self.forced_blank = True
        self.brightness = 0
        self.obj_size = 0
        self.obj_name_gap = 0
        self.obj_base = 0
        self.oamadd_reg = 0
        self.internal_oamadd = 0
        self.obj_rot = False
        self.oam_latch = 0
        self.bg_mode = 0
        self.bg3_prio = False
        self.tile16 = [False] * 4
        self.mosaic_size = 0
        self.mosaic_en = 0
        self.map_base = [0] * 4
        self.map_x2 = [0] * 4
        self.map_y2 = [0] * 4
        self.chr_base = [0] * 4
        self.hofs = [0] * 4
        self.vofs = [0] * 4
        self.bgofs_latch = 0
        self.bghofs_latch = 0
        self.mode7_latch = 0
        self.vmain = 0
        self.vmadd = 0
        self.vram_latch = 0
        self.m7sel = 0
        self.m7 = [0, 0, 0, 0]
        self.m7_cx = 0
        self.m7_cy = 0
        self.m7_hofs = 0
        self.m7_vofs = 0
        self.cgram_addr = 0
        self.cgram_toggle = 0
        self.cgram_latch = 0
        self.win_sel = [0, 0, 0]
        self.wh = [0, 0, 0, 0]
        self.win_log_bg = 0
        self.win_log_obj = 0
        self.win_log_col = 0
        self.tm = self.ts = self.tmw = self.tsw = 0
        self.cgwsel = 0
        self.cgadsub = 0
        self.fixed = [0, 0, 0]
        self.setini = 0
        self.line_obj = [None] * 256

    # ------------------------------------------------------------------ register writes (spec 05 section 2)
    def vram_ok(self):
        return self.forced_blank or self.v >= 225

    def remap(self, a):
        m = (self.vmain >> 2) & 3
        if m == 0:
            return a
        if m == 1:
            return (a & 0xFF00) | ((a & 0x00E0) >> 5) | ((a & 0x001F) << 3)
        if m == 2:
            return (a & 0xFE00) | ((a & 0x01C0) >> 6) | ((a & 0x003F) << 3)
        return (a & 0xFC00) | ((a & 0x0380) >> 7) | ((a & 0x007F) << 3)

    def vstep(self):
        return {0: 1, 1: 32, 2: 128, 3: 128}[self.vmain & 3]

    def write(self, reg, v):
        v &= 0xFF
        if reg == 0x00:
            self.forced_blank = bool(v & 0x80)
            self.brightness = v & 15
        elif reg == 0x01:
            self.obj_size = v >> 5
            self.obj_name_gap = (((v >> 3) & 3) + 1) << 12
            self.obj_base = (v & 7) << 13
        elif reg == 0x02:
            self.oamadd_reg = (self.oamadd_reg & 0x100) | v
            self.internal_oamadd = (self.oamadd_reg & 0x1FF) << 1
        elif reg == 0x03:
            self.oamadd_reg = (self.oamadd_reg & 0xFF) | ((v & 1) << 8)
            self.obj_rot = bool(v & 0x80)
            self.internal_oamadd = (self.oamadd_reg & 0x1FF) << 1
        elif reg == 0x04:
            a = self.internal_oamadd
            ok = self.forced_blank or self.v >= 225
            if a & 0x200:
                if ok:
                    self.oam[0x200 + (a & 0x1F)] = v
            else:
                if a & 1 == 0:
                    self.oam_latch = v
                elif ok:
                    self.oam[a - 1] = self.oam_latch
                    self.oam[a] = v
            self.internal_oamadd = (a + 1) & 0x3FF
        elif reg == 0x05:
            self.bg_mode = v & 7
            self.bg3_prio = bool(v & 8)
            for n in range(4):
                self.tile16[n] = bool((v >> (4 + n)) & 1)
        elif reg == 0x06:
            self.mosaic_size = v >> 4
            self.mosaic_en = v & 15
        elif 0x07 <= reg <= 0x0A:
            n = reg - 0x07
            self.map_base[n] = (v >> 2) << 10
            self.map_x2[n] = v & 1
            self.map_y2[n] = (v >> 1) & 1
        elif reg == 0x0B:
            self.chr_base[0] = (v & 15) << 12
            self.chr_base[1] = (v >> 4) << 12
        elif reg == 0x0C:
            self.chr_base[2] = (v & 15) << 12
            self.chr_base[3] = (v >> 4) << 12
        elif 0x0D <= reg <= 0x14:
            n = (reg - 0x0D) >> 1
            if (reg - 0x0D) & 1 == 0:      # HOFS
                self.hofs[n] = ((v << 8) | (self.bgofs_latch & ~7) | (self.bghofs_latch & 7)) & 0x3FF
                self.bgofs_latch = v
                self.bghofs_latch = v
                if n == 0:
                    self.m7_hofs = sext13((v << 8) | self.mode7_latch)
                    self.mode7_latch = v
            else:                            # VOFS
                self.vofs[n] = ((v << 8) | self.bgofs_latch) & 0x3FF
                self.bgofs_latch = v
                if n == 0:
                    self.m7_vofs = sext13((v << 8) | self.mode7_latch)
                    self.mode7_latch = v
        elif reg == 0x15:
            self.vmain = v
        elif reg == 0x16:
            self.vmadd = (self.vmadd & 0xFF00) | v
            self.vram_latch = self.vram[self.remap(self.vmadd) & 0x7FFF]
        elif reg == 0x17:
            self.vmadd = (self.vmadd & 0x00FF) | ((v & 0x7F) << 8)
            self.vram_latch = self.vram[self.remap(self.vmadd) & 0x7FFF]
        elif reg == 0x18 or reg == 0x19:
            ra = self.remap(self.vmadd) & 0x7FFF
            if reg == 0x18:
                if self.vram_ok():
                    self.vram[ra] = (self.vram[ra] & 0xFF00) | v
                if not (self.vmain & 0x80):
                    self.vmadd = (self.vmadd + self.vstep()) & 0x7FFF
            else:
                if self.vram_ok():
                    self.vram[ra] = (self.vram[ra] & 0x00FF) | (v << 8)
                if self.vmain & 0x80:
                    self.vmadd = (self.vmadd + self.vstep()) & 0x7FFF
        elif reg == 0x1A:
            self.m7sel = v
        elif 0x1B <= reg <= 0x1E:
            self.m7[reg - 0x1B] = sext16((v << 8) | self.mode7_latch)
            self.mode7_latch = v
        elif reg == 0x1F:
            self.m7_cx = sext13(((v & 0x1F) << 8) | self.mode7_latch)
            self.mode7_latch = v
        elif reg == 0x20:
            self.m7_cy = sext13(((v & 0x1F) << 8) | self.mode7_latch)
            self.mode7_latch = v
        elif reg == 0x21:
            self.cgram_addr = v
            self.cgram_toggle = 0
        elif reg == 0x22:
            if self.cgram_toggle == 0:
                self.cgram_latch = v
            else:
                self.cgram[self.cgram_addr] = ((v & 0x7F) << 8) | self.cgram_latch
                self.cgram_addr = (self.cgram_addr + 1) & 0xFF
            self.cgram_toggle ^= 1
        elif 0x23 <= reg <= 0x25:
            self.win_sel[reg - 0x23] = v
        elif 0x26 <= reg <= 0x29:
            self.wh[reg - 0x26] = v
        elif reg == 0x2A:
            self.win_log_bg = v
        elif reg == 0x2B:
            self.win_log_obj = v & 3
            self.win_log_col = (v >> 2) & 3
        elif reg == 0x2C:
            self.tm = v & 0x1F
        elif reg == 0x2D:
            self.ts = v & 0x1F
        elif reg == 0x2E:
            self.tmw = v & 0x1F
        elif reg == 0x2F:
            self.tsw = v & 0x1F
        elif reg == 0x30:
            self.cgwsel = v
        elif reg == 0x31:
            self.cgadsub = v
        elif reg == 0x32:
            c = v & 0x1F
            if v & 0x20:
                self.fixed[0] = c
            if v & 0x40:
                self.fixed[1] = c
            if v & 0x80:
                self.fixed[2] = c
        elif reg == 0x33:
            self.setini = v
        # other registers: no effect on the picture

    # ------------------------------------------------------------------ small accessors
    @property
    def direct_color(self):
        return bool(self.cgwsel & 1)

    @property
    def addend_sub(self):
        return bool(self.cgwsel & 2)

    @property
    def extbg(self):
        return bool(self.setini & 0x40)

    @property
    def pseudo_hires(self):
        return bool(self.setini & 0x08)

    def fixed_color(self):
        return self.fixed[0] | (self.fixed[1] << 5) | (self.fixed[2] << 10)

    def hires_line(self):
        return self.bg_mode in (5, 6) or self.pseudo_hires

    # ------------------------------------------------------------------ tiles and maps (05b sections 3-5)
    def decode_row(self, row_word, bpp):
        vr = self.vram
        m0 = vr[row_word & 0x7FFF]
        planes = [m0 & 0xFF, m0 >> 8]
        if bpp >= 4:
            m1 = vr[(row_word + 8) & 0x7FFF]
            planes += [m1 & 0xFF, m1 >> 8]
        if bpp == 8:
            m2 = vr[(row_word + 16) & 0x7FFF]
            m3 = vr[(row_word + 24) & 0x7FFF]
            planes += [m2 & 0xFF, m2 >> 8, m3 & 0xFF, m3 >> 8]
        out = []
        for px in range(8):
            i = 0
            for p, pl in enumerate(planes):
                i |= ((pl >> (7 - px)) & 1) << p
            out.append(i)
        return out

    def map_entry_addr(self, n, tx, ty):
        x2, y2 = self.map_x2[n], self.map_y2[n]
        tx &= 63 if x2 else 31
        ty &= 63 if y2 else 31
        scr = ((tx >> 5) & x2) + ((ty >> 5) & y2) * (2 if x2 else 1)
        return (self.map_base[n] + scr * 0x400 + (ty & 31) * 32 + (tx & 31)) & 0x7FFF

    def palette_index(self, mode, n, bpp, pal, i):
        if bpp == 2:
            return (n * 32 + pal * 4 + i) if mode == 0 else (pal * 4 + i)
        if bpp == 4:
            return pal * 16 + i
        return i

    def mosaic(self, n, x, v):
        size = self.mosaic_size + 1
        if (self.mosaic_en >> n) & 1 and size > 1:
            return x - (x % size), v - ((v - 1) % size)
        return x, v

    def opt_offsets(self, n, xm):
        mode = self.bg_mode
        hu, vu = self.hofs[n], self.vofs[n]
        if mode in (2, 4) and n <= 1:
            n_col = (xm + (self.hofs[n] & 7)) >> 3
            if n_col >= 1:
                ex = (n_col - 1) + (self.hofs[2] >> 3)
                ey = self.vofs[2] >> 3

                def e(tx, ty):
                    return self.vram[self.map_entry_addr(2, tx, ty)]
                if mode == 2:
                    eh, ev = e(ex, ey), e(ex, ey + 1)
                else:
                    e0 = e(ex, ey)
                    if e0 & 0x8000 == 0:
                        eh, ev = e0, 0
                    else:
                        eh, ev = 0, e0
                bit = 0x2000 if n == 0 else 0x4000
                if eh & bit:
                    hu = (eh & 0x3F8) | (self.hofs[n] & 7)
                if ev & bit:
                    vu = ev & 0x3FF
        return hu, vu

    def bg_pixel(self, n, x, v):
        mode = self.bg_mode
        if mode >= 5:
            return None
        bpp = BPP[mode][n]
        if bpp == 0:
            return None
        xm, vm = self.mosaic(n, x, v)
        hu, vu = self.opt_offsets(n, xm)
        px = (xm + hu) & 0x3FF
        py = (vm + vu) & 0x3FF
        if self.tile16[n]:
            tx, ty = px >> 4, py >> 4
        else:
            tx, ty = px >> 3, py >> 3
        ent = self.vram[self.map_entry_addr(n, tx, ty)]
        tile = ent & 0x3FF
        pal = (ent >> 10) & 7
        prio = (ent >> 13) & 1
        hf = (ent >> 14) & 1
        vf = (ent >> 15) & 1
        if self.tile16[n]:
            xs = ((px >> 3) & 1) ^ hf
            ys = ((py >> 3) & 1) ^ vf
            tile = (tile + xs + ys * 16) & 0x3FF
        fx = (px & 7) ^ (7 if hf else 0)
        fy = (py & 7) ^ (7 if vf else 0)
        words = {2: 8, 4: 16, 8: 32}[bpp]
        tword = (self.chr_base[n] + tile * words) & 0x7FFF
        i = self.decode_row((tword + fy) & 0x7FFF, bpp)[fx]
        if i == 0:
            return None
        if self.direct_color and n == 0 and bpp == 8 and mode in (3, 4):
            r5 = ((i & 7) << 2) | ((pal & 1) << 1)
            g5 = (((i >> 3) & 7) << 2) | (pal & 2)
            b5 = (((i >> 6) & 3) << 3) | (pal & 4)
            rgb = r5 | (g5 << 5) | (b5 << 10)
        else:
            rgb = self.cgram[self.palette_index(mode, n, bpp, pal, i)]
        return (rgb, prio, 0)

    def bg_pixel_hires(self, n, hx, v):
        mode = self.bg_mode
        if mode not in (5, 6):
            return None
        bpp = BPP[mode][n]
        if bpp == 0:
            return None
        x = hx >> 1
        xm, vm = self.mosaic(n, x, v)
        hxm = (xm << 1) | (hx & 1)
        px = (hxm + (self.hofs[n] << 1)) & 0x3FF
        py = (vm + self.vofs[n]) & 0x3FF
        tall = self.tile16[n]
        tx = px >> 4
        ty = (py >> 4) if tall else (py >> 3)
        ent = self.vram[self.map_entry_addr(n, tx, ty)]
        hf = (ent >> 14) & 1
        vf = (ent >> 15) & 1
        xs = ((px >> 3) & 1) ^ hf
        ys = (((py >> 3) & 1) ^ vf) if tall else 0
        number = ((ent & 0x3FF) + xs + ys * 16) & 0x3FF
        fx = (px & 7) ^ (7 if hf else 0)
        fy = (py & 7) ^ (7 if vf else 0)
        words = 16 if bpp == 4 else 8
        tword = (self.chr_base[n] + number * words) & 0x7FFF
        i = self.decode_row((tword + fy) & 0x7FFF, bpp)[fx]
        if i == 0:
            return None
        rgb = self.cgram[self.palette_index(mode, n, bpp, (ent >> 10) & 7, i)]
        return (rgb, (ent >> 13) & 1, 0)

    # ------------------------------------------------------------------ mode 7 (05b section 8)
    def m7_sample(self, x, v):
        sx = 255 - x if self.m7sel & 1 else x
        sy = 255 - v if self.m7sel & 2 else v
        a, b, c, d = self.m7

        def clip(n):
            return (n | ~0x3FF) if n & 0x2000 else (n & 0x3FF)
        dh = clip(self.m7_hofs - self.m7_cx)
        dv = clip(self.m7_vofs - self.m7_cy)
        ox = ((a * dh) & ~63) + ((b * dv) & ~63) + ((b * sy) & ~63) + (self.m7_cx << 8)
        oy = ((c * dh) & ~63) + ((d * dv) & ~63) + ((d * sy) & ~63) + (self.m7_cy << 8)
        return (ox + a * sx) >> 8, (oy + c * sx) >> 8

    def m7_pixel(self, n, x, v):
        if self.bg_mode != 7 or n > 1 or (n == 1 and not self.extbg):
            return None
        xm, vm = self.mosaic(n, x, v)
        px, py = self.m7_sample(xm, vm)
        outside = bool(self.m7sel & 0x80)
        fill = bool(self.m7sel & 0x40)
        if (px | py) & ~0x3FF != 0:
            if not outside:
                px &= 0x3FF
                py &= 0x3FF
                tile = self.vram[(((py >> 3) << 7) + (px >> 3)) & 0x7FFF] & 0xFF
            elif not fill:
                return None
            else:
                tile = 0
        else:
            tile = self.vram[(((py >> 3) << 7) + (px >> 3)) & 0x7FFF] & 0xFF
        fx, fy = px & 7, py & 7
        idx = (self.vram[(tile * 64 + fy * 8 + fx) & 0x7FFF] >> 8) & 0xFF
        if idx == 0:
            return None
        if n == 0:
            if self.direct_color:
                rgb = ((idx & 7) << 2) | ((((idx >> 3) & 7) << 2) << 5) | ((idx >> 6) << 3 << 10)
            else:
                rgb = self.cgram[idx]
            return (rgb, 0, 0)
        i7 = idx & 0x7F
        if i7 == 0:
            return None
        return (self.cgram[i7], idx >> 7, 0)

    # ------------------------------------------------------------------ sprites (05b section 6)
    def eval_sprites(self, r):
        oam = self.oam
        first = ((self.internal_oamadd >> 2) & 0x7F) if self.obj_rot else 0
        sizes = OBJ_SIZES[self.obj_size]
        info = []
        for i in range(128):
            hi = oam[512 + i // 4] >> ((i % 4) * 2)
            x = oam[i * 4] | ((hi & 1) << 8)
            if x >= 256:
                x -= 512
            large = (hi >> 1) & 1
            w, h = sizes[large]
            info.append((x, oam[i * 4 + 1], oam[i * 4 + 2], oam[i * 4 + 3], w, h))
        lst = []
        for k in range(128):
            i = (first + k) & 0x7F
            x, y, t, attr, w, h = info[i]
            row = (r - y) & 0xFF
            if row >= h:
                continue
            if not (x == -256 or x > -w):
                continue
            if len(lst) == 32:
                break
            lst.append(i)
        line = [None] * 256
        slivers = 0
        done = False
        for i in reversed(lst):
            x, y, t, attr, w, h = info[i]
            row = (r - y) & 0xFF
            hflip = (attr >> 6) & 1
            vflip = (attr >> 7) & 1
            pal = (attr >> 1) & 7
            prio = (attr >> 4) & 3
            tbit = attr & 1
            for c in range(w // 8):
                sx = x + c * 8
                if not ((sx > -8 and sx < 256) or x == -256):
                    continue
                if slivers == 34:
                    done = True
                    break
                slivers += 1
                tc = (w // 8 - 1 - c) if hflip else c
                if vflip:
                    if w == h:
                        rv = h - 1 - row
                    else:
                        half = h // 2
                        rv = (row // half) * half + (half - 1 - row % half)
                else:
                    rv = row
                tile_row = rv >> 3
                fine_y = rv & 7
                tx = ((t & 15) + tc) & 15
                ty = ((t >> 4) + tile_row) & 15
                tile8 = ty * 16 + tx
                addr = (self.obj_base + (self.obj_name_gap if tbit else 0) + tile8 * 16 + fine_y) & 0x7FFF
                lo = self.vram[addr]
                hi_w = self.vram[(addr + 8) & 0x7FFF]
                for px in range(8):
                    bit = px if hflip else 7 - px
                    idx = ((lo >> bit) & 1) | (((lo >> (8 + bit)) & 1) << 1) | (((hi_w >> bit) & 1) << 2) | (((hi_w >> (8 + bit)) & 1) << 3)
                    dest = sx + px
                    if idx != 0 and 0 <= dest < 256:
                        line[dest] = (self.cgram[128 + pal * 16 + idx], prio, pal)
            if done:
                break
        self.line_obj = line

    # ------------------------------------------------------------------ windows (05b section 10)
    def win_group(self, layer):
        ws = self.win_sel
        if layer == 0:
            return ws[0] & 15, self.win_log_bg & 3
        if layer == 1:
            return ws[0] >> 4, (self.win_log_bg >> 2) & 3
        if layer == 2:
            return ws[1] & 15, (self.win_log_bg >> 4) & 3
        if layer == 3:
            return ws[1] >> 4, (self.win_log_bg >> 6) & 3
        if layer == 4:
            return ws[2] & 15, self.win_log_obj
        return ws[2] >> 4, self.win_log_col

    def window_out(self, layer, x):
        g, logic = self.win_group(layer)
        wh = self.wh
        w1 = (wh[0] <= x <= wh[1]) != bool(g & 1)
        w2 = (wh[2] <= x <= wh[3]) != bool(g & 4)
        en1, en2 = bool(g & 2), bool(g & 8)
        if not en1 and not en2:
            return False
        if en1 and not en2:
            return w1
        if en2 and not en1:
            return w2
        if logic == 0:
            return w1 or w2
        if logic == 1:
            return w1 and w2
        if logic == 2:
            return w1 != w2
        return w1 == w2

    # ------------------------------------------------------------------ composition (05b section 11)
    def order(self):
        m = self.bg_mode
        if m == 0:
            return [(4, 3), (0, 1), (1, 1), (4, 2), (0, 0), (1, 0), (4, 1), (2, 1), (3, 1), (4, 0), (2, 0), (3, 0)]
        if m == 1:
            if self.bg3_prio:
                return [(2, 1), (4, 3), (0, 1), (1, 1), (4, 2), (0, 0), (1, 0), (4, 1), (4, 0), (2, 0)]
            return [(4, 3), (0, 1), (1, 1), (4, 2), (0, 0), (1, 0), (4, 1), (2, 1), (4, 0), (2, 0)]
        if m in (2, 3, 4, 5):
            return [(4, 3), (0, 1), (4, 2), (1, 1), (4, 1), (0, 0), (4, 0), (1, 0)]
        if m == 6:
            return [(4, 3), (0, 1), (4, 2), (4, 1), (0, 0), (4, 0)]
        if self.extbg:
            return [(4, 3), (4, 2), (1, 1), (4, 1), (0, 0), (4, 0), (1, 0)]
        return [(4, 3), (4, 2), (4, 1), (0, 0), (4, 0)]

    def sample_layers(self, x, hx, v):
        lp = [None] * 5
        en = self.tm | self.ts
        mode = self.bg_mode
        for n in range(4):
            if not (en >> n) & 1:
                continue
            if mode == 7:
                lp[n] = self.m7_pixel(n, x, v)
            elif mode in (5, 6):
                lp[n] = self.bg_pixel_hires(n, hx, v)
            else:
                lp[n] = self.bg_pixel(n, x, v)
        if (en >> 4) & 1:
            lp[4] = self.line_obj[x]
        return lp

    def pick(self, main, x, lp):
        en = self.tm if main else self.ts
        wm = self.tmw if main else self.tsw
        for layer, level in self.order():
            if not (en >> layer) & 1:
                continue
            if (wm >> layer) & 1 and self.window_out(layer, x):
                continue
            p = lp[layer]
            if p is not None and p[1] == level:
                return (p[0], layer, p[2])
        if main:
            return (self.cgram[0], BACKDROP, 0)
        return (self.fixed_color(), BACKDROP, 0)

    def color_flags(self, x):
        inside = self.window_out(5, x)

        def region(t, ins):
            return (False, not ins, ins, True)[t]
        black = region((self.cgwsel >> 6) & 3, inside)
        prevent = region((self.cgwsel >> 4) & 3, inside)
        return black, prevent

    def layer_ok(self, layer, obj_pal):
        if layer <= 3:
            return bool((self.cgadsub >> layer) & 1)
        if layer == 4:
            return bool((self.cgadsub >> 4) & 1) and obj_pal >= 4
        return bool((self.cgadsub >> 5) & 1)

    def blend(self, base, add, halve):
        sub = bool(self.cgadsub & 0x80)
        out = 0
        for sh in (0, 5, 10):
            b = (base >> sh) & 31
            a = (add >> sh) & 31
            t = (b - a) if sub else (b + a)
            if halve:
                t >>= 1
            out |= max(0, min(31, t)) << sh
        return out

    def color_math(self, main, sub, black, prevent):
        base = 0 if black else main[0]
        if self.layer_ok(main[1], main[2]) and not prevent:
            if self.addend_sub:
                add, sub_back = sub[0], sub[1] == BACKDROP
            else:
                add, sub_back = self.fixed_color(), False
            halve = bool(self.cgadsub & 0x40) and not (self.addend_sub and sub_back)
            return self.blend(base, add, halve)
        return base

    def compose(self, x, v):
        """returns a list of BGR555 colours: [col] (normal line) or [col 2x, col 2x+1] (hi-res line)"""
        black, prevent = self.color_flags(x)
        if not self.hires_line():
            lp = self.sample_layers(x, 2 * x + 1, v)
            main = self.pick(True, x, lp)
            if self.addend_sub:
                sub = self.pick(False, x, lp)
            else:
                sub = (self.fixed_color(), BACKDROP, 0)
            return [self.color_math(main, sub, black, prevent)]
        lm = self.sample_layers(x, 2 * x + 1, v)
        ls = self.sample_layers(x, 2 * x, v)
        main = self.pick(True, x, lm)
        sub = self.pick(False, x, ls)
        odd = self.color_math(main, sub, black, prevent)
        base = 0 if black else sub[0]
        if (not self.addend_sub) and (not prevent) and self.layer_ok(sub[1], sub[2]):
            even = self.blend(base, self.fixed_color(), bool(self.cgadsub & 0x40))
        else:
            even = base
        return [even, odd]

    def xrgb(self, c5):
        if self.forced_blank:
            return 0xFF000000
        out = 0xFF000000
        for k, sh in enumerate((0, 5, 10)):
            ch = (c5 >> sh) & 31
            cb = ch * self.brightness // 15
            c8 = (cb << 3) | (cb >> 2)
            out |= c8 << (16 - 8 * k)
        return out


# =====================================================================================================
#  Scene file: parse + run
# =====================================================================================================

def parse_scenes(text):
    scenes = []
    cur = None
    for ln, raw in enumerate(text.splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        t = line.split()
        d = t[0]
        try:
            if d == "scene":
                cur = {"name": t[1], "frames": int(t[2]), "ops": []}
            elif d == "end":
                scenes.append(cur)
                cur = None
            elif d == "vram":
                cur["ops"].append(("vram", int(t[1], 16), [int(w, 16) for w in t[2:]]))
            elif d == "cgram":
                cur["ops"].append(("cgram", int(t[1]), [int(w, 16) for w in t[2:]]))
            elif d == "oam":
                cur["ops"].append(("oam", int(t[1]), [int(b, 16) for b in t[2:]]))
            elif d == "reg":
                cur["ops"].append(("reg", int(t[1], 16), int(t[2], 16)))
            elif d == "at":
                cur["ops"].append(("at", int(t[1]), int(t[2], 16), int(t[3], 16)))
            else:
                raise ValueError("unknown directive " + d)
        except (IndexError, ValueError, TypeError) as e:
            raise SystemExit("scene file line %d: %s" % (ln, e))
    return scenes


def simulate(scene, log=None):
    """returns (width, height, pixels) exactly as Ppu::frame() after the harness of task E05-19"""
    p = RefPpu()
    at = {}
    for op in scene["ops"]:
        if op[0] == "vram":
            addr, words = op[1], op[2]
            p.write(0x15, 0x80)
            p.write(0x16, addr & 0xFF)
            p.write(0x17, addr >> 8)
            for w in words:
                p.write(0x18, w & 0xFF)
                p.write(0x19, w >> 8)
        elif op[0] == "cgram":
            p.write(0x21, op[1])
            for w in op[2]:
                p.write(0x22, w & 0xFF)
                p.write(0x22, w >> 8)
        elif op[0] == "oam":
            a = op[1]
            assert a % 2 == 0 and len(op[2]) % 2 == 0, "oam address and length must be even"
            p.write(0x02, (a // 2) & 0xFF)
            p.write(0x03, ((a // 2) >> 8) & 1)
            for b in op[2]:
                p.write(0x04, b)
        elif op[0] == "reg":
            p.write(op[1], op[2])
        elif op[0] == "at":
            at.setdefault(op[1], []).append((op[2], op[3]))
    frames = scene["frames"]
    rows = None
    row_hires = None
    for f in range(frames):
        rows = [None] * 224
        row_hires = [False] * 224
        for line in range(262):
            p.v = line
            for reg, val in at.get(line, []):
                p.write(reg, val)
            if line == 225 and f == frames - 1:
                break
            if 1 <= line <= 224:
                p.eval_sprites(line - 1)
                hl = p.hires_line()
                row_hires[line - 1] = hl
                cols = []
                for x in range(256):
                    cs = p.compose(x, line)
                    for c in cs:
                        cols.append(p.xrgb(c))
                rows[line - 1] = cols
    hires = any(row_hires)
    w = 512 if hires else 256
    pixels = []
    for r in range(224):
        cols = rows[r]
        if hires and not row_hires[r]:
            cols = [c for c in cols for _ in (0, 1)]
        pixels.extend(cols)
    return w, 224, pixels


def write_ppm(path, w, h, pixels):
    with open(path, "wb") as f:
        f.write(("P6\n%d %d\n255\n" % (w, h)).encode("ascii"))
        f.write(bytes(b for px in pixels for b in ((px >> 16) & 255, (px >> 8) & 255, px & 255)))


# =====================================================================================================
#  Self test (hand-checkable cases; spec 05b section 15)
# =====================================================================================================

def selftest():
    n = [0]

    def ok(cond, msg):
        n[0] += 1
        if not cond:
            raise SystemExit("SELFTEST FAIL: " + msg)

    def put_vram(p, addr, w):
        p.write(0x15, 0x80); p.write(0x16, addr & 255); p.write(0x17, addr >> 8); p.write(0x18, w & 255); p.write(0x19, w >> 8)

    def put_cgram(p, i, w):
        p.write(0x21, i); p.write(0x22, w & 255); p.write(0x22, w >> 8)

    # color conversion
    p = RefPpu(); p.forced_blank = False; p.brightness = 15
    ok(p.xrgb(0x001F) == 0xFFFF0000, "red 31 -> 0xFF0000")
    p.brightness = 0
    ok(p.xrgb(0x7FFF) == 0xFF000000, "brightness 0 -> black")
    p.brightness = 15
    ok(p.xrgb(1) == 0xFF080000, "c5=1 -> 8")
    p.forced_blank = True
    ok(p.xrgb(0x7FFF) == 0xFF000000, "forced blank")

    # scroll latches: BG1HOFS written 0x12 then 0x03 -> 0x312 & 0x3FF
    p = RefPpu(); p.write(0x0D, 0x12); p.write(0x0D, 0x03)
    ok(p.hofs[0] == 0x312, "hofs 0x312 got %x" % p.hofs[0])
    ok(p.m7_hofs == sext13((3 << 8) | 0x12), "m7 hofs shares the write")

    # mode 0, BG1: tile 1, row 0 pixel 0 = colour index 1 -> red.  VOFS = -1 -> output row 0 shows map row 0
    p = RefPpu()
    put_vram(p, 8, 0x0001)                 # tile 1 (2bpp = 8 words), row 0: plane0 = 0x01 -> pixel 7 = 1
    put_vram(p, 0x6000, 1)                 # map entry (0,0) = tile 1
    put_cgram(p, 1, 0x001F)
    p.write(0x07, 0x60); p.write(0x0E, 0xFF); p.write(0x0E, 0x03); p.write(0x2C, 1); p.write(0x00, 0x0F)
    p.v = 1
    cs = p.compose(7, 1)
    ok(p.xrgb(cs[0]) == 0xFFFF0000, "mode 0 pixel (7,row 0) red")
    cs = p.compose(6, 1)
    ok(p.xrgb(cs[0]) == 0xFF000000, "pixel 6 transparent -> backdrop (cgram[0] = 0 -> black)")

    # sprites: 8x8 at Y=0 visible at row 0 ; Y=1 not visible at row 0 (but at row 1)  [E09-00: row = (r - y) & 0xFF]
    p = RefPpu()
    for a in range(8):
        put_vram(p, 0x4000 + a, 0x00FF)    # obj tile 0 rows: plane0 = 0xFF -> index 1 everywhere
    put_cgram(p, 129, 0x03E0)              # sprite palette 0 index 1
    p.write(0x01, 0x02); p.write(0x2C, 0x10); p.write(0x00, 0x0F)
    for i in range(1, 128):                    # hide sprites 1..127: x = -64 (low 0xC0, bit 8 set)
        p.oam[i * 4:i * 4 + 4] = [0xC0, 0xF0, 0, 0]
    p.oam[0:4] = [10, 0, 0, 0]
    for k in range(32):
        p.oam[512 + k] = 0x55                  # every sprite: x bit 8 = 1, small
    p.oam[512] = 0x54                          # sprite 0: x bit 8 = 0
    p.eval_sprites(0)
    ok(p.line_obj[10] is not None and p.line_obj[17] is not None and p.line_obj[18] is None, "Y=0 visible at row 0")
    p.oam[0:4] = [10, 1, 0, 0]
    p.eval_sprites(0)
    ok(all(q is None for q in p.line_obj), "Y=1 not visible at row 0")
    p.eval_sprites(1)
    ok(p.line_obj[10] is not None, "Y=1 visible at row 1")

    # windows: left > right = empty ; inversion ; logic truth table
    p = RefPpu()
    p.win_sel[0] = 0x02; p.wh = [10, 5, 0, 0]
    ok(not any(p.window_out(0, x) for x in range(256)), "left > right: empty window")
    p.win_sel[0] = 0x03
    ok(all(p.window_out(0, x) for x in range(256)), "inverted empty window = everything")
    p.win_sel[0] = 0x0A; p.wh = [0, 9, 5, 14]
    for logic, f in enumerate((lambda a, b: a or b, lambda a, b: a and b, lambda a, b: a != b, lambda a, b: a == b)):
        p.win_log_bg = logic
        for x in (2, 7, 12, 20):
            ok(p.window_out(0, x) == f(0 <= x <= 9, 5 <= x <= 14), "window logic %d x=%d" % (logic, x))

    # colour math: 20 + 20 clamps to 31 ; sub ; half
    p = RefPpu(); p.forced_blank = False; p.brightness = 15
    put_cgram(p, 0, 20)
    p.write(0x32, 0x20 | 20); p.write(0x31, 0x20)
    ok(p.compose(0, 1)[0] == 31, "add clamp")
    p.write(0x31, 0x20 | 0x40)
    ok(p.compose(0, 1)[0] == 20, "add half (20+20)>>1")
    p.write(0x31, 0x20 | 0x80)
    ok(p.compose(0, 1)[0] == 0, "sub clamp to 0")
    # backdrop of the subscreen is the fixed colour and there is no halving then
    p.write(0x30, 0x02); p.write(0x2D, 0); p.write(0x31, 0x20 | 0x40)
    p.write(0x31, 0x20)
    ok(p.compose(0, 1)[0] == 31, "subscreen backdrop = fixed: add")
    p.write(0x31, 0x20 | 0x40)
    ok(p.compose(0, 1)[0] == 31, "no halve when subscreen is the backdrop")

    # mode 7 identity: pixel (x, v) samples map position (x, v)
    p = RefPpu()
    p.write(0x1B, 0x00); p.write(0x1B, 0x01); p.write(0x1E, 0x00); p.write(0x1E, 0x01)
    ok(p.m7 == [256, 0, 0, 256], "m7 matrix")
    ok(p.m7_sample(37, 101) == (37, 101), "identity m7_sample")
    p.write(0x1A, 0x03)
    ok(p.m7_sample(37, 101) == (255 - 37, 255 - 101), "flips")
    print("selftest OK (%d checks)" % n[0])
    return 0




# =====================================================================================================
#  Scene construction (deterministic: own LCG, integer arithmetic only)
# =====================================================================================================

class Rng:
    def __init__(self, seed):
        self.x = (seed * 2654435761 + 12345) & 0x7FFFFFFF

    def next(self):
        self.x = (self.x * 1103515245 + 12345) & 0x7FFFFFFF
        return (self.x >> 8) & 0xFFFFFF

    def int(self, n):
        return self.next() % n


def pal_color(i, seed):
    r = (i * 7 + seed * 3 + (i >> 2) * 5) & 31
    g = (i * 11 + seed * 5 + (i >> 3) * 3 + 7) & 31
    b = (i * 13 + seed * 7 + (i >> 4) * 9 + 3) & 31
    return r | (g << 5) | (b << 10)


GLYPH_F = ["########", "#.......", "#.......", "######..", "#.......", "#.......", "#.......", "#......."]


def tile_pixels(kind, c1, c2, maxc, rng):
    pix = []
    for py in range(8):
        for px in range(8):
            if kind == 0:
                i = c1 if GLYPH_F[py][px] == "#" else 0
            elif kind == 1:
                i = c1 if (((px >> 1) + (py >> 1)) & 1) else c2
            elif kind == 2:
                q = (px + py) & 3
                i = c1 if q == 0 else (c2 if q == 2 else 0)
            elif kind == 3:
                if px in (0, 7) or py in (0, 7):
                    i = c1
                else:
                    i = c2 if px == py else 0
            elif kind == 4:
                i = 1 + ((px + 2 * py) % maxc)
            elif kind == 5:
                i = rng.int(maxc + 1)
            elif kind == 6:
                i = c1
            else:
                i = c1 if py < 4 else (c2 if px < 4 else 0)
            pix.append(i)
    return pix


def encode_tile(pix, bpp):
    nb = bpp // 2
    out = [0] * (8 * nb)
    for fy in range(8):
        for pp in range(nb):
            lo = hi = 0
            for px in range(8):
                i = pix[fy * 8 + px]
                lo |= ((i >> (2 * pp)) & 1) << (7 - px)
                hi |= ((i >> (2 * pp + 1)) & 1) << (7 - px)
            out[pp * 8 + fy] = lo | (hi << 8)
    return out


class Scene:
    def __init__(self, name, frames=1, seed=1):
        self.name, self.frames, self.seed = name, frames, seed
        self.rng = Rng(seed)
        self.vram = {}
        self.cgram = [pal_color(i, seed) for i in range(256)]
        self.oam = [0] * 544
        for i in range(128):
            self.oam[i * 4:i * 4 + 4] = [0xC0, 0xF0, 0, 0]
        for k in range(32):
            self.oam[512 + k] = 0x55
        self.regs = []
        self.ats = []

    # memory helpers
    def put(self, addr, words):
        for k, w in enumerate(words):
            self.vram[(addr + k) & 0x7FFF] = w & 0xFFFF

    def reg(self, r, v):
        self.regs.append((r, v & 0xFF))

    def w16(self, r, val):
        self.reg(r, val & 0xFF)
        self.reg(r, (val >> 8) & 0xFF)

    def at(self, line, r, v):
        self.ats.append((line, r, v & 0xFF))

    def backdrop(self, c):
        self.cgram[0] = c

    # backgrounds
    def tileset(self, addr, bpp, n, seed_off=0, opaque=False):
        maxc = (1 << bpp) - 1
        r = Rng(self.seed * 31 + seed_off + bpp)
        for k in range(n):
            kind = (k + seed_off) % 8
            c1 = 1 + (k * 3 + seed_off) % maxc
            c2 = 1 + (k * 5 + 1 + seed_off) % maxc
            if opaque and kind in (0, 2, 3, 5, 7):
                kind = 4
            self.put(addr + k * 4 * bpp, encode_tile(tile_pixels(kind, c1, c2, maxc, r), bpp))

    def tilemap(self, base, x2, y2, tw, th, max_tile, pal_max=8, seed_off=0, flips=True, prio_bits=True, tile_step=1):
        r = Rng(self.seed * 17 + seed_off + base)
        for ty in range(th):
            for tx in range(tw):
                t = (tx * 5 + ty * 3 + r.int(4)) % max_tile
                ent = t | ((r.int(pal_max)) << 10)
                if prio_bits and r.int(3) == 0:
                    ent |= 1 << 13
                if flips:
                    if r.int(4) == 0:
                        ent |= 1 << 14
                    if r.int(4) == 0:
                        ent |= 1 << 15
                a = map_addr(base, x2, y2, tx, ty)
                self.vram[a] = ent

    def bg(self, n, bpp, ntiles, chr_base, map_base, x2=0, y2=0, tile16=False, seed_off=0, pal_max=8, opaque=False):
        self.tileset(chr_base, bpp, ntiles, seed_off + n, opaque)
        tw = 64 if x2 else 32
        th = 64 if y2 else 32
        if tile16:
            tw >>= 0
        mt = ntiles - 18 if tile16 else ntiles
        self.tilemap(map_base, x2, y2, tw, th, mt, pal_max, seed_off + n)
        self.reg(0x07 + n, ((map_base >> 10) << 2) | x2 | (y2 << 1))

    def chr_regs(self, c1, c2, c3=0, c4=0):
        self.reg(0x0B, (c1 >> 12) | ((c2 >> 12) << 4))
        self.reg(0x0C, (c3 >> 12) | ((c4 >> 12) << 4))

    def scroll(self, n, hofs, vofs):
        self.w16(0x0D + 2 * n, hofs)
        self.w16(0x0E + 2 * n, vofs)

    # sprites
    def obj_tiles(self, base, rows=8, seed_off=0):
        r = Rng(self.seed * 13 + seed_off)
        for ty in range(rows):
            for tx in range(16):
                k = tx + ty * 16
                kind = (tx + 2 * ty + seed_off) % 8
                c1 = 1 + (k * 3 + seed_off) % 15
                c2 = 1 + (k * 5 + 4 + seed_off) % 15
                self.put(base + k * 16, encode_tile(tile_pixels(kind, c1, c2, 15, r), 4))

    def sprite(self, i, x, y, tile, pal=0, prio=0, h=0, v=0, tbit=0, large=0):
        self.oam[i * 4] = x & 0xFF
        self.oam[i * 4 + 1] = y & 0xFF
        self.oam[i * 4 + 2] = tile & 0xFF
        self.oam[i * 4 + 3] = (v << 7) | (h << 6) | (prio << 4) | (pal << 1) | tbit
        shift = (i % 4) * 2
        hb = self.oam[512 + i // 4] & ~(3 << shift)
        hb |= (((x >> 8) & 1) | (large << 1)) << shift
        self.oam[512 + i // 4] = hb

    # mode 7
    def m7_data(self, ntiles=16, region=64, seed_off=0):
        r = Rng(self.seed * 7 + seed_off)
        tiles = []
        for t in range(ntiles):
            kind = t % 8
            c1 = 1 + (t * 29 + 7) % 255
            c2 = 1 + (t * 53 + 11) % 255
            tiles.append(tile_pixels(kind, c1, c2, 255, r))
        words = {}
        for i in range(0x4000):
            row, col = i >> 7, i & 127
            m = 0
            if row < region and col < region:
                m = ((col >> 2) + (row >> 2) * 3 + ((col >> 3) ^ (row >> 3))) % ntiles
            hi = tiles[i >> 6][i & 63] if (i >> 6) < ntiles else 0
            w = (hi << 8) | m
            if w:
                words[i] = w
        for a, w in words.items():
            self.vram[a] = w

    def m7(self, a, b, c, d, cx, cy, hofs, vofs):
        self.w16(0x1B, a)
        self.w16(0x1C, b)
        self.w16(0x1D, c)
        self.w16(0x1E, d)
        self.w16(0x1F, cx & 0x1FFF)
        self.w16(0x20, cy & 0x1FFF)
        self.w16(0x0D, hofs & 0x1FFF)
        self.w16(0x0E, vofs & 0x1FFF)

    def coldata(self, r, g, b):
        self.reg(0x32, 0x20 | r)
        self.reg(0x32, 0x40 | g)
        self.reg(0x32, 0x80 | b)

    # text
    def text(self):
        out = ["scene %s %d" % (self.name, self.frames)]
        addrs = sorted(self.vram)
        k = 0
        while k < len(addrs):
            start = addrs[k]
            run = [self.vram[start]]
            j = k + 1
            while j < len(addrs) and addrs[j] == addrs[j - 1] + 1 and len(run) < 16:
                run.append(self.vram[addrs[j]])
                j += 1
            out.append("vram %04x %s" % (start, " ".join("%04x" % w for w in run)))
            k = j
        for i in range(0, 256, 16):
            out.append("cgram %d %s" % (i, " ".join("%04x" % w for w in self.cgram[i:i + 16])))
        for i in range(0, 544, 16):
            out.append("oam %d %s" % (i, " ".join("%02x" % b for b in self.oam[i:i + 16])))
        for r, v in self.regs:
            out.append("reg %02x %02x" % (r, v))
        for line, r, v in self.ats:
            out.append("at %d %02x %02x" % (line, r, v))
        out.append("end")
        return "\n".join(out) + "\n"


def map_addr(base, x2, y2, tx, ty):
    tx &= 63 if x2 else 31
    ty &= 63 if y2 else 31
    scr = ((tx >> 5) & x2) + ((ty >> 5) & y2) * (2 if x2 else 1)
    return (base + scr * 0x400 + (ty & 31) * 32 + (tx & 31)) & 0x7FFF


def unblank(sc, brightness=15):
    sc.reg(0x00, brightness)


# ------------------------------------------------------------------------------------------ scenes

def scene_mode0():
    sc = Scene("mode0_four_layers", 1, 1)
    sc.backdrop(0x2C60)
    for n, (chr_b, map_b) in enumerate(((0x0000, 0x6000), (0x1000, 0x6400), (0x2000, 0x6800), (0x3000, 0x6C00))):
        sc.bg(n, 2, 20, chr_b, map_b)
    sc.reg(0x05, 0x00)
    sc.chr_regs(0x0000, 0x1000, 0x2000, 0x3000)
    sc.scroll(0, 3, 0x3FF)
    sc.scroll(1, 261, 7)
    sc.scroll(2, 80, 1000)
    sc.scroll(3, 511, 300)
    sc.reg(0x2C, 0x0F)
    unblank(sc, 11)
    return sc


def sprites_demo(sc, count=12, size_small=8, seed_off=0, tbit_mix=False, pal_lo=0):
    r = Rng(sc.seed * 5 + seed_off)
    for i in range(count):
        x = 12 + (i % 6) * 40 + r.int(12)
        y = 20 + (i // 6) * 60 + r.int(40)
        sc.sprite(i, x, y, (r.int(8) * 2) + (r.int(4) * 32), pal=pal_lo + r.int(8 - pal_lo), prio=i % 4,
                  h=r.int(2), v=r.int(2), tbit=(r.int(2) if tbit_mix else 0), large=r.int(2))


def scene_mode1_prio():
    sc = Scene("mode1_bg3prio_tile16", 1, 2)
    sc.backdrop(0x1084)
    sc.bg(0, 4, 40, 0x0000, 0x6000, x2=1, tile16=True)
    sc.bg(1, 4, 24, 0x1000, 0x6800)
    sc.bg(2, 2, 24, 0x2000, 0x6C00, y2=1)
    sc.obj_tiles(0x4000, 8, 3)
    sc.obj_tiles(0x5000, 8, 5)
    sc.reg(0x01, 0x02)
    sprites_demo(sc, 14, tbit_mix=True)
    sc.reg(0x05, 0x19)
    sc.chr_regs(0x0000, 0x1000, 0x2000, 0)
    sc.scroll(0, 37, 0x3FF)
    sc.scroll(1, 9, 0x3F0)
    sc.scroll(2, 100, 200)
    sc.reg(0x2C, 0x17)
    unblank(sc)
    return sc


def scene_mode1_mosaic():
    sc = Scene("mode1_mosaic", 1, 3)
    sc.backdrop(0x0C63)
    sc.bg(0, 4, 24, 0x0000, 0x6000)
    sc.bg(1, 4, 24, 0x1000, 0x6400)
    sc.bg(2, 2, 24, 0x2000, 0x6800)
    sc.obj_tiles(0x4000, 8, 7)
    sc.reg(0x01, 0x02)
    sprites_demo(sc, 10, seed_off=3)
    sc.reg(0x05, 0x01)
    sc.chr_regs(0x0000, 0x1000, 0x2000, 0)
    sc.scroll(0, 11, 0x3FF)
    sc.scroll(1, 50, 0x3FF)
    sc.scroll(2, 3, 5)
    sc.reg(0x06, 0x45)
    sc.reg(0x2C, 0x17)
    unblank(sc)
    return sc


def scene_mode2_opt():
    sc = Scene("mode2_offset_per_tile", 1, 4)
    sc.backdrop(0x1CE7)
    sc.bg(0, 4, 32, 0x0000, 0x6000, opaque=True)
    sc.bg(1, 4, 32, 0x1000, 0x6400, opaque=False)
    sc.tilemap(0x6800, 0, 0, 32, 32, 8)          # BG3 map region = OPT data (overwritten below)
    sc.reg(0x09, 0x68)
    r = Rng(41)
    for tx in range(32):
        for ty in range(2):
            val = 0
            if ty == 0:
                val = ((r.int(64) << 3) & 0x3F8)
            else:
                val = r.int(1024)
            val |= (r.int(2) << 13) | (r.int(2) << 14)
            sc.vram[map_addr(0x6800, 0, 0, tx, ty)] = val
    sc.reg(0x05, 0x02)
    sc.chr_regs(0x0000, 0x1000, 0, 0)
    sc.scroll(0, 5, 0x3FF)
    sc.scroll(1, 2, 0x3FF)
    sc.scroll(2, 8, 0)
    sc.reg(0x2C, 0x03)
    unblank(sc)
    return sc


def scene_mode4_opt():
    sc = Scene("mode4_offset_per_tile", 1, 5)
    sc.backdrop(0x0842)
    sc.bg(0, 8, 12, 0x0000, 0x6000, opaque=True)
    sc.bg(1, 2, 24, 0x2000, 0x6400)
    sc.tilemap(0x6800, 0, 0, 32, 32, 8)
    sc.reg(0x09, 0x68)
    r = Rng(43)
    for tx in range(32):
        e = (r.int(1024) & 0x3F8) | (r.int(2) << 13) | (r.int(2) << 14) | (r.int(2) << 15)
        sc.vram[map_addr(0x6800, 0, 0, tx, 2)] = e
    sc.reg(0x05, 0x04)
    sc.chr_regs(0x0000, 0x2000, 0, 0)
    sc.scroll(0, 6, 0x3FF)
    sc.scroll(1, 12, 0x3FF)
    sc.scroll(2, 16, 16)
    sc.reg(0x2C, 0x03)
    unblank(sc)
    return sc


def scene_mode3_direct():
    sc = Scene("mode3_direct_color", 1, 6)
    sc.backdrop(0x0000)
    sc.bg(0, 8, 14, 0x0000, 0x6000, opaque=False)
    sc.bg(1, 4, 24, 0x2000, 0x6400)
    sc.reg(0x05, 0x03)
    sc.chr_regs(0x0000, 0x2000, 0, 0)
    sc.scroll(0, 0, 0x3FF)
    sc.scroll(1, 21, 0x3FF)
    sc.reg(0x30, 0x01)
    sc.reg(0x2C, 0x03)
    unblank(sc)
    return sc


def scene_mode3_palette():
    sc = Scene("mode3_8bpp_palette_tile16", 1, 7)
    sc.backdrop(0x3DEF)
    sc.bg(0, 8, 40, 0x0000, 0x6000, x2=1, tile16=True, opaque=False)
    sc.bg(1, 4, 24, 0x4000, 0x6800, y2=1)
    sc.reg(0x05, 0x13)
    sc.chr_regs(0x0000, 0x4000, 0, 0)
    sc.scroll(0, 130, 0x3FF)
    sc.scroll(1, 7, 0x3FF)
    sc.reg(0x2C, 0x03)
    unblank(sc, 13)
    return sc


def hires_base(name, seed, math):
    sc = Scene(name, 1, seed)
    sc.backdrop(0x1D6B)
    sc.bg(0, 4, 40, 0x0000, 0x6000, x2=1, tile16=True)
    sc.bg(1, 2, 24, 0x2000, 0x6800)
    sc.obj_tiles(0x4000, 8, 9)
    sc.reg(0x01, 0x02)
    sprites_demo(sc, 8, seed_off=seed)
    sc.reg(0x05, 0x15)
    sc.chr_regs(0x0000, 0x2000, 0, 0)
    sc.scroll(0, 31, 0x3FF)
    sc.scroll(1, 5, 0x3FF)
    sc.coldata(5, 10, 3)
    sc.reg(0x2C, 0x13)
    sc.reg(0x2D, 0x03)
    if math:
        sc.reg(0x31, 0x63 | 0x10)
    unblank(sc)
    return sc


def scene_mode5():
    return hires_base("mode5_hires", 8, False)


def scene_mode5_math():
    return hires_base("mode5_hires_colormath", 9, True)


def scene_pseudo_hires():
    sc = Scene("pseudo_hires_mode1", 1, 10)
    sc.backdrop(0x14A5)
    sc.bg(0, 4, 24, 0x0000, 0x6000, opaque=True)
    sc.bg(1, 4, 24, 0x1000, 0x6400)
    sc.bg(2, 2, 24, 0x2000, 0x6800)
    sc.obj_tiles(0x4000, 8, 11)
    sc.reg(0x01, 0x02)
    sprites_demo(sc, 8, seed_off=10)
    sc.reg(0x05, 0x01)
    sc.chr_regs(0x0000, 0x1000, 0x2000, 0)
    sc.scroll(0, 0, 0x3FF)
    sc.scroll(1, 7, 0x3FF)
    sc.scroll(2, 0, 0)
    sc.coldata(4, 8, 12)
    sc.reg(0x2C, 0x13)
    sc.reg(0x2D, 0x06)
    sc.reg(0x31, 0x23 | 0x40)
    sc.reg(0x33, 0x08)
    unblank(sc)
    return sc


def scene_m7_rotate():
    sc = Scene("mode7_rotate_repeat", 1, 11)
    sc.backdrop(0x2108)
    sc.m7_data(16, 64)
    sc.reg(0x05, 0x07)
    sc.reg(0x1A, 0x00)
    sc.m7(222, 128, -128, 222, 128, 112, 0, 0)
    sc.reg(0x2C, 0x01)
    unblank(sc)
    return sc


def scene_m7_extbg():
    sc = Scene("mode7_extbg_fill_sprites", 1, 12)
    sc.backdrop(0x0421)
    sc.m7_data(16, 64, 3)
    sc.obj_tiles(0x4000, 8, 13)
    sc.reg(0x01, 0x02)
    sprites_demo(sc, 10, seed_off=12)
    sc.reg(0x05, 0x07)
    sc.reg(0x1A, 0xC0)
    sc.m7(0x0200, 0, 0, 0x0200, 100, 100, -64, -48)
    sc.reg(0x33, 0x40)
    sc.reg(0x2C, 0x13)
    unblank(sc)
    return sc


def scene_m7_flip_mosaic():
    sc = Scene("mode7_flip_mosaic_direct", 1, 13)
    sc.backdrop(0x0000)
    sc.m7_data(16, 64, 5)
    sc.reg(0x05, 0x07)
    sc.reg(0x1A, 0x03)
    sc.m7(0x0100, 0x0040, 0, 0x0100, 0, 0, 20, 10)
    sc.reg(0x06, 0x31)
    sc.reg(0x30, 0x01)
    sc.reg(0x2C, 0x01)
    unblank(sc)
    return sc


def obj_scene(name, seed, size_sel, gap, base_nib, n_spr, large_mix=True):
    sc = Scene(name, 1, seed)
    sc.backdrop(0x18C6)
    sc.bg(0, 4, 24, 0x0000, 0x6000)
    sc.reg(0x05, 0x01)
    sc.chr_regs(0x0000, 0, 0, 0)
    sc.scroll(0, 0, 0x3FF)
    obj_base = base_nib << 13
    sc.obj_tiles(obj_base, 8, seed)
    sc.obj_tiles((obj_base + ((gap + 1) << 12)) & 0x7FFF, 8, seed + 20)
    sc.reg(0x01, (size_sel << 5) | (gap << 3) | base_nib)
    sc.reg(0x2C, 0x11)
    return sc


def scene_obj0():
    sc = obj_scene("obj_size0_flips_prio", 14, 0, 0, 2, 24)
    r = Rng(70)
    for i in range(24):
        x = 6 + (i % 8) * 30 + r.int(10)
        y = 10 + (i // 8) * 70 + r.int(30)
        sc.sprite(i, x, y, (r.int(8) * 2) + (r.int(6) * 32), pal=r.int(8), prio=r.int(4), h=r.int(2), v=r.int(2),
                  tbit=r.int(2), large=r.int(2))
    unblank(sc)
    return sc


def scene_obj6():
    sc = obj_scene("obj_size6_rect_flip", 15, 6, 0, 2, 12)
    r = Rng(71)
    for i in range(12):
        x = 8 + (i % 4) * 60 + r.int(10)
        y = 12 + (i // 4) * 70 + r.int(10)
        large = i % 2
        sc.sprite(i, x, y, (r.int(6) * 2) + (r.int(2) * 32), pal=r.int(8), prio=r.int(4), h=(i // 2) % 2, v=(i // 3) % 2,
                  tbit=r.int(2), large=large)
    unblank(sc)
    return sc


def scene_obj5():
    sc = obj_scene("obj_size5_name_gap", 16, 5, 1, 1, 8)
    r = Rng(72)
    for i in range(8):
        x = 4 + (i % 4) * 62 + r.int(8)
        y = 8 + (i // 4) * 100 + r.int(20)
        sc.sprite(i, x, y, (r.int(4) * 2) + (r.int(2) * 32), pal=r.int(8), prio=r.int(4), h=r.int(2), v=r.int(2),
                  tbit=i % 2, large=(i // 2) % 2)
    unblank(sc)
    return sc


def scene_obj_limits():
    sc = obj_scene("obj_limits_slivers", 17, 4, 0, 2, 0)
    for i in range(34):
        sc.sprite(i, 8 + i * 7, 100, (i % 8) * 2, pal=i % 8, prio=0, tbit=i % 2, large=0)
    for k, i in enumerate(range(40, 52)):
        sc.sprite(i, 10 + k * 18, 150, (k % 8) * 2, pal=(k + 3) % 8, prio=1, large=0)
    sc.sprite(60, -256, 150, 0, pal=1, prio=2, large=1)
    sc.sprite(61, -256, 150, 0, pal=2, prio=2, large=1)
    sc.sprite(80, 100, 20, 0x22, pal=5, prio=3, large=1)
    unblank(sc)
    return sc


def scene_obj_rotation():
    sc = obj_scene("obj_priority_rotation", 18, 1, 0, 2, 0)
    r = Rng(73)
    for i in range(24):
        grp = i % 3
        sc.sprite(i, 40 + grp * 70 + (i // 3) * 3, 60 + (i // 3) * 4, ((i % 6) * 2) + 32 * (i % 3), pal=i % 8, prio=i % 2,
                  h=r.int(2), large=i % 2)
    sc.reg(0x02, 0x10)
    sc.reg(0x03, 0x80)
    unblank(sc)
    return sc


def window_base(name, seed):
    sc = Scene(name, 1, seed)
    sc.backdrop(0x1CE7)
    sc.bg(0, 4, 24, 0x0000, 0x6000, opaque=True)
    sc.bg(1, 4, 24, 0x1000, 0x6400, opaque=True)
    sc.bg(2, 2, 24, 0x2000, 0x6800, opaque=True)
    sc.obj_tiles(0x4000, 8, seed)
    sc.reg(0x01, 0x02)
    for i in range(10):
        sc.sprite(i, 10 + i * 24, 40 + (i % 3) * 60, (i % 6) * 2, pal=i % 8, prio=3, large=i % 2)
    sc.reg(0x05, 0x01)
    sc.chr_regs(0x0000, 0x1000, 0x2000, 0)
    sc.scroll(0, 0, 0x3FF)
    sc.scroll(1, 9, 0x3FF)
    sc.scroll(2, 4, 0x3FF)
    return sc


def scene_windows():
    sc = window_base("windows_all_logic", 19)
    sc.reg(0x26, 40)
    sc.reg(0x27, 120)
    sc.reg(0x28, 90)
    sc.reg(0x29, 200)
    sc.reg(0x23, 0xFA)
    sc.reg(0x24, 0x0B)
    sc.reg(0x25, 0x06)
    sc.reg(0x2A, 0xE4)
    sc.reg(0x2B, 0x03)
    sc.reg(0x2C, 0x17)
    sc.reg(0x2E, 0x17)
    unblank(sc)
    return sc


def scene_windows_edges():
    sc = window_base("windows_edges_subscreen", 21)
    sc.reg(0x26, 200)
    sc.reg(0x27, 50)
    sc.reg(0x28, 100)
    sc.reg(0x29, 180)
    sc.reg(0x23, 0x3A)            # BG1: window 1 inverted+enabled (empty window inverted = everything) ; BG2: g=3 -> same
    sc.reg(0x24, 0x08)            # BG3: window 2 only
    sc.reg(0x25, 0x04)            # OBJ: inversion bit without enable -> no effect
    sc.reg(0x2A, 0x00)
    sc.reg(0x2B, 0x00)
    sc.reg(0x2C, 0x11)
    sc.reg(0x2D, 0x06)
    sc.reg(0x2E, 0x01)
    sc.reg(0x2F, 0x04)
    sc.reg(0x30, 0x02)
    sc.reg(0x31, 0x01)
    unblank(sc)
    return sc


def scene_math_fixed_windows():
    sc = window_base("colormath_fixed_windows", 22)
    sc.coldata(12, 3, 20)
    sc.reg(0x26, 64)
    sc.reg(0x27, 191)
    sc.reg(0x25, 0x20)           # colour window: window 1 enabled, no inversion
    sc.reg(0x2B, 0x00)
    sc.reg(0x30, 0x40 | 0x20)    # black region = 1 (outside), math region = 2 (prevent inside)
    sc.reg(0x31, 0x73)
    sc.reg(0x2C, 0x13)
    sc.at(112, 0x31, 0xB3)
    sc.at(112, 0x30, 0x00)
    unblank(sc)
    return sc


def scene_math_sub_obj():
    sc = window_base("colormath_subscreen_obj_pal", 23)
    sc.coldata(8, 16, 4)
    sc.reg(0x2C, 0x11)
    sc.reg(0x2D, 0x06)
    sc.reg(0x30, 0x02)
    sc.reg(0x31, 0x11)
    sc.at(100, 0x31, 0x51)
    sc.at(180, 0x2D, 0x00)
    unblank(sc)
    return sc


def scene_hdma_style():
    sc = Scene("hdma_style_changes", 2, 24)
    sc.backdrop(0x0000)
    sc.bg(0, 4, 24, 0x0000, 0x6000, opaque=True)
    sc.bg(1, 4, 24, 0x1000, 0x6400)
    sc.reg(0x05, 0x01)
    sc.chr_regs(0x0000, 0x1000, 0, 0)
    sc.scroll(0, 0, 0x3FF)
    sc.scroll(1, 30, 0x3FF)
    sc.reg(0x2C, 0x01)
    for line in range(1, 225, 8):
        hv = ((line * 5) // 8) % 40
        sc.at(line, 0x0D, hv & 0xFF)
        sc.at(line, 0x0D, (hv >> 8) & 3)
        col = ((line >> 3) & 31) << 10 | ((31 - ((line >> 3) & 31)) << 5)
        sc.at(line, 0x21, 0)
        sc.at(line, 0x22, col & 0xFF)
        sc.at(line, 0x22, col >> 8)
    sc.at(112, 0x2C, 0x03)
    for line in range(200, 225, 4):
        sc.at(line, 0x00, 15 - (line - 200) // 4)
    sc.at(1, 0x00, 0x0F)
    unblank(sc, 15)
    return sc


SCENE_BUILDERS = [scene_mode0, scene_mode1_prio, scene_mode1_mosaic, scene_mode2_opt, scene_mode4_opt, scene_mode3_direct,
                  scene_mode3_palette, scene_mode5, scene_mode5_math, scene_pseudo_hires, scene_m7_rotate, scene_m7_extbg,
                  scene_m7_flip_mosaic, scene_obj0, scene_obj6, scene_obj5, scene_obj_limits, scene_obj_rotation,
                  scene_windows, scene_windows_edges, scene_math_fixed_windows, scene_math_sub_obj, scene_hdma_style]


# =====================================================================================================
#  Commands
# =====================================================================================================

HEADER = ("# Generated by docs/tools/ppu_ref.py - DO NOT EDIT.  Scenes for the PPU composition test (task E05-19).\n"
          "# Format: see task E05-19.  Expected values: ppu_scenes.expected (computed by the independent reference renderer).\n")


def build_text():
    return HEADER + "".join(b().text() for b in SCENE_BUILDERS)


def compute_expected(text, only=None):
    lines = []
    for sc in parse_scenes(text):
        if only and sc["name"] != only:
            continue
        w, h, px = simulate(sc)
        lines.append("%s %d %d %016x" % (sc["name"], w, h, frame_hash(w, h, px)))
        print("  " + lines[-1], file=sys.stderr)
    return "\n".join(lines) + "\n"


def cmd_generate():
    text = build_text()
    SCENES_TXT.write_text(text, encoding="ascii", newline="\n")
    EXPECTED.write_text(compute_expected(text), encoding="ascii", newline="\n")
    print("written %s (%d bytes) and %s" % (SCENES_TXT.relative_to(ROOT), len(text), EXPECTED.relative_to(ROOT)))
    return 0


def cmd_verify():
    text = build_text()
    if not SCENES_TXT.exists() or SCENES_TXT.read_text(encoding="ascii") != text:
        print("ppu_scenes.txt is out of date (run: python docs/tools/ppu_ref.py generate)")
        return 1
    exp = compute_expected(SCENES_TXT.read_text(encoding="ascii"))
    if not EXPECTED.exists() or EXPECTED.read_text(encoding="ascii") != exp:
        print("ppu_scenes.expected is out of date or differs from the reference renderer")
        return 1
    print("verify OK: %d scenes" % len(SCENE_BUILDERS))
    return 0


def cmd_images(outdir, only=None):
    out = Path(outdir)
    out.mkdir(parents=True, exist_ok=True)
    for sc in parse_scenes(build_text()):
        if only and sc["name"] != only:
            continue
        w, h, px = simulate(sc)
        write_ppm(out / (sc["name"] + ".ppm"), w, h, px)
        print("  %s.ppm %dx%d" % (sc["name"], w, h), file=sys.stderr)
    return 0


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    c = argv[1]
    if c == "selftest":
        return selftest()
    if c == "generate":
        return cmd_generate()
    if c == "verify":
        return cmd_verify()
    if c == "images":
        return cmd_images(argv[2] if len(argv) > 2 else str(ROOT / "tests" / "out" / "ppu_ref"), argv[3] if len(argv) > 3 else None)
    if c == "list":
        for b in SCENE_BUILDERS:
            print(b().name)
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
