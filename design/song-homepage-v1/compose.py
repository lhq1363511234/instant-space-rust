# -*- coding: utf-8 -*-
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np, random

random.seed(7); np.random.seed(7)
W,H = 1920,1080
PAPER = (232,224,208)      # 绢本米白 #E8E0D0
INK   = (26,26,24)         # 墨黑 #1A1A18
TIANQ = (121,167,181)      # 天青 #79A7B5
CAP   = (87,87,79)         # 标注 #57574F

F = "/tmp/song-poster/LXGWWenKai-Regular.ttf"
f_title = ImageFont.truetype(F,64)
f_brand = ImageFont.truetype(F,30)
f_slogan = ImageFont.truetype(F,26)
f_micro = ImageFont.truetype(F,18)
f_small = ImageFont.truetype(F,15)
try:
    f_ital = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",26)
    f_lat  = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",22)
    f_lat2 = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",16)
except Exception as e:
    print("font fallback:", e); f_ital=f_lat=f_lat2=f_small

# ---------- base paper ----------
base = Image.new("RGB",(W,H),PAPER)
arr = np.asarray(base).astype(np.int16)
noise = np.random.normal(0,5,(H,W,1))
arr = np.clip(arr+noise,0,255).astype(np.uint8)
base = Image.fromarray(arr)

# ---------- main visual: 千里江山 band ----------
q = Image.open("qianli_orig.jpg").convert("RGB")
x0 = int(0.46*q.width) - 1750
crop = q.crop((x0, 0, x0+3500, 1600))
band = crop.resize((1400,640), Image.LANCZOS)
band = ImageEnhance.Color(band).enhance(0.96)
band = ImageEnhance.Brightness(band).enhance(0.98)
# warm blend toward paper
warm = Image.new("RGB", band.size, PAPER)
band = Image.blend(band, warm, 0.10)
bx, by = -90, 230
# alpha mask: fade right edge into paper
mask = Image.new("L", band.size, 255)
ma = np.asarray(mask).astype(np.float32)
fade_w = 150
ma[:, -fade_w:] = np.linspace(255,0,fade_w).astype(np.float32)
mask = Image.fromarray(ma.astype(np.uint8))
base.paste(band, (bx,by), mask)
# hairline divider at band right edge
d = ImageDraw.Draw(base)
d.line([(1310,230),(1310,870)], fill=(120,120,112), width=1)

# ---------- vertical title (楷体, 两列竖排) ----------
cols = ["走到导航的尽头，","才是体验的开始。"]
cxA, cxB = 1560, 1468
cy0 = 300
char_gap = 16; col_gap = 78
for ci, col in enumerate(cols):
    x = cxA if ci==0 else cxB
    y = cy0
    for ch in col:
        d.text((x, y), ch, font=f_title, fill=INK)
        y += 64 + char_gap

# ---------- brand block ----------
by2 = cy0 + 7*(64+char_gap) + 26
d.line([(1468,by2),(1610,by2)], fill=INK, width=2)
sp = "".join(c+" " for c in "INSPACE").strip()
d.text((1468, by2+20), sp, font=f_brand, fill=INK)
d.text((1468, by2+58), "Be IN the space, beyond the map.", font=f_ital, fill=(60,60,55))
d.text((1468, by2+96), "介观空间网络 · GLOBAL MESO SPACE NETWORK", font=f_lat2, fill=CAP)

# ---------- 汝窑 specimen (circle + label) ----------
r = Image.open("00_originals/ruyao.jpg").convert("RGB")
sq = min(r.size); rr = r.crop(((r.width-sq)//2,(r.height-sq)//2,(r.width+sq)//2,(r.height+sq)//2))
rr = rr.resize((148,148), Image.LANCZOS)
circ = Image.new("L",(148,148),0)
cd = ImageDraw.Draw(circ); cd.ellipse((2,2,146,146),fill=255)
cx, cy = 1745, 268
base.paste(rr,(cx-74,cy-74),circ)
d.ellipse((cx-74,cy-74,cx+74,cy+74), outline=(26,26,24), width=2)
d.text((cx-52, cy+90), "SPECIMEN · 001", font=f_lat2, fill=CAP)
d.line([(cx-60,cy+132),(cx+60,cy+132)], fill=(26,26,24), width=1)
d.text((cx-60, cy+140), "北宋 · 汝窑天青釉", font=f_small, fill=CAP)

# ---------- 瘦金 strip specimen ----------
s = Image.open("00_originals/shoujinti.jpg").convert("RGB")
sc = s.crop((int(s.width*0.35),0,s.width,218)).resize((520,45), Image.LANCZOS)
sx, sy = 1335, 108
base.paste(sc,(sx,sy))
d.rectangle((sx-2,sy-2,sx+522,sy+47), outline=(26,26,24), width=1)
d.text((sx+522, sy+58), "宋徽宗 瘦金体 · 《穠芳诗帖》", font=f_small, fill=CAP, anchor="ra")

# ---------- caption under band ----------
d.text((120, 900), "千里江山图 · 北宋 · 王希孟 · 绢本设色", font=f_micro, fill=(60,60,55))
d.text((120, 928), "A THOUSAND LI OF RIVERS AND MOUNTAINS · WANG XIMENG · 1113", font=f_lat2, fill=CAP)

# ---------- right edge rotated note ----------
note = Image.new("RGBA",(40,640),(0,0,0,0))
nd = ImageDraw.Draw(note)
n = "BEYOND THE MAP · 步出地图之外"
xx = 0
for ch in n:
    f = f_lat2 if ord(ch)<128 else f_small
    nd.text((20, xx), ch, font=f, fill=(60,60,55,200))
    wch = f.getsize(ch)[0]
    xx += wch + 8
base.paste(note, (1878, 300), note)

# ---------- global grain + vignette ----------
a = np.asarray(base).astype(np.int16)
g = np.random.normal(0,3.5,a.shape)
a = np.clip(a+g,0,255).astype(np.uint8)
base = Image.fromarray(a)
# vignette
yy,xx = np.mgrid[0:H,0:W].astype(np.float32)
cxv,cyv = W/2,H/2
dd = np.sqrt(((xx-cxv)/(W*0.62))**2 + ((yy-cyv)/(H*0.62))**2)
vig = np.clip((dd-0.82)*90,0,50).astype(np.uint8)
base = Image.fromarray(np.clip(np.asarray(base).astype(np.int16)-vig[...,None],0,255).astype(np.uint8))

# ---------- thin outer hairline ----------
d = ImageDraw.Draw(base)
d.rectangle((22,22,W-23,H-23), outline=(26,26,24), width=1)

base.save("06_outputs/final/inspace-homepage-song-v1.png")
base.save("06_outputs/final/inspace-homepage-song-v1.jpg", quality=92)
print("saved", base.size)
