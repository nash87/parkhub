#!/usr/bin/env python3
"""Generate PNG screenshots for ParkHub documentation."""

from PIL import Image, ImageDraw, ImageFont
import os

W, H = 800, 500
BG = (15, 23, 42)        # #0f172a
BLUE = (59, 130, 246)     # #3b82f6
DARK_BLUE = (29, 78, 216)
WHITE = (255, 255, 255)
GRAY = (148, 163, 184)
LIGHT_GRAY = (203, 213, 225)
CARD_BG = (30, 41, 59)    # #1e293b
CARD_BORDER = (51, 65, 85)
GREEN = (34, 197, 94)
RED = (239, 68, 68)
YELLOW = (250, 204, 21)
PURPLE = (168, 85, 247)

OUT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'docs', 'screenshots')
os.makedirs(OUT_DIR, exist_ok=True)

try:
    font_lg = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 28)
    font_md = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 18)
    font_sm = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
    font_xs = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 11)
    font_bold = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 18)
    font_title = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 22)
except:
    font_lg = ImageFont.load_default()
    font_md = font_sm = font_xs = font_bold = font_title = font_lg


def rounded_rect(draw, xy, fill, radius=12, outline=None):
    x0, y0, x1, y1 = xy
    draw.rounded_rectangle(xy, radius=radius, fill=fill, outline=outline)


def draw_navbar(draw, title="ParkHub", active=None):
    """Draw top navbar."""
    rounded_rect(draw, (0, 0, W, 56), CARD_BG)
    draw.rounded_rectangle((16, 10, 52, 46), radius=8, fill=BLUE)
    draw.text((22, 14), "PH", fill=WHITE, font=font_bold)
    draw.text((62, 16), title, fill=WHITE, font=font_bold)
    items = ["Dashboard", "Buchungen", "Profil", "Admin"]
    x = 300
    for item in items:
        color = BLUE if item == active else GRAY
        draw.text((x, 18), item, fill=color, font=font_sm)
        x += 100


def draw_sidebar(draw, active=None):
    """Draw left sidebar."""
    rounded_rect(draw, (0, 56, 200, H), (20, 30, 48))
    items = [("Dashboard", 80), ("Buchungen", 115), ("Fahrzeuge", 150), 
             ("Homeoffice", 185), ("Profil", 220), ("Admin", 270)]
    for label, y in items:
        if label == active:
            rounded_rect(draw, (8, y-4, 192, y+26), BLUE, radius=8)
            draw.text((20, y), label, fill=WHITE, font=font_sm)
        else:
            draw.text((20, y), label, fill=GRAY, font=font_sm)


