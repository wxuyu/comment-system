#!/usr/bin/env python3
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
with open('server/src/db.rs', 'rb') as f:
    raw = f.read()
# Line 140 area
lines = raw.split(b'\n')
for i, line in enumerate(lines, 1):
    if 135 <= i <= 165:
        print(f'{i:4}: {line!r}')
