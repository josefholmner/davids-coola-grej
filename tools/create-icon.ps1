$ErrorActionPreference = 'Stop'
# Rebuild the application's vector tree-and-book icon at Windows icon sizes.
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
public static class LearningTreeIcon {
    public static byte[] Png(int size) {
        using (var large = new Bitmap(1024,1024)) {
            using (var g = Graphics.FromImage(large)) {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.ScaleTransform(4,4);
                using (var path = new GraphicsPath()) {
                    path.AddArc(8,8,58,58,180,90); path.AddArc(190,8,58,58,270,90);
                    path.AddArc(190,190,58,58,0,90); path.AddArc(8,190,58,58,90,90); path.CloseFigure();
                    using (var brush = new SolidBrush(Color.FromArgb(255,24,53,58))) g.FillPath(brush,path);
                }
                using (var trunk = new SolidBrush(Color.FromArgb(255,190,139,86)))
                    g.FillPolygon(trunk,new PointF[]{new PointF(113,201),new PointF(143,201),new PointF(134,97),new PointF(122,97)});
                using (var branch = new Pen(Color.FromArgb(255,190,139,86),9)) {
                    branch.StartCap=LineCap.Round;branch.EndCap=LineCap.Round;
                    g.DrawLine(branch,128,158,89,114);g.DrawLine(branch,130,153,169,112);
                }
                using (var leaf = new SolidBrush(Color.FromArgb(255,47,160,112))) g.FillEllipse(leaf,42,71,106,97);
                using (var leaf = new SolidBrush(Color.FromArgb(255,68,190,126))) g.FillEllipse(leaf,108,70,106,97);
                using (var leaf = new SolidBrush(Color.FromArgb(255,111,225,150))) g.FillEllipse(leaf,77,31,105,113);
                using (var shine = new SolidBrush(Color.FromArgb(255,179,245,190))) g.FillEllipse(shine,98,48,27,13);
                using (var fruit = new SolidBrush(Color.FromArgb(255,255,212,112))) {
                    g.FillEllipse(fruit,66,109,17,17);g.FillEllipse(fruit,164,102,17,17);g.FillEllipse(fruit,132,62,17,17);
                }
                using (var cover = new SolidBrush(Color.FromArgb(255,76,171,172)))
                    g.FillPolygon(cover,new PointF[]{new PointF(61,182),new PointF(128,191),new PointF(195,182),new PointF(195,225),new PointF(128,234),new PointF(61,225)});
                using (var page = new SolidBrush(Color.FromArgb(255,255,248,224))) {
                    g.FillPolygon(page,new PointF[]{new PointF(66,177),new PointF(128,188),new PointF(128,226),new PointF(66,215)});
                    g.FillPolygon(page,new PointF[]{new PointF(128,188),new PointF(190,177),new PointF(190,215),new PointF(128,226)});
                }
                using(var spine=new Pen(Color.FromArgb(255,216,208,177),2))g.DrawLine(spine,128,189,128,225);
            }
            using(var result = new Bitmap(size,size)) {
                using(var g=Graphics.FromImage(result)) {g.InterpolationMode=InterpolationMode.HighQualityBicubic;g.PixelOffsetMode=PixelOffsetMode.HighQuality;g.DrawImage(large,0,0,size,size);}
                using(var stream=new MemoryStream()){result.Save(stream,ImageFormat.Png);return stream.ToArray();}
            }
        }
    }
}
"@ -ReferencedAssemblies System.Drawing.Common,System.Drawing.Primitives,(Join-Path $PSHOME "System.Private.Windows.GdiPlus.dll"),(Join-Path $PSHOME "System.Private.Windows.Core.dll")
$root=Split-Path $PSScriptRoot -Parent
$sizes=@(16,24,32,48,64,128,256)
$images=@($sizes | ForEach-Object {,[LearningTreeIcon]::Png($_)})
$stream=[System.IO.MemoryStream]::new()
$writer=[System.IO.BinaryWriter]::new($stream)
$writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]$sizes.Count)
$offset=6+16*$sizes.Count
for($index=0;$index -lt $sizes.Count;$index++){
    $dimension=if($sizes[$index] -eq 256){0}else{$sizes[$index]}
    $writer.Write([byte]$dimension);$writer.Write([byte]$dimension);$writer.Write([byte]0);$writer.Write([byte]0)
    $writer.Write([uint16]1);$writer.Write([uint16]32);$writer.Write([uint32]$images[$index].Length);$writer.Write([uint32]$offset)
    $offset+=$images[$index].Length
}
foreach($image in $images){$writer.Write([byte[]]$image)}
[System.IO.File]::WriteAllBytes((Join-Path $root 'assets\app.ico'),$stream.ToArray())
$writer.Dispose();$stream.Dispose()