def create_welcome():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw.text((W//2 - 100, 60), "Willkommen", fill=BLUE, font=font_lg)
    draw.text((W//2 - 140, 110), "Sprache wahlen / Select your language", fill=GRAY, font=font_sm)
    langs = [("Deutsch", "German"), ("English", "English"), ("Espanol", "Spanish"),
             ("Francais", "French"), ("Portugues", "Portuguese"),
             ("Turkce", "Turkish"), ("Hindi", "Hindi"), ("Japanisch", "Japanese"),
             ("Chinesisch", "Chinese"), ("Arabisch", "Arabic")]
    cols, cw, ch = 5, 130, 70
    sx = (W - cols * cw - (cols-1)*12) // 2
    for i, (native, name) in enumerate(langs):
        r, c = divmod(i, cols)
        x = sx + c * (cw + 12)
        y = 160 + r * (ch + 12)
        rounded_rect(draw, (x, y, x+cw, y+ch), CARD_BG, radius=10, outline=CARD_BORDER)
        draw.text((x+12, y+14), native, fill=WHITE, font=font_sm)
        draw.text((x+12, y+36), name, fill=GRAY, font=font_xs)
    draw.text((W//2 - 80, 390), "Barrierefreiheit", fill=GRAY, font=font_xs)
    draw.text((W//2 - 100, 460), "Open Source  |  MIT License  |  GitHub", fill=(71,85,105), font=font_xs)
    img.save(os.path.join(OUT_DIR, 'welcome.png'))


def create_login():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    # Left panel
    draw.rectangle((0, 0, 380, H), fill=BLUE)
    draw.text((40, 60), "PH", fill=WHITE, font=font_lg)
    draw.text((90, 68), "ParkHub", fill=WHITE, font=font_bold)
    draw.text((40, 140), "Intelligentes", fill=WHITE, font=font_lg)
    draw.text((40, 175), "Parkplatz-", fill=WHITE, font=font_lg)
    draw.text((40, 210), "Management", fill=WHITE, font=font_lg)
    draw.text((40, 260), "Einfach. Effizient. Open Source.", fill=(200,220,255), font=font_sm)
    rounded_rect(draw, (40, 320, 170, 380), (255,255,255,30), radius=12)
    draw.text((60, 335), "24/7", fill=WHITE, font=font_bold)
    draw.text((60, 358), "Verfugbar", fill=(200,220,255), font=font_xs)
    rounded_rect(draw, (190, 320, 340, 380), (255,255,255,30), radius=12)
    draw.text((210, 335), "100%", fill=WHITE, font=font_bold)
    draw.text((210, 358), "Open Source", fill=(200,220,255), font=font_xs)
    # Right panel - form
    draw.text((420, 100), "Anmeldung", fill=WHITE, font=font_title)
    draw.text((420, 135), "Melden Sie sich an", fill=GRAY, font=font_sm)
    draw.text((420, 180), "Benutzername", fill=GRAY, font=font_xs)
    rounded_rect(draw, (420, 200, 740, 236), CARD_BG, radius=8, outline=CARD_BORDER)
    draw.text((435, 210), "admin", fill=LIGHT_GRAY, font=font_sm)
    draw.text((420, 260), "Passwort", fill=GRAY, font=font_xs)
    rounded_rect(draw, (420, 280, 740, 316), CARD_BG, radius=8, outline=CARD_BORDER)
    draw.text((435, 290), "••••••••", fill=LIGHT_GRAY, font=font_sm)
    rounded_rect(draw, (420, 340, 740, 376), BLUE, radius=8)
    draw.text((545, 350), "Anmelden", fill=WHITE, font=font_bold)
    img.save(os.path.join(OUT_DIR, 'login.png'))


def create_onboarding():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw.text((W//2 - 100, 30), "Ersteinrichtung", fill=WHITE, font=font_lg)
    # Progress bar
    rounded_rect(draw, (100, 80, 700, 90), CARD_BG, radius=5)
    rounded_rect(draw, (100, 80, 400, 90), BLUE, radius=5)
    steps = ["Passwort", "Anwendungsfall", "Organisation", "Fertig"]
    for i, s in enumerate(steps):
        x = 130 + i * 160
        color = BLUE if i < 2 else GRAY
        draw.ellipse((x, 96, x+20, 116), fill=color)
        draw.text((x-10, 122), s, fill=color, font=font_xs)
    # Content
    rounded_rect(draw, (100, 160, 700, 440), CARD_BG, radius=12, outline=CARD_BORDER)
    draw.text((130, 180), "Anwendungsfall wahlen", fill=WHITE, font=font_title)
    draw.text((130, 215), "Wie mochten Sie ParkHub nutzen?", fill=GRAY, font=font_sm)
    modes = [("Unternehmen", "Firmenparkplatze verwalten"),
             ("Wohnanlage", "Mieterparkplatze organisieren"),
             ("Familie", "Familienparkplatze teilen")]
    for i, (title, desc) in enumerate(modes):
        y = 260 + i * 55
        is_sel = i == 0
        bg = (59, 130, 246, 30) if is_sel else CARD_BG
        ol = BLUE if is_sel else CARD_BORDER
        rounded_rect(draw, (130, y, 670, y+45), bg if not is_sel else (30, 50, 80), radius=8, outline=ol)
        draw.text((155, y+6), title, fill=WHITE if is_sel else LIGHT_GRAY, font=font_bold)
        draw.text((155, y+26), desc, fill=GRAY, font=font_xs)
    rounded_rect(draw, (550, 450, 700, 480), BLUE, radius=8)
    draw.text((595, 455), "Weiter", fill=WHITE, font=font_bold)
    img.save(os.path.join(OUT_DIR, 'onboarding.png'))


def create_dashboard():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw_navbar(draw, active="Dashboard")
    draw_sidebar(draw, active="Dashboard")
    # Stats cards
    stats = [("Stellplatze", "24", GREEN), ("Belegt", "18", RED), 
             ("Frei", "6", BLUE), ("Buchungen", "142", PURPLE)]
    for i, (label, val, color) in enumerate(stats):
        x = 220 + i * 145
        rounded_rect(draw, (x, 72, x+132, 142), CARD_BG, radius=10, outline=CARD_BORDER)
        draw.text((x+12, 82), label, fill=GRAY, font=font_xs)
        draw.text((x+12, 102), val, fill=color, font=font_lg)
    # Parking grid
    rounded_rect(draw, (220, 158, 560, 440), CARD_BG, radius=10, outline=CARD_BORDER)
    draw.text((236, 168), "Parkplatzbelegung", fill=WHITE, font=font_bold)
    for r in range(4):
        for c in range(6):
            x = 240 + c * 50
            y = 205 + r * 55
            occupied = (r * 6 + c) < 18
            color = (239, 68, 68, 100) if occupied else (34, 197, 94, 100)
            rounded_rect(draw, (x, y, x+40, y+42), color if occupied else (30, 60, 40), radius=6, 
                        outline=RED if occupied else GREEN)
            draw.text((x+10, y+14), f"P{r*6+c+1:02d}", fill=WHITE if occupied else GREEN, font=font_xs)
    # Recent bookings
    rounded_rect(draw, (575, 158, 785, 440), CARD_BG, radius=10, outline=CARD_BORDER)
    draw.text((591, 168), "Letzte Buchungen", fill=WHITE, font=font_bold)
    bookings = [("Max M.", "P03", "Aktiv"), ("Anna S.", "P12", "Aktiv"), 
                ("Tom K.", "P07", "Beendet"), ("Lisa R.", "P21", "Geplant")]
    for i, (name, slot, status) in enumerate(bookings):
        y = 200 + i * 55
        draw.text((591, y), name, fill=WHITE, font=font_sm)
        draw.text((591, y+20), slot, fill=GRAY, font=font_xs)
        sc = GREEN if status == "Aktiv" else (GRAY if status == "Beendet" else BLUE)
        draw.text((710, y+5), status, fill=sc, font=font_xs)
    img.save(os.path.join(OUT_DIR, 'dashboard.png'))


def create_booking():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw_navbar(draw, active="Buchungen")
    draw_sidebar(draw, active="Buchungen")
    # Main content
    draw.text((220, 72), "Stellplatz buchen", fill=WHITE, font=font_title)
    draw.text((220, 102), "Wahlen Sie einen freien Stellplatz", fill=GRAY, font=font_sm)
    # Date picker
    rounded_rect(draw, (220, 135, 500, 175), CARD_BG, radius=8, outline=CARD_BORDER)
    draw.text((235, 147), "Datum:  08.02.2026", fill=LIGHT_GRAY, font=font_sm)
    rounded_rect(draw, (515, 135, 700, 175), CARD_BG, radius=8, outline=CARD_BORDER)
    draw.text((530, 147), "08:00 - 18:00", fill=LIGHT_GRAY, font=font_sm)
    # Parking map
    rounded_rect(draw, (220, 190, 785, 440), CARD_BG, radius=10, outline=CARD_BORDER)
    draw.text((236, 200), "Parkplatz A - Tiefgarage", fill=WHITE, font=font_bold)
    for r in range(3):
        for c in range(8):
            x = 240 + c * 65
            y = 240 + r * 60
            idx = r * 8 + c
            if idx in [2, 5, 9, 14, 18, 20]:
                color_fill = (20, 50, 30)
                color_border = GREEN
                label_color = GREEN
            elif idx == 7:
                color_fill = (30, 50, 80)
                color_border = BLUE
                label_color = BLUE
            else:
                color_fill = (50, 30, 30)
                color_border = RED
                label_color = RED
            rounded_rect(draw, (x, y, x+55, y+48), color_fill, radius=6, outline=color_border)
            draw.text((x+12, y+16), f"A{idx+1:02d}", fill=label_color, font=font_sm)
    # Selected slot info
    rounded_rect(draw, (240, 430, 500, 460), BLUE, radius=8)
    draw.text((260, 435), "A08 buchen - Bestatigen", fill=WHITE, font=font_sm)
    img.save(os.path.join(OUT_DIR, 'booking.png'))


def create_admin():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw_navbar(draw, active="Admin")
    draw_sidebar(draw, active="Admin")
    draw.text((220, 72), "Administration", fill=WHITE, font=font_title)
    # Tabs
    tabs = ["Benutzer", "Stellplatze", "Berichte", "Branding", "System"]
    x = 220
    for tab in tabs:
        is_active = tab == "Benutzer"
        if is_active:
            rounded_rect(draw, (x, 108, x+90, 134), BLUE, radius=6)
        draw.text((x+10, 112), tab, fill=WHITE if is_active else GRAY, font=font_sm)
        x += 100
    # Users table
    rounded_rect(draw, (220, 150, 785, 440), CARD_BG, radius=10, outline=CARD_BORDER)
    draw.text((236, 160), "Benutzer (47)", fill=WHITE, font=font_bold)
    # Header row
    headers = ["Name", "E-Mail", "Rolle", "Status"]
    hx = [240, 380, 540, 660]
    for i, h in enumerate(headers):
        draw.text((hx[i], 195), h, fill=GRAY, font=font_xs)
    draw.line((236, 215, 770, 215), fill=CARD_BORDER)
    users = [("Max Mustermann", "max@firma.de", "Benutzer", "Aktiv"),
             ("Anna Schmidt", "anna@firma.de", "Admin", "Aktiv"),
             ("Tom Krause", "tom@firma.de", "Benutzer", "Inaktiv"),
             ("Lisa Richter", "lisa@firma.de", "Benutzer", "Aktiv"),
             ("Jan Weber", "jan@firma.de", "Manager", "Aktiv")]
    for i, (name, email, role, status) in enumerate(users):
        y = 225 + i * 38
        draw.text((240, y), name, fill=WHITE, font=font_sm)
        draw.text((380, y), email, fill=GRAY, font=font_sm)
        rc = PURPLE if role == "Admin" else (BLUE if role == "Manager" else GRAY)
        draw.text((540, y), role, fill=rc, font=font_sm)
        sc = GREEN if status == "Aktiv" else RED
        draw.text((660, y), status, fill=sc, font=font_sm)
        if i < 4:
            draw.line((236, y+30, 770, y+30), fill=(30, 41, 59))
    img.save(os.path.join(OUT_DIR, 'admin.png'))


def create_themes():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    draw_navbar(draw)
    draw.text((W//2 - 80, 72), "Farbthemen", fill=WHITE, font=font_title)
    draw.text((W//2 - 140, 102), "Wahlen Sie Ihr bevorzugtes Erscheinungsbild", fill=GRAY, font=font_sm)
    themes = [
        ("Default Blue", (59, 130, 246)), ("Solarized", (181, 137, 0)),
        ("Dracula", (189, 147, 249)), ("Nord", (136, 192, 208)),
        ("Gruvbox", (214, 153, 33)), ("Catppuccin", (203, 166, 247)),
        ("Tokyo Night", (122, 162, 247)), ("One Dark", (97, 175, 239)),
        ("Rose Pine", (235, 188, 186)), ("Everforest", (167, 192, 128)),
    ]
    for i, (name, color) in enumerate(themes):
        r, c = divmod(i, 5)
        x = 60 + c * 145
        y = 145 + r * 170
        is_sel = i == 0
        ol = BLUE if is_sel else CARD_BORDER
        rounded_rect(draw, (x, y, x+130, y+150), CARD_BG, radius=10, outline=ol)
        # Color preview
        rounded_rect(draw, (x+10, y+10, x+120, y+90), (color[0]//4, color[1]//4, color[2]//4), radius=6)
        draw.ellipse((x+50, y+35, x+80, y+65), fill=color)
        draw.text((x+10, y+100), name, fill=WHITE if is_sel else LIGHT_GRAY, font=font_sm)
        if is_sel:
            draw.text((x+10, y+125), "Aktiv", fill=BLUE, font=font_xs)
    img.save(os.path.join(OUT_DIR, 'themes.png'))


def create_mobile():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    # Phone frame
    px, py, pw, ph = 280, 20, 240, 460
    rounded_rect(draw, (px, py, px+pw, py+ph), (10, 15, 30), radius=24, outline=CARD_BORDER)
    # Status bar
    draw.text((px+15, py+10), "9:41", fill=WHITE, font=font_xs)
    # Navbar
    rounded_rect(draw, (px+8, py+28, px+pw-8, py+68), CARD_BG, radius=10)
    draw.rounded_rectangle((px+16, py+35, px+46, py+60), radius=6, fill=BLUE)
    draw.text((px+20, py+40), "PH", fill=WHITE, font=font_sm)
    draw.text((px+54, py+42), "ParkHub", fill=WHITE, font=font_sm)
    # Stats
    for i, (val, label) in enumerate([("6", "Frei"), ("18", "Belegt")]):
        x = px + 20 + i * 110
        rounded_rect(draw, (x, py+78, x+95, py+128), CARD_BG, radius=8, outline=CARD_BORDER)
        draw.text((x+10, py+84), val, fill=BLUE, font=font_bold)
        draw.text((x+10, py+106), label, fill=GRAY, font=font_xs)
    # Parking slots
    draw.text((px+16, py+140), "Stellplatze", fill=WHITE, font=font_sm)
    for r in range(3):
        for c in range(3):
            x = px + 16 + c * 72
            y = py + 165 + r * 52
            occ = (r * 3 + c) < 5
            rounded_rect(draw, (x, y, x+64, y+42), (50,30,30) if occ else (20,50,30), radius=6,
                         outline=RED if occ else GREEN)
            draw.text((x+18, y+14), f"P{r*3+c+1}", fill=RED if occ else GREEN, font=font_xs)
    # Bottom nav
    rounded_rect(draw, (px+8, py+ph-52, px+pw-8, py+ph-8), CARD_BG, radius=10)
    nav_items = ["Home", "Buchen", "Profil"]
    for i, item in enumerate(nav_items):
        x = px + 30 + i * 75
        color = BLUE if i == 0 else GRAY
        draw.text((x, py+ph-38), item, fill=color, font=font_xs)
    # Labels
    draw.text((60, 100), "PWA-fahig", fill=WHITE, font=font_title)
    draw.text((60, 135), "Installierbar auf", fill=GRAY, font=font_sm)
    draw.text((60, 158), "jedem Gerat", fill=GRAY, font=font_sm)
    draw.text((560, 100), "Responsive", fill=WHITE, font=font_title)
    draw.text((560, 135), "Optimiert fur", fill=GRAY, font=font_sm)
    draw.text((560, 158), "alle Bildschirme", fill=GRAY, font=font_sm)
    img.save(os.path.join(OUT_DIR, 'mobile.png'))


def create_dark_mode():
    img = Image.new('RGB', (W, H), BG)
    draw = ImageDraw.Draw(img)
    # Split view - dark left, light right
    mid = W // 2
    # Dark side
    rounded_rect(draw, (30, 30, mid-15, H-30), (15, 23, 42), radius=12, outline=CARD_BORDER)
    draw.text((50, 45), "Dark Mode", fill=WHITE, font=font_bold)
    rounded_rect(draw, (50, 80, mid-35, 140), CARD_BG, radius=8, outline=CARD_BORDER)
    draw.text((65, 90), "Dashboard", fill=WHITE, font=font_sm)
    draw.text((65, 112), "18 von 24 belegt", fill=GRAY, font=font_xs)
    for i in range(3):
        y = 155 + i * 50
        rounded_rect(draw, (50, y, mid-35, y+40), CARD_BG, radius=8, outline=CARD_BORDER)
        draw.text((65, y+6), f"Buchung #{i+1}", fill=LIGHT_GRAY, font=font_sm)
        draw.text((65, y+24), "Stellplatz P0" + str(i+1), fill=GRAY, font=font_xs)
    rounded_rect(draw, (50, 320, mid-35, 380), CARD_BG, radius=8, outline=CARD_BORDER)
    for j in range(4):
        x = 60 + j * 80
        c = [GREEN, RED, BLUE, PURPLE][j]
        rounded_rect(draw, (x, 330, x+65, 370), (c[0]//6, c[1]//6, c[2]//6), radius=6, outline=c)
    # Moon icon
    draw.text((50, 400), "Automatische Erkennung", fill=GRAY, font=font_xs)
    # Light side
    rounded_rect(draw, (mid+15, 30, W-30, H-30), (248, 250, 252), radius=12, outline=(229,231,235))
    draw.text((mid+35, 45), "Light Mode", fill=(30,41,59), font=font_bold)
    rounded_rect(draw, (mid+35, 80, W-50, 140), WHITE, radius=8, outline=(229,231,235))
    draw.text((mid+50, 90), "Dashboard", fill=(30,41,59), font=font_sm)
    draw.text((mid+50, 112), "18 von 24 belegt", fill=(107,114,128), font=font_xs)
    for i in range(3):
        y = 155 + i * 50
        rounded_rect(draw, (mid+35, y, W-50, y+40), WHITE, radius=8, outline=(229,231,235))
        draw.text((mid+50, y+6), f"Buchung #{i+1}", fill=(30,41,59), font=font_sm)
        draw.text((mid+50, y+24), "Stellplatz P0" + str(i+1), fill=(107,114,128), font=font_xs)
    rounded_rect(draw, (mid+35, 320, W-50, 380), WHITE, radius=8, outline=(229,231,235))
    for j in range(4):
        x = mid + 45 + j * 80
        c = [GREEN, RED, BLUE, PURPLE][j]
        rounded_rect(draw, (x, 330, x+65, 370), (c[0]//2+128, c[1]//2+128, c[2]//2+128), radius=6, outline=c)
    draw.text((mid+35, 400), "Systemeinstellung", fill=(107,114,128), font=font_xs)
    img.save(os.path.join(OUT_DIR, 'dark-mode.png'))


if __name__ == '__main__':
    print("Generating screenshots...")
    create_welcome()
    print("  welcome.png")
    create_login()
    print("  login.png")
    create_onboarding()
    print("  onboarding.png")
    create_dashboard()
    print("  dashboard.png")
    create_booking()
    print("  booking.png")
    create_admin()
    print("  admin.png")
    create_themes()
    print("  themes.png")
    create_mobile()
    print("  mobile.png")
    create_dark_mode()
    print("  dark-mode.png")
    print("Done! All screenshots saved to", OUT_DIR)
