# medir_ui.ps1 - mede a geometria da UI de um processo Windows em execucao.
#
# Uso:
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools/ui/medir_ui.ps1
#   powershell ... -File tools/ui/medir_ui.ps1 -NomeProcesso rekordbox -SaidaDir <dir>
#
# Produz em -SaidaDir (padrao: third_party/rekordbox/ui-workbench/medicoes):
#   meta.txt    tela, DPI, retangulo da janela principal, origem/tamanho do cliente
#   janela.tsv  janelas de nivel superior e filhas do processo (classe, titulo, retangulo)
#   uia.tsv     arvore de acessibilidade completa (UIA): tipo, nome, automationId, retangulo
#   captura.png janela inteira (inclui moldura), via PrintWindow - nao exige foco
#   cliente.raw pixels RGB crus da area de cliente, 3 bytes por pixel, linha a linha
#
# Nada aqui traz o rekordbox para frente: a captura usa PrintWindow (PW_RENDERFULLCONTENT),
# que funciona com a janela atras de outras. O processo precisa estar na sessao interativa.

param(
    [string]$NomeProcesso = 'rekordbox',
    [string]$SaidaDir = 'third_party/rekordbox/ui-workbench/medicoes',
    # PrintWindow nao exige foco, mas a area de reproducao do rekordbox e composta na GPU:
    # com a janela ocluida ela sai preta. '-TrazerParaFrente' resolve isso com um foco breve.
    [switch]$TrazerParaFrente
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

$sig = @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class MedWin32 {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool attach);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc cb, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
'@
Add-Type -TypeDefinition $sig

# PER_MONITOR_AWARE_V2: sem isso o Windows devolve coordenadas virtualizadas
# (nesta maquina a escala e 125%, logo 1920x1080 fisicos viram 1536x864 logicos).
[void][MedWin32]::SetProcessDpiAwarenessContext([IntPtr](-4))

$pids = @()
foreach ($p in (Get-Process -Name $NomeProcesso -ErrorAction SilentlyContinue)) { $pids += [int]$p.Id }
if ($pids.Count -eq 0) { throw "processo '$NomeProcesso' nao esta em execucao" }

$dir = Join-Path (Get-Location) $SaidaDir
New-Item -ItemType Directory -Force -Path $dir | Out-Null

function Descreve([IntPtr]$h, [string]$tag) {
    $r = New-Object MedWin32+RECT
    [void][MedWin32]::GetWindowRect($h, [ref]$r)
    $cr = New-Object MedWin32+RECT
    [void][MedWin32]::GetClientRect($h, [ref]$cr)
    $t = New-Object System.Text.StringBuilder 512
    [void][MedWin32]::GetWindowTextW($h, $t, 512)
    $c = New-Object System.Text.StringBuilder 256
    [void][MedWin32]::GetClassNameW($h, $c, 256)
    return ("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}`t{8}`t{9}`t{10}`t{11}" -f `
        $tag, ('0x{0:X}' -f $h.ToInt64()), $c.ToString(), $t.ToString(), `
        $r.Left, $r.Top, ($r.Right - $r.Left), ($r.Bottom - $r.Top), `
        $cr.Right, $cr.Bottom, [MedWin32]::GetDpiForWindow($h), [MedWin32]::IsWindowVisible($h))
}

# ---- janela principal: a de maior area entre as janelas visiveis do processo
# Um app recem-lancado pode levar alguns segundos ate criar e mapear a janela, entao
# a procura se repete por ate 10 s em vez de falhar na primeira tentativa.
$script:principal = [IntPtr]::Zero
$script:area = 0
$script:pids = $pids
$script:linhas = New-Object System.Collections.Generic.List[string]
$script:linhas.Add("tag`thwnd`tclasse`ttitulo`tx`ty`tlargura`taltura`tcliente_l`tcliente_a`tdpi`tvisivel")
for ($tentativa = 0; $tentativa -lt 20 -and $script:principal -eq [IntPtr]::Zero; $tentativa++) {
    if ($tentativa -gt 0) { Start-Sleep -Milliseconds 500 }
    $script:linhas.Clear()
    $script:linhas.Add("tag`thwnd`tclasse`ttitulo`tx`ty`tlargura`taltura`tcliente_l`tcliente_a`tdpi`tvisivel")
    $script:area = 0
    $cb = [MedWin32+EnumProc]{
        param($h, $l)
        $wpid = [uint32]0
        [void][MedWin32]::GetWindowThreadProcessId($h, [ref]$wpid)
        if ($script:pids -contains [int]$wpid) {
            $r = New-Object MedWin32+RECT
            [void][MedWin32]::GetWindowRect($h, [ref]$r)
            $a = ($r.Right - $r.Left) * ($r.Bottom - $r.Top)
            if ([MedWin32]::IsWindowVisible($h) -and $a -gt $script:area) { $script:area = $a; $script:principal = $h }
            $script:linhas.Add((Descreve $h 'topo'))
        }
        return $true
    }
    [void][MedWin32]::EnumWindows($cb, [IntPtr]::Zero)
}
if ($script:principal -eq [IntPtr]::Zero) { throw "nenhuma janela visivel de '$NomeProcesso' apos 10 s" }

$cbFilho = [MedWin32+EnumProc]{
    param($h, $l)
    $script:linhas.Add((Descreve $h 'filha'))
    return $true
}
[void][MedWin32]::EnumChildWindows($script:principal, $cbFilho, [IntPtr]::Zero)
Set-Content -Encoding UTF8 (Join-Path $dir 'janela.tsv') ($script:linhas -join "`n")

# ---- geometria da janela e do cliente
$wr = New-Object MedWin32+RECT
[void][MedWin32]::GetWindowRect($script:principal, [ref]$wr)
$cr = New-Object MedWin32+RECT
[void][MedWin32]::GetClientRect($script:principal, [ref]$cr)
$pt = New-Object MedWin32+POINT
[void][MedWin32]::ClientToScreen($script:principal, [ref]$pt)
$dpi = [MedWin32]::GetDpiForWindow($script:principal)
$tela = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds

$meta = @(
    ("processo=" + $NomeProcesso),
    ("pids=" + ($pids -join ',')),
    ("hwnd=" + ('0x{0:X}' -f $script:principal.ToInt64())),
    ("tela=" + $tela.Width + "x" + $tela.Height),
    ("dpi=" + $dpi),
    ("escala=" + ([math]::Round($dpi / 96.0, 4))),
    ("janela_x=" + $wr.Left), ("janela_y=" + $wr.Top),
    ("janela_l=" + ($wr.Right - $wr.Left)), ("janela_a=" + ($wr.Bottom - $wr.Top)),
    ("cliente_x=" + $pt.X), ("cliente_y=" + $pt.Y),
    ("cliente_l=" + $cr.Right), ("cliente_a=" + $cr.Bottom)
)
Set-Content -Encoding UTF8 (Join-Path $dir 'meta.txt') ($meta -join "`n")

# ---- captura da janela inteira
# PrintWindow nao rouba foco, mas pode devolver preto nas superficies compostas na GPU.
# Com -TrazerParaFrente o foco e tomado por ~0,4 s e devolvido a janela anterior.
$larguraJanela = $wr.Right - $wr.Left
$alturaJanela = $wr.Bottom - $wr.Top
$bmp = New-Object System.Drawing.Bitmap($larguraJanela, $alturaJanela, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$anterior = [MedWin32]::GetForegroundWindow()
$modoCaptura = 'printwindow'
if ($TrazerParaFrente) {
    $modoCaptura = 'frente'
    if (-not [MedWin32]::SetForegroundWindow($script:principal)) {
        $tid = [MedWin32]::GetCurrentThreadId()
        $tidAlvo = [MedWin32]::GetWindowThreadProcessId($script:principal, [ref]([uint32]0))
        [void][MedWin32]::AttachThreadInput($tid, $tidAlvo, $true)
        [void][MedWin32]::SetForegroundWindow($script:principal)
        [void][MedWin32]::AttachThreadInput($tid, $tidAlvo, $false)
    }
    Start-Sleep -Milliseconds 400
    $g0 = [System.Drawing.Graphics]::FromImage($bmp)
    $g0.CopyFromScreen($wr.Left, $wr.Top, 0, 0, (New-Object System.Drawing.Size($larguraJanela, $alturaJanela)))
    $g0.Dispose()
    $ok = $true
} else {
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $g.GetHdc()
    $ok = [MedWin32]::PrintWindow($script:principal, $hdc, 2)
    $g.ReleaseHdc($hdc)
    $g.Dispose()
}
if ($anterior -ne [IntPtr]::Zero -and $anterior -ne $script:principal) { [void][MedWin32]::SetForegroundWindow($anterior) }
$bmp.Save((Join-Path $dir 'captura.png'), [System.Drawing.Imaging.ImageFormat]::Png)

# conferencia: uma area de reproducao legitima tem muita variacao de cor; preto uniforme nao
$amostraCores = @{}
for ($y = 100; $y -lt [Math]::Min(500, $alturaJanela); $y += 37) {
    for ($x = 20; $x -lt [Math]::Min(1900, $larguraJanela); $x += 53) {
        $amostraCores[$bmp.GetPixel($x, $y).ToArgb()] = 1
    }
}
$coresDistintas = $amostraCores.Count
if ($coresDistintas -lt 12) { [Console]::WriteLine("AVISO: captura com apenas $coresDistintas cores na area de reproducao; use -TrazerParaFrente") }
Add-Content -Encoding UTF8 (Join-Path $dir 'meta.txt') ("captura_modo=" + $modoCaptura)
Add-Content -Encoding UTF8 (Join-Path $dir 'meta.txt') ("captura_cores_distintas=" + $coresDistintas)

# ---- pixels crus somente da area de cliente, para analise sem dependencias
$cliente = New-Object System.Drawing.Bitmap($cr.Right, $cr.Bottom, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g2 = [System.Drawing.Graphics]::FromImage($cliente)
$g2.DrawImage($bmp, (New-Object System.Drawing.Rectangle(0, 0, $cr.Right, $cr.Bottom)),
    (New-Object System.Drawing.Rectangle(($pt.X - $wr.Left), ($pt.Y - $wr.Top), $cr.Right, $cr.Bottom)),
    [System.Drawing.GraphicsUnit]::Pixel)
$g2.Dispose()
$rect = New-Object System.Drawing.Rectangle(0, 0, $cliente.Width, $cliente.Height)
$dados = $cliente.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$bytes = New-Object byte[] ($dados.Stride * $cliente.Height)
[System.Runtime.InteropServices.Marshal]::Copy($dados.Scan0, $bytes, 0, $bytes.Length)
$cliente.UnlockBits($dados)
$stride = $dados.Stride
$cliente.Dispose()
[System.IO.File]::WriteAllBytes((Join-Path $dir 'cliente.raw'), $bytes)
Add-Content -Encoding UTF8 (Join-Path $dir 'meta.txt') ("cliente_stride=" + $stride)
$bmp.Dispose()

# ---- arvore de acessibilidade
$raiz = [System.Windows.Automation.AutomationElement]::FromHandle($script:principal)
$uia = New-Object System.Collections.Generic.List[string]
$uia.Add("id`tdepth`tparent`ttipo`tnome`tautomationid`tx`ty`tlargura`taltura")
$script:id = 0
$script:uia = $uia

function Percorre($el, [int]$prof, [string]$pai) {
    $script:id++
    $meu = $script:id
    $r = $el.Current.BoundingRectangle
    $nome = $el.Current.Name -replace "`t", ' ' -replace "`r?`n", ' '
    $aid = $el.Current.AutomationId -replace "`t", ' ' -replace "`r?`n", ' '
    $aid = $aid -replace '(\.<empty>)+', '.<...>'
    $tipo = $el.Current.ControlType.ProgrammaticName -replace 'ControlType\.', ''
    $script:uia.Add(("{0}`t{1}`t{2}`t{3}`t{4}`t{5}`t{6}`t{7}`t{8}`t{9}" -f `
        $meu, $prof, $pai, $tipo, $nome, $aid, [int]$r.X, [int]$r.Y, [int]$r.Width, [int]$r.Height))
    if ($prof -ge 40) { return }
    $filhos = $el.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
    foreach ($f in $filhos) { Percorre $f ($prof + 1) $meu }
}
Percorre $raiz 0 '0'
Set-Content -Encoding UTF8 (Join-Path $dir 'uia.tsv') ($uia -join "`n")

[Console]::WriteLine("saida=" + $dir)
[Console]::WriteLine("printwindow=" + $ok + " nos_uia=" + $script:id)
[Console]::WriteLine(($meta -join ' '))
