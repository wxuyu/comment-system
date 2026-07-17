#!/usr/bin/env python3
"""修复 db.rs 中的 v.0 -> v 模式 + 添加 values_of 函数"""
src = r"C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\db.rs"
with open(src, "r", encoding="utf-8", newline="") as f:
    content = f.read()

# 修复 v.0 模式 -> 直接用 v
# 但 Value 是个 enum，match v { Value::Null => ... }
old_block_1 = '''/// 获取字符串
pub fn row_str(row: &Row, idx: RowIdx) -> anyhow::Result<String> {
    use libsql::ValueType;
    let v = row.get_value(idx)?;
    match v.0 {
        ValueType::Null => Ok(String::new()),
        ValueType::Text => row.get::<String>(idx).map_err(Into::into),
        ValueType::Integer => row.get::<i64>(idx).map(|n| n.to_string()).map_err(Into::into),
        ValueType::Real => row.get::<f64>(idx).map(|n| n.to_string()).map_err(Into::into),
        other => anyhow::bail!("row_str: 不支持的类型 {:?}", other),
    }
}'''
new_block_1 = '''/// 获取字符串
pub fn row_str(row: &Row, idx: RowIdx) -> anyhow::Result<String> {
    let v = row.get_value(idx)?;
    match v {
        Value::Null => Ok(String::new()),
        Value::Text(s) => Ok(s),
        Value::Integer(n) => Ok(n.to_string()),
        Value::Real(n) => Ok(n.to_string()),
        other => anyhow::bail!("row_str: 不支持的类型 {:?}", other),
    }
}'''
content = content.replace(old_block_1, new_block_1)

old_block_2 = '''pub fn row_opt_str(row: &Row, idx: RowIdx) -> anyhow::Result<Option<String>> {
    use libsql::ValueType;
    let v = row.get_value(idx)?;
    if matches!(v.0, ValueType::Null) {
        return Ok(None);
    }
    Ok(Some(row_str(row, idx)?))
}'''
new_block_2 = '''pub fn row_opt_str(row: &Row, idx: RowIdx) -> anyhow::Result<Option<String>> {
    let v = row.get_value(idx)?;
    if matches!(v, Value::Null) {
        return Ok(None);
    }
    Ok(Some(row_str(row, idx)?))
}'''
content = content.replace(old_block_2, new_block_2)

old_block_3 = '''pub fn row_opt_i64(row: &Row, idx: RowIdx) -> anyhow::Result<Option<i64>> {
    use libsql::ValueType;
    let v = row.get_value(idx)?;
    if matches!(v.0, ValueType::Null) {
        return Ok(None);
    }
    row.get::<i64>(idx).map(Some).map_err(Into::into)
}'''
new_block_3 = '''pub fn row_opt_i64(row: &Row, idx: RowIdx) -> anyhow::Result<Option<i64>> {
    let v = row.get_value(idx)?;
    if matches!(v, Value::Null) {
        return Ok(None);
    }
    row.get::<i64>(idx).map(Some).map_err(Into::into)
}'''
content = content.replace(old_block_3, new_block_3)

# 修复 Builder::new_remote - url 是 &String，需要 .to_string()
content = content.replace(
    "Builder::new_remote(url, token)",
    "Builder::new_remote(url.to_string(), token.to_string())"
)

# 添加 values_of 函数（用于把 Vec<Value> 转成 params）
# 插入到 FromRow 之后
values_of_func = '''

/// 把 Vec<Value> 转成 libsql Params（用于 `&db::values_of(&[...])` 模式）
pub fn values_of(values: &[Value]) -> Vec<Value> {
    values.to_vec()
}
'''
# 找到 FromRow trait 之前的位置插入
marker = "/// FromRow 特性：让任何结构体可以从 libsql::Row 构造"
if marker in content and "pub fn values_of" not in content:
    content = content.replace(marker, values_of_func + "\n" + marker)

with open(src, "w", encoding="utf-8", newline="") as f:
    f.write(content)
print("Saved")
