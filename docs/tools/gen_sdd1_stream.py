import sys
sys.path.insert(0, 'docs/tools')
from sdd1_ref import lcg_bytes

seed = 0x1000
data = lcg_bytes(seed, 4096)
data[0] = (data[0] & 0x0F) | (0 << 4)  # header $00
# Write first 256 bytes as hex for embedding in test
print(''.join(f'{b:02x}' for b in data[:256]))
