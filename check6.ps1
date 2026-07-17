$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$bytes = [System.IO.File]::ReadAllBytes($path)
Write-Host "File size: $($bytes.Length) bytes"
Write-Host "First 4 bytes (hex): $($bytes[0].ToString('X2')) $($bytes[1].ToString('X2')) $($bytes[2].ToString('X2')) $($bytes[3].ToString('X2'))"

# 查找 'AND c.status = ' 的字节位置
$target = "AND c.status = "
$targetBytes = [System.Text.Encoding]::UTF8.GetBytes($target)
$found = $false
for ($i = 0; $i -le $bytes.Length - $targetBytes.Length; $i++) {
    $match = $true
    for ($j = 0; $j -lt $targetBytes.Length; $j++) {
        if ($bytes[$i + $j] -ne $targetBytes[$j]) {
            $match = $false
            break
        }
    }
    if ($match) {
        $found = $true
        Write-Host "Found at offset $i"
        $next = $bytes[($i)..($i + 25)]
        $hex = $next | ForEach-Object { '{0:X2}' -f $_ }
        Write-Host "Bytes: $($hex -join ' ')"
        break
    }
}
if (-not $found) {
    Write-Host "Not found"
}

# 查找 'approved' 的字节
$target2 = "approved"
$targetBytes2 = [System.Text.Encoding]::UTF8.GetBytes($target2)
$found2 = $false
for ($i = 0; $i -le $bytes.Length - $targetBytes2.Length; $i++) {
    $match = $true
    for ($j = 0; $j -lt $targetBytes2.Length; $j++) {
        if ($bytes[$i + $j] -ne $targetBytes2[$j]) {
            $match = $false
            break
        }
    }
    if ($match) {
        if (-not $found2) { Write-Host "First 'approved' at offset $i" }
        $found2 = $true
        # Look at the 5 bytes before
        $prev = $bytes[($i - 5)..($i - 1)]
        $hex = $prev | ForEach-Object { '{0:X2}' -f $_ }
        Write-Host "  Prev 5 bytes: $($hex -join ' ')"
    }
}
