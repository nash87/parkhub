export const de = {
  // Navigation
  nav: {
    dashboard: 'Dashboard',
    book: 'Buchen',
    bookings: 'Buchungen',
    vehicles: 'Fahrzeuge',
    homeoffice: 'Homeoffice',
    admin: 'Admin',
    more: 'Mehr',
    profile: 'Profil',
    settings: 'Einstellungen',
    logout: 'Abmelden',
  },

  // Common
  common: {
    save: 'Speichern',
    cancel: 'Abbrechen',
    delete: 'Löschen',
    edit: 'Bearbeiten',
    add: 'Hinzufügen',
    close: 'Schließen',
    confirm: 'Bestätigen',
    back: 'Zurück',
    next: 'Weiter',
    loading: 'Laden...',
    search: 'Suchen',
    filter: 'Filter',
    all: 'Alle',
    none: 'Keine',
    yes: 'Ja',
    no: 'Nein',
    refresh: 'Aktualisieren',
    available: 'verfügbar',
    selected: 'ausgewählt',
    free: 'frei',
    full: 'Voll',
    online: 'Online',
    active: 'Aktiv',
    blocked: 'Gesperrt',
  },

  // Theme
  theme: {
    toggle: 'Theme umschalten',
  },

  // Notifications
  notifications: {
    title: 'Benachrichtigungen',
    empty: 'Keine Benachrichtigungen',
    timeAgo: 'vor 5 Min',
    bookingExpiring: 'Ihre Buchung für Stellplatz {{slot}} läuft in 30 Min ab',
    newLotAvailable: "Neuer Parkplatz '{{name}}' verfügbar",
    slotFreed: 'Stellplatz {{slot}} wurde für Sie freigegeben (Homeoffice)',
  },

  // Login
  login: {
    title: 'Willkommen zurück',
    subtitle: 'Melden Sie sich an, um fortzufahren',
    username: 'Benutzername',
    usernamePlaceholder: 'Ihr Benutzername',
    password: 'Passwort',
    rememberMe: 'Angemeldet bleiben',
    forgotPassword: 'Passwort vergessen?',
    submit: 'Anmelden',
    noAccount: 'Noch kein Konto?',
    register: 'Jetzt registrieren',
    welcomeBack: 'Willkommen zurück!',
    invalidCredentials: 'Ungültige Anmeldedaten',
    heroTitle: 'Intelligentes Parkplatz-Management',
    heroSubtitle: 'Verwalten Sie Parkplätze effizient. Buchen Sie mit wenigen Klicks. Behalten Sie die Übersicht.',
    available247: 'Verfügbar',
    openSource: 'Open Source',
  },

  // Register
  register: {
    title: 'Konto erstellen',
    subtitle: 'Füllen Sie das Formular aus, um loszulegen',
    fullName: 'Vollständiger Name',
    fullNamePlaceholder: 'Max Mustermann',
    username: 'Benutzername',
    usernamePlaceholder: 'maxmuster',
    email: 'E-Mail',
    emailPlaceholder: 'max@beispiel.de',
    password: 'Passwort',
    confirmPassword: 'Passwort bestätigen',
    submit: 'Registrieren',
    alreadyRegistered: 'Bereits registriert?',
    loginLink: 'Jetzt anmelden',
    heroTitle: 'Werden Sie Teil von ParkHub',
    heroSubtitle: 'Erstellen Sie Ihr Konto und beginnen Sie noch heute mit der intelligenten Parkplatzverwaltung.',
    welcomeMessage: 'Willkommen bei ParkHub!',
    failed: 'Registrierung fehlgeschlagen',
    passwordMismatch: 'Passwörter stimmen nicht überein',
    passwordTooShort: 'Passwort muss mindestens 8 Zeichen haben',
  },

  // Dashboard
  dashboard: {
    welcome: 'Willkommen, {{name}}',
    availableSlots: 'Verfügbare Plätze',
    occupancy: 'Auslastung',
    normal: 'Normal',
    activeBookings: 'Aktive Buchungen',
    viewAll: 'Alle anzeigen',
    inOffice: 'Im Büro',
    homeofficeToday: 'Heute Homeoffice — Ihr Stellplatz {{slot}} ist für Kollegen freigegeben',
    quickBooking: 'Schnellbuchung',
    quickBookingSubtitle: 'Häufig genutzte Stellplätze',
    slot: 'Stellplatz',
    lotOverview: 'Parkplatz-Übersicht',
    parkingLots: 'Parkplätze',
    bookNowTitle: 'Jetzt Parkplatz buchen',
    bookNowSubtitle: 'Wählen Sie einen freien Platz für heute oder die kommenden Tage',
    bookNow: 'Jetzt buchen',
    until: 'Bis {{time}} Uhr',
    noPlate: 'Kein Kennzeichen',
  },

  // Book page
  book: {
    title: 'Parkplatz buchen',
    subtitle: 'Wählen Sie Ihren Stellplatz und die gewünschte Dauer',
    step1: 'Parkplatz wählen',
    step2: 'Stellplatz wählen',
    step3: 'Dauer & Fahrzeug',
    slotSelected: 'Stellplatz <strong>{{slot}}</strong> ausgewählt',
    ofAvailable: '{{available}} von {{total}} verfügbar',
    bookingType: 'Buchungsart',
    single: 'Einmalig',
    multiDay: 'Mehrtägig',
    recurring: 'Dauerbuchung',
    duration: 'Parkdauer',
    min30: '30 Min',
    hour1: '1 Std',
    hour2: '2 Std',
    hour4: '4 Std',
    hour8: '8 Std',
    hour12: '12 Std',
    untilTime: 'Bis {{time}} Uhr ({{day}})',
    period: 'Zeitraum',
    startDate: 'Startdatum',
    endDate: 'Enddatum',
    days: '{{count}} Tage',
    dateRange: '{{start}} bis {{end}}',
    interval: 'Intervall',
    weekly: 'Wöchentlich',
    monthly: 'Monatlich',
    weekdays: 'Wochentage',
    vehicle: 'Fahrzeug',
    otherPlate: 'Anderes Kennzeichen eingeben',
    enterPlate: 'Kennzeichen eingeben (z.B. M-AB 1234)',
    summary: 'Zusammenfassung',
    parkingLot: 'Parkplatz',
    parkingSlot: 'Stellplatz',
    type: 'Buchungsart',
    durationLabel: 'Dauer',
    periodLabel: 'Zeitraum',
    intervalLabel: 'Intervall',
    licensePlate: 'Kennzeichen',
    bookNow: 'Jetzt buchen',
    bookingFailed: 'Buchung fehlgeschlagen',
    weeklyShort: 'Wöchentl.',
  },

  // Booking success
  bookingSuccess: {
    title: 'Buchung erfolgreich!',
    subtitle: 'Ihr Parkplatz wurde reserviert',
    parkingLot: 'Parkplatz',
    slot: 'Stellplatz',
    type: 'Typ',
    time: 'Zeit',
    plate: 'Kennzeichen',
    newBooking: 'Weitere Buchung',
    toDashboard: 'Zum Dashboard',
  },

  // Bookings page
  bookings: {
    title: 'Meine Buchungen',
    subtitle: 'Übersicht Ihrer Parkplatz-Buchungen',
    active: 'Aktive Buchungen',
    upcoming: 'Anstehende Buchungen',
    past: 'Vergangene Buchungen',
    noActive: 'Keine aktiven Buchungen',
    noUpcoming: 'Keine anstehenden Buchungen',
    noPast: 'Noch keine vergangenen Buchungen',
    extend: 'Verlängern',
    cancelBtn: 'Stornieren',
    endsIn: 'Endet {{time}}',
    startsIn: 'Beginnt {{time}}',
    bookNow: 'Jetzt buchen',
    cancelled: 'Buchung storniert',
    cancelFailed: 'Stornierung fehlgeschlagen',
    statusActive: 'Aktiv',
    statusCompleted: 'Abgeschlossen',
    statusCancelled: 'Storniert',
    typeSingle: 'Einmalig',
    typeMultiDay: 'Mehrtägig',
    typeRecurring: 'Dauer',
    weekly: 'Wöchentlich',
    monthly: 'Monatlich',
  },

  // Confirm dialogs
  confirm: {
    cancelBookingTitle: 'Buchung wirklich stornieren?',
    cancelBookingMessage: 'Diese Aktion kann nicht rückgängig gemacht werden. Der Stellplatz wird für andere Benutzer freigegeben.',
    cancelBookingConfirm: 'Stornieren',
    deleteVehicleTitle: 'Fahrzeug wirklich löschen?',
    deleteVehicleMessage: 'Das Fahrzeug wird aus Ihrem Konto entfernt. Bestehende Buchungen bleiben erhalten.',
    deleteVehicleConfirm: 'Löschen',
  },

  // Vehicles
  vehicles: {
    title: 'Meine Fahrzeuge',
    subtitle: 'Verwalten Sie Ihre registrierten Fahrzeuge',
    add: 'Hinzufügen',
    addVehicle: 'Fahrzeug hinzufügen',
    newVehicle: 'Neues Fahrzeug',
    plate: 'Kennzeichen',
    platePlaceholder: 'M-AB 1234',
    make: 'Marke',
    makePlaceholder: 'BMW',
    model: 'Modell',
    modelPlaceholder: '320i',
    color: 'Farbe',
    colorSelect: '— Auswählen —',
    defaultVehicle: 'Als Standard-Fahrzeug setzen',
    defaultVehicleDesc: 'Wird automatisch bei Buchungen ausgewählt',
    isDefault: 'Standard-Fahrzeug',
    noVehicles: 'Keine Fahrzeuge registriert',
    noVehiclesDesc: 'Fügen Sie Ihr erstes Fahrzeug hinzu, um schneller buchen zu können',
    added: 'Fahrzeug hinzugefügt',
    removed: 'Fahrzeug entfernt',
    uploadPhoto: 'Foto hochladen',
    uploadPhotoOrCamera: 'Foto hochladen oder Kamera verwenden',
  },

  // Colors
  colors: {
    black: 'Schwarz',
    white: 'Weiß',
    silver: 'Silber',
    blue: 'Blau',
    red: 'Rot',
    green: 'Grün',
    gray: 'Grau',
    yellow: 'Gelb',
    other: 'Sonstige',
  },

  // Homeoffice
  homeoffice: {
    title: 'Homeoffice-Verwaltung',
    subtitle: 'Verwalten Sie Ihre Homeoffice-Tage und geben Sie Ihren Parkplatz für Kollegen frei.',
    todayBannerTitle: 'Heute ist Homeoffice-Tag',
    todayBannerDesc: 'Ihr Stellplatz {{slot}} ist für Kollegen freigegeben.',
    thisWeek: 'Diese Woche',
    homeOfficeDays: 'Homeoffice-Tage',
    yourParkingSlot: 'Ihr Parkplatz',
    slotAvailableOnHo: 'ist an HO-Tagen für Kollegen verfügbar',
    regularDays: 'Regelmäßige Homeoffice-Tage',
    regularDaysDesc: 'Wählen Sie die Wochentage, an denen Sie regelmäßig im Homeoffice arbeiten.',
    singleDays: 'Einzelne Homeoffice-Tage',
    nextWeekComplete: 'Nächste Woche komplett',
    noSingleDays: 'Keine einzelnen HO-Tage geplant',
    patternUpdated: 'Homeoffice-Muster aktualisiert',
    dayAdded: 'Homeoffice-Tag hinzugefügt',
    dayRemoved: 'Homeoffice-Tag entfernt',
    nextWeekMarked: 'Nächste Woche als Homeoffice markiert',
    legendRegular: 'Regelmäßig',
    legendSingle: 'Einzeltag',
    legendToday: 'Heute',
    weekdays: {
      mon: 'Montag',
      tue: 'Dienstag',
      wed: 'Mittwoch',
      thu: 'Donnerstag',
      fri: 'Freitag',
    },
    weekdaysShort: {
      mon: 'Mo',
      tue: 'Di',
      wed: 'Mi',
      thu: 'Do',
      fri: 'Fr',
      sat: 'Sa',
      sun: 'So',
    },
  },

  // Admin
  admin: {
    title: 'Administration',
    subtitle: 'System- und Benutzerverwaltung',
    tabs: {
      overview: 'Übersicht',
      lots: 'Parkplätze',
      users: 'Benutzer',
      bookings: 'Buchungen',
    },
    overview: {
      title: 'System-Übersicht',
      totalSlots: 'Gesamte Parkplätze',
      activeBookings: 'Aktive Buchungen',
      occupancyToday: 'Auslastung heute',
      homeofficeToday: 'Homeoffice heute',
      recentActivity: 'Letzte Aktivitäten',
      quickActions: 'Schnellaktionen',
      systemStatus: 'Systemstatus',
      blockSlot: 'Parkplatz sperren',
      manageUsers: 'Benutzer verwalten',
      cancelBooking: 'Buchung stornieren',
      backendApi: 'Backend API',
      database: 'Datenbank',
      authService: 'Auth Service',
    },
    lots: {
      title: 'Parkplätze verwalten',
      newLot: 'Neuer Parkplatz',
      createLot: 'Neuen Parkplatz anlegen',
      edit: 'Bearbeiten',
    },
    users: {
      title: 'Benutzer verwalten',
      addUser: 'Benutzer hinzufügen',
      searchPlaceholder: 'Name oder E-Mail suchen...',
      allRoles: 'Alle Rollen',
      name: 'Name',
      email: 'E-Mail',
      role: 'Rolle',
      vehiclesCol: 'Fahrzeuge',
      status: 'Status',
      actions: 'Aktionen',
      noUsers: 'Keine Benutzer gefunden',
    },
    bookings: {
      title: 'Alle Buchungen',
      allLots: 'Alle Parkplätze',
      allStatus: 'Alle Status',
      selected: '{{count}} ausgewählt',
      user: 'Benutzer',
      lot: 'Parkplatz',
      slot: 'Stellplatz',
      type: 'Typ',
      period: 'Zeitraum',
      status: 'Status',
      noBookings: 'Keine Buchungen gefunden',
    },
  },

  // Profile
  profile: {
    title: 'Mein Profil',
    subtitle: 'Ihre persönlichen Daten und Statistiken',
    name: 'Name',
    email: 'E-Mail',
    mySlot: 'Mein Stellplatz',
    fixedSlot: 'Fester Stellplatz · An HO-Tagen freigegeben',
    bookingsThisMonth: 'Buchungen diesen Monat',
    homeOfficeDays: 'Homeoffice-Tage',
    avgDuration: 'Durchschn. Parkdauer',
    updated: 'Profil aktualisiert',
    roles: {
      user: 'Benutzer',
      admin: 'Administrator',
      superadmin: 'Super-Admin',
    },
  },

  // Parking lot grid
  grid: {
    free: 'Frei',
    occupied: 'Belegt',
    reserved: 'Reserviert',
    disabled: 'Gesperrt',
    homeoffice: '🏠 Homeoffice (frei)',
    road: 'Fahrweg',
    occupiedBy: 'Belegt: {{plate}}',
    hoFreeFrom: 'Frei (Homeoffice von {{user}})',
  },

  // Footer
  footer: {
    tagline: 'ParkHub — Open Source Parking Management',
    help: 'Hilfe',
    about: 'Über',
    privacy: 'Datenschutz',
    terms: 'AGB',
    imprint: 'Impressum',
  },

  // PWA
  pwa: {
    installBanner: 'ParkHub als App installieren',
    install: 'Installieren',
    dismiss: 'Später',
  },

  // Day names short (for booking page)
  dayNamesShort: ['So', 'Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa'],

  // Barrierefreiheit
  accessibility: {
    title: 'Barrierefreiheit',
    colorMode: 'Farbmodus',
    colorModes: {
      normal: 'Normal',
      protanopia: 'Protanopie (Rotblind)',
      deuteranopia: 'Deuteranopie (Grünblind)',
      tritanopia: 'Tritanopie (Blaublind)',
    },
    fontScale: 'Schriftgröße',
    fontScales: {
      small: 'Klein',
      normal: 'Normal',
      large: 'Groß',
      xlarge: 'Sehr groß',
    },
    reducedMotion: 'Weniger Bewegung',
    reducedMotionDesc: 'Animationen und Übergänge deaktivieren',
    highContrast: 'Hoher Kontrast',
    highContrastDesc: 'Kontrast erhöhen für bessere Lesbarkeit',
  },

  // Zustimmungsbanner
  consent: {
    title: 'Datenschutz & Speicher-Zustimmung',
    message: 'ParkHub verwendet localStorage, um Ihre Einstellungen (Theme, Sprache, Barrierefreiheit) zu speichern. Es werden keine Cookies verwendet. Ihre Daten verbleiben auf Ihrem Gerät und unserem Server — keine Drittanbieter.',
    accept: 'Akzeptieren',
    decline: 'Ablehnen',
  },

  // DSGVO
  gdpr: {
    dataExport: 'Meine Daten exportieren',
    dataExportDesc: 'Alle persönlichen Daten als JSON herunterladen (DSGVO Art. 15)',
    deleteAccount: 'Konto löschen',
    deleteAccountDesc: 'Konto und alle Daten endgültig löschen (DSGVO Art. 17)',
    deleteConfirmTitle: 'Konto wirklich löschen?',
    deleteConfirmMessage: 'Dies löscht unwiderruflich Ihr Profil, alle Buchungen, Fahrzeuge und Einstellungen.',
    deleteConfirmBtn: 'Endgültig löschen',
    exporting: 'Daten werden exportiert...',
    exported: 'Daten erfolgreich exportiert',
  },

  // Datenschutz
  privacy: {
    title: 'Datenschutzerklärung',
    subtitle: 'Wie wir mit Ihren Daten umgehen — transparent und DSGVO-konform.',
    dataCollected: {
      title: 'Welche Daten wir erheben',
      content: 'Wir erheben nur die für die Parkplatzverwaltung notwendigen Daten:\n\n• Kontodaten (Name, E-Mail, Benutzername)\n• Fahrzeuginformationen (Kennzeichen, Marke, Modell)\n• Buchungshistorie (Stellplatz, Zeit, Dauer)\n• Homeoffice-Planung\n• Einstellungen (Theme, Sprache, Barrierefreiheit)\n\nKein Tracking, keine Analytics, keine Drittanbieter-Scripts.',
    },
    storage: {
      title: 'Datenspeicherung',
      content: 'Alle Daten werden in einer eingebetteten Datenbank (redb) direkt auf Ihrem Server gespeichert. ParkHub ist 100% selbst gehostet — Ihre Daten verlassen niemals Ihre Infrastruktur.\n\nKeine Cloud-Dienste, keine externen Datenbanken, keine Datenübertragungen an Dritte.',
    },
    security: {
      title: 'Sicherheit',
      content: 'Passwörter werden mit bcrypt gehasht. API-Zugriff wird über JWT-Tokens gesichert. Die Kommunikation sollte über HTTPS erfolgen.\n\nDie Datenbankdatei wird auf dem Server-Dateisystem gespeichert.',
    },
    access: {
      title: 'Zugriff',
      content: 'Nur Administratoren Ihrer ParkHub-Instanz haben Zugriff auf Benutzerdaten. Da ParkHub selbst gehostet wird, kontrolliert Ihre IT-Abteilung den gesamten Zugriff.',
    },
    rights: {
      title: 'Ihre Rechte (DSGVO)',
      access: 'Auskunftsrecht — Laden Sie alle Ihre Daten über die Profilseite herunter (Art. 15 DSGVO)',
      rectification: 'Recht auf Berichtigung — Bearbeiten Sie Ihre Profildaten jederzeit (Art. 16 DSGVO)',
      erasure: 'Recht auf Löschung — Löschen Sie Ihr Konto und alle Daten über die Profilseite (Art. 17 DSGVO)',
      portability: 'Recht auf Datenübertragbarkeit — Exportieren Sie Ihre Daten als JSON (Art. 20 DSGVO)',
    },
  },

  // AGB
  terms: {
    title: 'Nutzungsbedingungen',
    usage: {
      title: 'Nutzung',
      content: 'ParkHub wird zur Verwaltung von Parkplätzen in Ihrer Organisation bereitgestellt. Mit der Nutzung von ParkHub stimmen Sie zu, es verantwortungsvoll und gemäß Ihren Unternehmensrichtlinien zu nutzen.',
    },
    accounts: {
      title: 'Konten',
      content: 'Sie sind für die Sicherheit Ihrer Zugangsdaten verantwortlich. Teilen Sie Ihr Passwort nicht. Administratoren können Konten nach Bedarf erstellen, ändern oder löschen.',
    },
    liability: {
      title: 'Haftung',
      content: 'ParkHub wird "wie besehen" ohne jegliche Gewährleistung bereitgestellt. Die Software ist Open Source (MIT-Lizenz). Der Betreiber dieser Instanz ist für den ordnungsgemäßen Betrieb und Datenschutz verantwortlich.',
    },
    changes: {
      title: 'Änderungen',
      content: 'Diese Bedingungen können jederzeit aktualisiert werden. Die fortgesetzte Nutzung von ParkHub gilt als Zustimmung zu Änderungen.',
    },
  },

  // Impressum
  legal: {
    title: 'Impressum',
    content: '[Firmenname]\n[Straße]\n[PLZ, Ort]\n[Land]\n\nVertreten durch: [Name]\nE-Mail: [E-Mail]\nTelefon: [Telefon]\n\nVerantwortlich für den Inhalt nach § 55 Abs. 2 RStV:\n[Name, Adresse]\n\nBitte aktualisieren Sie diese Seite mit Ihren tatsächlichen Unternehmensdaten.',
  },

  // Über
  about: {
    title: 'Über ParkHub',
    subtitle: 'Open Source Parkplatzverwaltung für Ihre Organisation.',
    techStack: {
      title: 'Technologie',
      frontend: 'Frontend',
      backend: 'Backend',
    },
    architecture: {
      title: 'Architektur',
    },
    version: {
      title: 'Version',
      current: 'Version',
      license: 'Lizenz',
    },
    data: {
      title: 'Datentransparenz',
      content: 'Alle Daten werden lokal in einer einzelnen redb-Datenbankdatei auf Ihrem Server gespeichert.\n\nGespeicherte Daten: Benutzerkonten, Fahrzeuge, Buchungen, Homeoffice-Pläne, Parkplatzkonfigurationen.\n\nAufbewahrung: Daten werden aufbewahrt, solange das Konto besteht. Gelöschte Konten werden dauerhaft entfernt.\n\nVerschlüsselung: Passwörter sind bcrypt-gehasht. Die Datenbankdatei erbt die Server-Dateisystem-Verschlüsselung, falls konfiguriert.',
    },
  },

  // Hilfe
  help: {
    title: 'Hilfe & FAQ',
    subtitle: 'Antworten auf häufige Fragen.',
    searchPlaceholder: 'Hilfethemen durchsuchen...',
    userFaq: 'Allgemeine Fragen',
    adminFaq: 'Administrator-Fragen',
    faq: {
      bookSpot: {
        q: 'Wie buche ich einen Parkplatz?',
        a: 'Gehen Sie zu "Buchen" in der Navigation. Wählen Sie einen Parkplatz, einen freien Stellplatz (grün dargestellt), Ihre Buchungsdauer und Fahrzeug, dann bestätigen. Sie können einmalige, mehrtägige oder Dauerbuchungen vornehmen.',
      },
      homeOffice: {
        q: 'Wie richte ich Homeoffice-Tage ein?',
        a: 'Navigieren Sie zu "Homeoffice" im Menü. Sie können regelmäßige Wochenmuster (z.B. jeden Mittwoch und Freitag) oder einzelne Tage festlegen. Wenn Sie einen Tag als Homeoffice markieren, wird Ihr Stellplatz für Kollegen freigegeben.',
      },
      vehicles: {
        q: 'Wie verwalte ich meine Fahrzeuge?',
        a: 'Gehen Sie zu "Fahrzeuge", um Ihre registrierten Fahrzeuge hinzuzufügen, zu bearbeiten oder zu entfernen. Sie können ein Foto hochladen, ein Standardfahrzeug festlegen und Kennzeichen verwalten.',
      },
      recurring: {
        q: 'Wie funktionieren Dauerbuchungen?',
        a: 'Wählen Sie beim Buchen "Dauerbuchung" als Buchungsart. Wählen Sie wöchentliches oder monatliches Intervall und die gewünschten Wochentage. Das System reserviert den Stellplatz automatisch für alle passenden Termine im gewählten Zeitraum.',
      },
      waitlist: {
        q: 'Wie funktioniert die Warteliste?',
        a: 'Wenn alle Stellplätze belegt sind, können Sie sich auf die Warteliste setzen. Wenn ein Platz frei wird (z.B. jemand markiert einen Homeoffice-Tag), werden Sie automatisch benachrichtigt.',
      },
      checkin: {
        q: 'Wie funktioniert das Check-in?',
        a: 'Bei Ankunft am Parkplatz wird Ihre Buchung automatisch basierend auf der geplanten Zeit aktiviert. Wenn Check-in-Bestätigung von Ihrem Admin aktiviert ist, müssen Sie möglicherweise Ihre Ankunft in der App bestätigen.',
      },
      configureLots: {
        q: 'Wie konfiguriere ich Parkplätze?',
        a: 'Gehen Sie zu Admin → Parkplätze. Sie können neue Parkplätze erstellen, deren Layout bearbeiten (Reihen und Stellplätze hinzufügen), Labels setzen und Stellplatz-Eigenschaften konfigurieren.',
      },
      manageUsers: {
        q: 'Wie verwalte ich Benutzer?',
        a: 'Gehen Sie zu Admin → Benutzer. Sie können alle registrierten Benutzer einsehen, Rollen ändern (Benutzer/Admin), Konten sperren oder löschen. Bei aktivierter Selbstregistrierung können sich neue Benutzer selbst anmelden.',
      },
    },
  },

  // Einführung
  onboarding: {
    title: 'Willkommen bei ParkHub!',
    finish: 'Loslegen',
    steps: {
      password: {
        title: 'Konto sichern',
        desc: 'Ändern Sie das Standard-Admin-Passwort, um Ihre Instanz zu sichern.',
      },
      company: {
        title: 'Firmeneinstellungen',
        desc: 'Konfigurieren Sie Ihren Firmennamen und Grundeinstellungen.',
      },
      lot: {
        title: 'Parkplatz anlegen',
        desc: 'Erstellen Sie Ihren ersten Parkplatz mit Name und Adresse.',
      },
      slots: {
        title: 'Stellplätze hinzufügen',
        desc: 'Verwenden Sie den Layout-Editor, um Reihen und einzelne Stellplätze hinzuzufügen.',
      },
      users: {
        title: 'Benutzer hinzufügen',
        desc: 'Aktivieren Sie die Selbstregistrierung oder erstellen Sie Benutzerkonten manuell.',
      },
      done: {
        title: 'Fertig!',
        desc: 'Ihre ParkHub-Instanz ist bereit. Benutzer können jetzt Parkplätze buchen.',
      },
    },
  },

  // Grid ARIA
  gridAria: {
    available: 'Stellplatz {{number}}, verfügbar',
    occupied: 'Stellplatz {{number}}, belegt von {{plate}}',
    reserved: 'Stellplatz {{number}}, reserviert',
    disabled: 'Stellplatz {{number}}, gesperrt',
    homeoffice: 'Stellplatz {{number}}, verfügbar durch Homeoffice',
    blocked: 'Stellplatz {{number}}, blockiert',
  },
};
