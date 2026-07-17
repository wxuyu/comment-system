$path = 'C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src\routes\comments.rs'
$content = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
# 查找 "AND c.status = 'approved'" 字符串
$idx = $content.IndexOf("AND c.status = ")
Write-Host "Index: $idx"
if ($idx -ge 0) {
    $snippet = $content.Substring($idx, 30)
    # 逐字符输出字节
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($snippet)
    $hex = $bytes | ForEach-Object { '{0:X2}' -f $_ }
    Write-Host "Bytes: $($hex -join ' ')"
    Write-Host "Snippet: $snippet"
}
