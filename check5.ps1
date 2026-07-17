$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$content = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
$lines = $content -split "`n"
for ($i = 170; $i -lt 200; $i++) {
    if ($i -lt $lines.Length) {
        Write-Host "$($i+1): $($lines[$i])"
    }
}
