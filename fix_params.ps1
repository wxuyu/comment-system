# 替换 db::values_of(&[...]) 为 params![...]（处理 Vec）
$srcDir = "C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
$files = Get-ChildItem -Recurse -Path $srcDir -Filter "*.rs"

foreach ($f in $files) {
    $content = Get-Content $f.FullName -Raw
    $original = $content

    # 1) db::values_of(&[])  -> params![]
    $content = [regex]::Replace($content, 'db::values_of\(&\[\]\)', 'params![]')

    # 2) db::values_of(&[&var])  -> params![var]
    $content = [regex]::Replace($content, 'db::values_of\(&\[&([a-zA-Z_][a-zA-Z0-9_\.]*)\]\)', 'params![$1]')

    # 3) db::values_of(&[&a, &b, ...])  -> params![a, b, ...]
    # 复杂模式：内部有 & 引用
    $content = [regex]::Replace($content, 'db::values_of\(&\[(.*?)\]\)', {
        param($match)
        $inner = $match.Groups[1].Value
        # 把 "&xxx" 替换为 "xxx"，但保留字符串字面量和数字
        # 简化：把所有 " &" 开头的标识符去掉 & 前缀
        $inner2 = $inner -replace ',\s*&', ', ' -replace '^\s*&\s*', ''
        return "params![$inner2]"
    })

    if ($content -ne $original) {
        Set-Content -Path $f.FullName -Value $content -NoNewline
        Write-Host "Updated: $($f.Name)"
    }
}
Write-Host "Done."
