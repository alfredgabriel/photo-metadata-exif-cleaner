Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap(512, 512)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.Clear([System.Drawing.Color]::Transparent)
$pen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 59, 130, 246), 40)
$pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$g.DrawRectangle($pen, 64, 160, 384, 256)
$g.DrawRectangle($pen, 176, 80, 160, 80)
$g.DrawEllipse($pen, 192, 224, 128, 128)
$g.FillEllipse([System.Drawing.Brushes]::White, 192, 224, 128, 128)
$pen.Dispose()
$bmp.Save("transparent-icon.png", [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
