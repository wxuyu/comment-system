$srcDir = "C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
$files = Get-ChildItem -Recurse -Path $srcDir -Filter "*.rs"
$replacement = [char]0xFFFD
foreach ($f in $files) {
    $content = [System.IO.File]::ReadAllText($f.FullName, [System.Text.Encoding]::UTF8)
    $count = 0
    foreach ($c in $content.ToCharArray()) {
        if ($c -eq $replacement) { $count++ }
    }
    if ($count -gt 0) {
        Write-Host "Corrupted: $($f.Name) - $count FFFD chars"
    }
}
