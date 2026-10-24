param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("inventory", "decompile", "rtti")]
    [string]$Mode,

    [string]$List,

    [string]$Ghidra = "C:\Tools\ghidra_12.1.4_PUBLIC",

    [string]$Out = "third_party\rekordbox\re\export",

    # Pasta que contém rekord_clone.gpr; padrão = raiz do repo (projeto original).
    [string]$ProjectDir,

    # Heap do Java (ex.: 5G). Padrão do Ghidra headless é 2G, pouco para o modo rtti.
    [string]$MaxMem
)

if (-not [string]::IsNullOrWhiteSpace($MaxMem)) {
    $env:GHIDRA_HEADLESS_MAXMEM = $MaxMem
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$projectRoot = if ([string]::IsNullOrWhiteSpace($ProjectDir)) {
    $repoRoot
} elseif ([System.IO.Path]::IsPathRooted($ProjectDir)) {
    [System.IO.Path]::GetFullPath($ProjectDir)
} else {
    [System.IO.Path]::GetFullPath((Join-Path $repoRoot $ProjectDir))
}
if (-not (Test-Path -LiteralPath (Join-Path $projectRoot "rekord_clone.gpr") -PathType Leaf)) {
    Write-Error "rekord_clone.gpr nao encontrado em: $projectRoot" -ErrorAction Continue
    exit 2
}
$lock = Get-ChildItem -LiteralPath $projectRoot -Filter "rekord_clone.lock*" -Force -ErrorAction SilentlyContinue |
    Select-Object -First 1

if ($null -ne $lock) {
    Write-Error "Projeto Ghidra aberto (rekord_clone.lock encontrado). Feche o Ghidra e rode de novo." -ErrorAction Continue
    exit 3
}

$headless = Join-Path $Ghidra "support\analyzeHeadless.bat"
if (-not (Test-Path -LiteralPath $headless -PathType Leaf)) {
    Write-Error "analyzeHeadless.bat nao encontrado em: $headless" -ErrorAction Continue
    exit 2
}

$outPath = if ([System.IO.Path]::IsPathRooted($Out)) {
    [System.IO.Path]::GetFullPath($Out)
} else {
    [System.IO.Path]::GetFullPath((Join-Path $repoRoot $Out))
}

$scriptPath = Join-Path $repoRoot "tools\re"
$arguments = @(
    $projectRoot,
    "rekord_clone",
    "-process", "rekordbox.exe",
    "-noanalysis",
    "-scriptPath", $scriptPath
)

if ($Mode -eq "rtti") {
    # Único modo que grava no projeto: só numa cópia, nunca no original.
    if ($projectRoot -eq $repoRoot) {
        Write-Error "O modo rtti grava no projeto; use -ProjectDir com uma copia." -ErrorAction Continue
        exit 2
    }
    $arguments += @("-postScript", "RecoverClassesFromRTTIScript.java")
} elseif ($Mode -eq "inventory") {
    $arguments += "-readOnly"
    $arguments += @("-postScript", "ExportInventory.java", $outPath)
} else {
    if ([string]::IsNullOrWhiteSpace($List)) {
        Write-Error "O modo decompile exige -List <arquivo>." -ErrorAction Continue
        exit 2
    }

    $listPath = if ([System.IO.Path]::IsPathRooted($List)) {
        [System.IO.Path]::GetFullPath($List)
    } else {
        [System.IO.Path]::GetFullPath((Join-Path $repoRoot $List))
    }
    if (-not (Test-Path -LiteralPath $listPath -PathType Leaf)) {
        Write-Error "Arquivo de enderecos nao encontrado: $listPath" -ErrorAction Continue
        exit 2
    }
    $arguments += @("-readOnly", "-postScript", "DecompileList.java", $outPath, $listPath)
}

& $headless @arguments
exit $LASTEXITCODE
