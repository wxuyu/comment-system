# 强制所有 .rs 文件为 UTF-8 无 BOM
$srcDir = "C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
$files = Get-ChildItem -Recurse -Path $srcDir -Filter "*.rs"

foreach ($f in $files) {
    try {
        $bytes = [System.IO.File]::ReadAllBytes($f.FullName)
        # 检查 BOM
        if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
            # 已经有 UTF-8 BOM，保持
            continue
        }
        # 检查是否 UTF-16 LE
        if ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE) {
            # UTF-16 LE，转换
            $content = [System.Text.Encoding]::Unicode.GetString($bytes)
            [System.IO.File]::WriteAllText($f.FullName, $content, [System.Text.UTF8Encoding]::new($false))
            Write-Host "Converted from UTF-16: $($f.Name)"
        }
    } catch {
        Write-Host "Error: $($f.Name): $_"
    }
}
Write-Host "Done."
