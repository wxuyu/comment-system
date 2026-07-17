$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$bytes = [System.IO.File]::ReadAllBytes($path)
$first = $bytes[0..10] | ForEach-Object { '{0:X2}' -f $_ }
Write-Host "First 10 bytes: $($first -join ' ')"

# Try to detect encoding
$utf8 = [System.Text.Encoding]::UTF8
$text = $utf8.GetString($bytes)
$ok = $true
foreach ($c in $text.ToCharArray()) {
    if ([int]$c -gt 127 -and [int]$c -lt 0x4E00) {
        # 非 ASCII 也非中文
    }
}

# 直接用 UTF-8 写回
$path2 = $path
$content = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
[System.IO.File]::WriteAllText($path2, $content, [System.Text.UTF8Encoding]::new($false))
Write-Host "Wrote as UTF-8 no BOM"

# Verify
$bytes2 = [System.IO.File]::ReadAllBytes($path2)
$first2 = $bytes2[0..10] | ForEach-Object { '{0:X2}' -f $_ }
Write-Host "After: $($first2 -join ' ')"
