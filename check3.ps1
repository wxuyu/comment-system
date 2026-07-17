$content = [System.IO.File]::ReadAllText('C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs', [System.Text.Encoding]::UTF8)
# 查找 'approved'
$idx = $content.IndexOf("approved")
if ($idx -ge 0) {
    $start = [Math]::Max(0, $idx - 30)
    $end = [Math]::Min($content.Length, $idx + 50)
    $snippet = $content.Substring($start, $end - $start)
    Write-Host "Context: ...$snippet..."
}
# 检查是否有 "approved" 出现
$idx2 = $content.IndexOf('"approved"')
Write-Host "Index of `"approved`": $idx2"
