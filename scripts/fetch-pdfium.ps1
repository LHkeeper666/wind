param()

$ErrorActionPreference = 'Stop'
$version = '151.0.7891.0'
$url = 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7891/pdfium-win-x64.tgz'
$archiveSha256 = '1A5B95FDE0EB446A5709FFC4A2E6691FA2B5ACE224CEE20DDDD16C110C5CE60E'
$dllSha256 = 'E66BD5AEFC2829F12EF41227F83A6EC6D60673EC77EF64153A05E7FFBAE9072E'
$root = Split-Path -Parent $PSScriptRoot
$dllPath = Join-Path $root 'src-tauri/resources/pdfium.dll'
$licenseRoot = Join-Path $root 'src-tauri/third-party/pdfium'

$client = [System.Net.Http.HttpClient]::new()
try {
    $archive = $client.GetByteArrayAsync($url).GetAwaiter().GetResult()
} finally {
    $client.Dispose()
}
$actualArchiveHash = [Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($archive))
if ($actualArchiveHash -ne $archiveSha256) {
    throw "PDFium archive SHA-256 mismatch: $actualArchiveHash"
}

$archiveStream = [System.IO.MemoryStream]::new($archive)
$gzip = [System.IO.Compression.GZipStream]::new($archiveStream, [System.IO.Compression.CompressionMode]::Decompress)
$reader = [System.Formats.Tar.TarReader]::new($gzip)
$foundDll = $false
$licenseCount = 0
try {
    while ($entry = $reader.GetNextEntry()) {
        $name = $entry.Name.Replace('\', '/')
        if ($entry.EntryType -ne [System.Formats.Tar.TarEntryType]::RegularFile) { continue }
        if ($name -ne 'bin/pdfium.dll' -and $name -ne 'LICENSE' -and -not $name.StartsWith('licenses/')) { continue }

        $content = [System.IO.MemoryStream]::new()
        try {
            $entry.DataStream.CopyTo($content)
            $bytes = $content.ToArray()
        } finally {
            $content.Dispose()
        }
        if ($name -eq 'bin/pdfium.dll') {
            $actualDllHash = [Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($bytes))
            if ($actualDllHash -ne $dllSha256) { throw "PDFium DLL SHA-256 mismatch: $actualDllHash" }
            $destination = $dllPath
            $foundDll = $true
        } else {
            $destination = Join-Path $licenseRoot $name
            $licenseCount++
        }
        $directory = Split-Path -Parent $destination
        [System.IO.Directory]::CreateDirectory($directory) | Out-Null
        [System.IO.File]::WriteAllBytes($destination, $bytes)
    }
} finally {
    $reader.Dispose()
    $gzip.Dispose()
    $archiveStream.Dispose()
}
if (-not $foundDll -or $licenseCount -eq 0) { throw 'PDFium archive is missing the DLL or license files.' }
Write-Host "PDFium $version verified; extracted DLL and $licenseCount license files."
