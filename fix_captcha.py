#!/usr/bin/env python3
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\captcha.rs"
with open(src, "rb") as f:
    raw = f.read()
lines = raw.split(b"\n")
# 修复 L110: 把 ?)?  改为 ")?
old_l110 = lines[109]
# 当前: b'... 失败?)?;'
# 期望: b'... 失败")?;'
# 找 ?)?  模式
if b"?)?;" in old_l110:
    new_l110 = old_l110.replace(b"?)?;", b'")?;')
    print(f"L110: {old_l110!r} -> {new_l110!r}")
    lines[109] = new_l110
# 修复 L249: error-codes -> error_codes
old_l249 = lines[248]
if b"error-codes" in old_l249:
    new_l249 = old_l249.replace(b"error-codes", b"error_codes")
    print(f"L249: {old_l249!r} -> {new_l249!r}")
    lines[248] = new_l249
with open(src, "wb") as f:
    f.write(b"\n".join(lines))
print("Saved")
