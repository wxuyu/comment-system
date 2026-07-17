$srcDir = "C:\Users\a1319\.qclaw\workspace-agent-d089e98f\comment-system\server\src"
$files = Get-ChildItem -Recurse -Path $srcDir -Filter "*.rs"
$replacement = [char]0xFFFD
foreach ($f in $files) {
    $content = [System.IO.File]::ReadAllText($f.FullName, [System.Text.Encoding]::UTF8)
    if ($content.Contains($replacement)) {
        [System.IO.File]::WriteAllText($f.FullName, $content, [System.Text.UTF8Encoding]::new($false))
        Write-Host "Fixed: $($f.Name)"
    }
}
Write-Host "Done."
