#!/usr/bin/env python3
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
with open('server/src/db.rs', 'rb') as f:
    raw = f.read()
# 检查 BOM
print(f'First 3 bytes: {raw[:3].hex()}')
if raw[:3] == b'\xef\xbb\xbf':
    print('  -> UTF-8 BOM detected!')
else:
    print('  -> No BOM')

# 检查 line endings
crlf = raw.count(b'\r\n')
lf = raw.count(b'\n') - crlf
print(f'CRLF: {crlf}, LF: {lf}')

# 找 "INSERT 没有返回 id" 上下文
needle = b'INSERT \xe6\xb2\xa1\xe6\x9c\x89\xe8\xbf\x94\xe5\x9b\x9e id'
idx = raw.find(needle)
if idx < 0:
    # 试试 FFFD 版本
    needle2 = b'INSERT \xe6\xb2\xa1\xe6\x9c\x89\xe8\xbf\x94'
    idx2 = raw.find(needle2)
    print(f'FFFD version at {idx2}: {raw[idx2:idx2+50]}')
else:
    print(f'Found at {idx}: {raw[idx:idx+50]}')
