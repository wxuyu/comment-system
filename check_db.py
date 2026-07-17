#!/usr/bin/env python3
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
with open('server/src/db.rs', 'rb') as f:
    raw = f.read()
needle = b'format!("{} RETURNING id'
idx = raw.find(needle)
print(f'Found at {idx}')
print(f'Bytes:')
print(repr(raw[idx-5:idx+60]))
