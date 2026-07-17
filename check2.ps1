$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$bytes = [System.IO.File]::ReadAllBytes($path)
Write-Host "File size: $($bytes.Length) bytes"
# Read as UTF-8 strict
$enc = [System.Text.Encoding]::GetEncoding('utf-8', $null, [System.Text.DecoderExceptionFallback]::new())
try {
    $text = $enc.GetString($bytes)
    Write-Host "Valid UTF-8"
} catch {
    Write-Host "Invalid UTF-8"
}
# 检查第一行 raw bytes
$first = $bytes[0..100] | ForEach-Object { '{0:X2}' -f $_ }
Write-Host "First 100 bytes: $($first -join ' ')"
