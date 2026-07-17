$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$content = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
# 查找 'approved'
$idx = $content.IndexOf("'approved'")
Write-Host "Index of 'approved': $idx"
if ($idx -ge 0) {
    $snippet = $content.Substring([Math]::Max(0, $idx - 100), 200)
    Write-Host "Context:"
    Write-Host $snippet
    Write-Host "---"
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($snippet)
    $hex = $bytes | ForEach-Object { '{0:X2}' -f $_ }
    Write-Host "Bytes:"
    Write-Host ($hex -join ' ')
}
