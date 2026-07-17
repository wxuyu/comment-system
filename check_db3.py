#!/usr/bin/env python3
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
with open('server/src/db.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()
# 检查 130-165
for i in range(130, 166):
    line = lines[i-1]
    # 打印每个字符的 repr
    print(f'{i:4}: {line!r}')
