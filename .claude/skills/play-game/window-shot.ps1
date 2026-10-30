<#
Saves ONE window (never the whole desktop) to a PNG, so the game can be looked at.

  powershell -NoProfile -ExecutionPolicy Bypass -File window-shot.ps1 -Title "Laivanupotus"
  powershell -NoProfile -ExecutionPolicy Bypass -File window-shot.ps1 -ProcId 1234 -Out C:\x\y.png

-Title   exact window title (FindWindow)
-ProcId  main window of that process id
-Out     PNG path, default %TEMP%\bs_shot.png
-Screen  copy the window's pixels from the desktop instead of PrintWindow. Use it when the
         PrintWindow image is black (GPU-drawn windows sometimes are). The window must then
         be visible and not covered, otherwise the covering window's pixels end up in the PNG.

The visible frame is cropped to the window's real bounds (no invisible resize border).
ASCII only on purpose: Windows PowerShell 5.1 reads BOM-less files as ANSI.
#>
param(
  [string]$Title,
  [int]$ProcId,
  [string]$Out = (Join-Path $env:TEMP 'bs_shot.png'),
  [switch]$Screen
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
if (-not ('BsWin' -as [type])) {
  Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class BsWin {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int size);
}
"@
}
[void][BsWin]::SetProcessDPIAware()

# [NullString]::Value, because PowerShell turns a plain $null into "" for string parameters
if ($Title) { $h = [BsWin]::FindWindow([NullString]::Value, $Title) }
elseif ($ProcId) { $h = (Get-Process -Id $ProcId).MainWindowHandle }
else { throw 'Give -Title or -ProcId.' }
if ($h -eq [IntPtr]::Zero) { throw 'Window not found.' }
if ([BsWin]::IsIconic($h)) { throw 'Window is minimized, nothing to capture.' }

$win = New-Object BsWin+RECT
[void][BsWin]::GetWindowRect($h, [ref]$win)
$vis = New-Object BsWin+RECT
# 9 = DWMWA_EXTENDED_FRAME_BOUNDS, the visible frame without the invisible resize border
if ([BsWin]::DwmGetWindowAttribute($h, 9, [ref]$vis, 16) -ne 0) { $vis = $win }

$full = New-Object Drawing.Bitmap ($win.R - $win.L), ($win.B - $win.T)
$g = [Drawing.Graphics]::FromImage($full)
if ($Screen) {
  $g.CopyFromScreen($win.L, $win.T, 0, 0, $full.Size)
} else {
  $dc = $g.GetHdc()
  [void][BsWin]::PrintWindow($h, $dc, 2)  # 2 = PW_RENDERFULLCONTENT
  $g.ReleaseHdc($dc)
}
$g.Dispose()

$crop = New-Object Drawing.Rectangle ($vis.L - $win.L), ($vis.T - $win.T), ($vis.R - $vis.L), ($vis.B - $vis.T)
$bmp = $full.Clone($crop, $full.PixelFormat)
$bmp.Save($Out, [Drawing.Imaging.ImageFormat]::Png)
$full.Dispose(); $bmp.Dispose()
'{0} ({1}x{2})' -f $Out, $crop.Width, $crop.Height
