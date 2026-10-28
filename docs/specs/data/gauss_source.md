# Source: SNES S-DSP Gaussian Interpolation Table (GAUSS[512])

## URL
https://raw.githubusercontent.com/snes9xgit/snes9x/master/apu/bapu/dsp/SPC_DSP.cpp

## Date retrieved
2026-10-28

## File / section
`snes9x/apu/bapu/dsp/SPC_DSP.cpp` — `static short const gauss [512] = { ... };` (Gaussian interpolation table)

## Notes
This is the canonical 512-entry Gaussian interpolation lookup table used by the SNES S-DSP for sample-rate conversion. Values are unsigned shorts in range [0, 1305], non-decreasing, with GAUSS[0]=0 and GAUSS[511]=1305 (0x519).

## Validation
```
$ python docs/tools/check_gauss.py docs/specs/data/gauss_table.tsv
OK: 512 values, range [0, 1305], non-decreasing, sum property holds for all f in [0, 255]
```
