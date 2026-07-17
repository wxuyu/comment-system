#!/usr/bin/env python3
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
# 验证整个文件可以被 UTF-8 严格解码
for f in ['server/src/db.rs', 'server/src/captcha.rs', 'server/src/oauth.rs',
          'server/src/routes/comments.rs', 'server/src/routes/admin.rs',
          'server/src/routes/email.rs', 'server/src/routes/pages.rs',
          'server/src/routes/public.rs', 'server/src/routes/sites.rs',
          'server/src/routes/uploads.rs']:
    with open(f, 'rb') as fp:
        raw = fp.read()
    # 尝试严格 UTF-8 解码
    try:
        raw.decode('utf-8', errors='strict')
        # 检查非 ASCII 字节
        non_ascii = sum(1 for b in raw if b > 0x7F)
        print(f'{f}: UTF-8 OK, {non_ascii} non-ASCII bytes')
    except UnicodeDecodeError as e:
        print(f'{f}: UTF-8 ERROR at byte {e.start}: {raw[max(0,e.start-5):e.start+5]!r}')
        # 显示附近的可读字符
        print(f'  Context: {raw[max(0,e.start-30):e.start+30]!r}')
