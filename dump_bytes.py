#!/usr/bin/env python3
import sys
with open('server/src/db.rs', 'rb') as f:
    raw = f.read()
needle = b'format!("{} RETURNING id'
idx = raw.find(needle)
print(f'format! pattern at byte {idx}')
for i in range(max(0,idx-5), idx+50):
    b = raw[i]
    c = chr(b) if 0x20 <= b < 0x7F else '?'
    print(f'  byte {i}: 0x{b:02x} ({c})')
