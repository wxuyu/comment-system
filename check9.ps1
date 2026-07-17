$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$bytes = [System.IO.File]::ReadAllBytes($path)
# 查找 0xE8 0xAF 后面不是 0x80-0xBF 的位置
$bad = @()
for ($i = 0; $i -lt $bytes.Length - 2; $i++) {
    if ($bytes[$i] -eq 0xE8 -and $bytes[$i+1] -ge 0x80 -and $bytes[$i+1] -le 0xBF) {
        # start of 3-byte UTF-8
        if ($bytes[$i+2] -lt 0x80 -or $bytes[$i+2] -gt 0xBF) {
            $bad += $i
        }
    }
}
Write-Host "Found $($bad.Count) bad UTF-8 sequences"
foreach ($i in $bad[0..5]) {
    Write-Host "  At offset $i: $($bytes[$i].ToString('X2')) $($bytes[$i+1].ToString('X2')) $($bytes[$i+2].ToString('X2'))"
}

# 找 BOM
if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
    Write-Host "Has UTF-8 BOM"
} elseif ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE) {
    Write-Host "Has UTF-16 LE BOM"
} elseif ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFE -and $bytes[1] -eq 0xFF) {
    Write-Host "Has UTF-16 BE BOM"
} else {
    Write-Host "No BOM"
}
