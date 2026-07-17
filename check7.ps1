$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$content = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
$lines = $content -split "`n"
# Look at lines 169-180 (0-indexed)
for ($i = 168; $i -lt 180; $i++) {
    $line = $lines[$i]
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($line)
    $hex = $bytes | ForEach-Object { '{0:X2}' -f $_ }
    Write-Host "L$($i+1) (bytes): $($hex -join ' ')"
}
