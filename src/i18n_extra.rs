//! Words for the font panel (Preview, Sizes, Glyphs, Features, Info), in the same
//! language order as `i18n::LANGS`.

pub const N2: usize = 52;

pub const EXTRA: [&[&str; N2]; 25] = [
    &EN, &ES, &FR, &DE, &IT, &PT, &NL, &SV, &DA, &NB, &FI, &PL, &CS, &TR, &RO, &HU, &EL, &RU, &UK, &ID, &VI, &ZH_HANS,
    &ZH_HANT, &JA, &KO,
];

const EN: [&str; N2] = [
    "Preview", "Sizes", "Glyphs", "Features", "Info", "Uppercase", "Lowercase", "Numbers", "Punctuation",
    "Click a size to use it in the list", "{n} glyphs in this font", "Unicode", "Glyph name", "No character", "Not in this font",
    "{n} rules", "+{n} more", "This feature works through context rules, so there are no fixed examples to show.",
    "{n} sets available", "This font has no stylistic sets", "Set {n}",
    "Access All Alternates", "Shows every alternate shape the font has for a letter.",
    "Case-Sensitive Forms", "Moves punctuation and symbols to sit well with capital letters.",
    "Glyph Composition/Decomposition", "Joins or splits letters and accents so they show correctly.",
    "Standard Ligatures", "Replaces letter pairs such as fi or fl with one joined shape.",
    "Localized Forms", "Uses the letter shapes a particular language prefers.",
    "Medial Forms", "Shapes letters that sit in the middle of a word, as in Arabic.",
    "Stylistic Sets", "Groups of alternate letter designs made by the type designer.",
    "Name", "Family", "Style", "Format", "PostScript Name", "Unique Name", "Location", "Font Sets", "Font Variables", "Version",
    "Glyph Count", "Tags", "None", "Switches change every preview in FontCraft.", "Variable", "Bold", "default {n}",
];

const ES: [&str; N2] = [
    "Vista previa", "Tamaños", "Glifos", "Funciones", "Información", "Mayúsculas", "Minúsculas", "Números", "Puntuación",
    "Haz clic en un tamaño para usarlo en la lista", "{n} glifos en esta fuente", "Unicode", "Nombre del glifo", "Sin carácter", "No está en esta fuente",
    "{n} reglas", "+{n} más", "Esta función actúa mediante reglas de contexto, así que no hay ejemplos fijos que mostrar.",
    "{n} conjuntos disponibles", "Esta fuente no tiene conjuntos estilísticos", "Conjunto {n}",
    "Acceso a todas las alternativas", "Muestra todas las formas alternativas que la fuente tiene para una letra.",
    "Formas sensibles a mayúsculas", "Ajusta la puntuación y los símbolos para que encajen con las mayúsculas.",
    "Composición/descomposición de glifos", "Une o separa letras y acentos para que se muestren bien.",
    "Ligaduras estándar", "Sustituye pares como fi o fl por una sola forma unida.",
    "Formas localizadas", "Usa las formas de letra que prefiere un idioma concreto.",
    "Formas mediales", "Da forma a las letras en medio de una palabra, como en árabe.",
    "Conjuntos estilísticos", "Grupos de diseños alternativos creados por el diseñador tipográfico.",
    "Nombre", "Familia", "Estilo", "Formato", "Nombre PostScript", "Nombre único", "Ubicación", "Conjuntos de fuentes", "Variables de la fuente", "Versión",
    "Número de glifos", "Etiquetas", "Ninguno", "Los interruptores cambian todas las vistas previas de FontCraft.", "Variable", "Negrita", "predeterminado {n}",
];

const FR: [&str; N2] = [
    "Aperçu", "Tailles", "Glyphes", "Fonctions", "Infos", "Majuscules", "Minuscules", "Chiffres", "Ponctuation",
    "Cliquez sur une taille pour l'utiliser dans la liste", "{n} glyphes dans cette police", "Unicode", "Nom du glyphe", "Aucun caractère", "Absent de cette police",
    "{n} règles", "+{n} de plus", "Cette fonction agit par règles de contexte : aucun exemple fixe à montrer.",
    "{n} jeux disponibles", "Cette police n'a pas de jeux stylistiques", "Jeu {n}",
    "Accès à toutes les variantes", "Affiche toutes les formes alternatives d'une lettre.",
    "Formes sensibles à la casse", "Place la ponctuation et les symboles en accord avec les majuscules.",
    "Composition/décomposition de glyphes", "Assemble ou sépare lettres et accents pour un affichage correct.",
    "Ligatures standard", "Remplace des paires comme fi ou fl par une seule forme liée.",
    "Formes localisées", "Utilise les formes de lettres préférées d'une langue donnée.",
    "Formes médianes", "Façonne les lettres au milieu d'un mot, comme en arabe.",
    "Jeux stylistiques", "Groupes de dessins de lettres alternatifs créés par le dessinateur.",
    "Nom", "Famille", "Style", "Format", "Nom PostScript", "Nom unique", "Emplacement", "Ensembles de polices", "Variables de police", "Version",
    "Nombre de glyphes", "Étiquettes", "Aucun", "Les interrupteurs modifient tous les aperçus de FontCraft.", "Variable", "Gras", "par défaut {n}",
];

const DE: [&str; N2] = [
    "Vorschau", "Größen", "Glyphen", "Funktionen", "Info", "Großbuchstaben", "Kleinbuchstaben", "Ziffern", "Satzzeichen",
    "Klicke auf eine Größe, um sie in der Liste zu verwenden", "{n} Glyphen in dieser Schrift", "Unicode", "Glyphenname", "Kein Zeichen", "Nicht in dieser Schrift",
    "{n} Regeln", "+{n} weitere", "Diese Funktion wirkt über Kontextregeln, daher gibt es keine festen Beispiele.",
    "{n} Sets verfügbar", "Diese Schrift hat keine Stilsets", "Set {n}",
    "Zugriff auf alle Alternativen", "Zeigt alle alternativen Formen eines Buchstabens.",
    "Versalsensitive Formen", "Passt Satzzeichen und Symbole an Großbuchstaben an.",
    "Glyphen-Zusammensetzung/-Zerlegung", "Verbindet oder trennt Buchstaben und Akzente für die richtige Darstellung.",
    "Standardligaturen", "Ersetzt Paare wie fi oder fl durch eine verbundene Form.",
    "Lokalisierte Formen", "Verwendet die Buchstabenformen, die eine Sprache bevorzugt.",
    "Mittelformen", "Formt Buchstaben in der Wortmitte, etwa im Arabischen.",
    "Stilsets", "Gruppen alternativer Buchstabenentwürfe des Schriftgestalters.",
    "Name", "Familie", "Schnitt", "Format", "PostScript-Name", "Eindeutiger Name", "Speicherort", "Schriftsammlungen", "Schriftvariablen", "Version",
    "Anzahl Glyphen", "Tags", "Keine", "Die Schalter ändern alle Vorschauen in FontCraft.", "Variabel", "Fett", "Standard {n}",
];

const IT: [&str; N2] = [
    "Anteprima", "Dimensioni", "Glifi", "Funzioni", "Info", "Maiuscole", "Minuscole", "Numeri", "Punteggiatura",
    "Fai clic su una dimensione per usarla nell'elenco", "{n} glifi in questo font", "Unicode", "Nome del glifo", "Nessun carattere", "Non presente in questo font",
    "{n} regole", "+{n} altri", "Questa funzione agisce tramite regole di contesto, quindi non ci sono esempi fissi.",
    "{n} set disponibili", "Questo font non ha set stilistici", "Set {n}",
    "Accesso a tutte le alternative", "Mostra tutte le forme alternative di una lettera.",
    "Forme sensibili alle maiuscole", "Adatta punteggiatura e simboli alle lettere maiuscole.",
    "Composizione/scomposizione dei glifi", "Unisce o separa lettere e accenti per mostrarli correttamente.",
    "Legature standard", "Sostituisce coppie come fi o fl con un'unica forma legata.",
    "Forme localizzate", "Usa le forme di lettera preferite da una lingua.",
    "Forme mediali", "Modella le lettere a metà parola, come in arabo.",
    "Set stilistici", "Gruppi di disegni alternativi creati dal type designer.",
    "Nome", "Famiglia", "Stile", "Formato", "Nome PostScript", "Nome univoco", "Posizione", "Raccolte di font", "Variabili del font", "Versione",
    "Numero di glifi", "Tag", "Nessuno", "Gli interruttori cambiano tutte le anteprime di FontCraft.", "Variabile", "Grassetto", "predefinito {n}",
];

const PT: [&str; N2] = [
    "Prévia", "Tamanhos", "Glifos", "Recursos", "Informações", "Maiúsculas", "Minúsculas", "Números", "Pontuação",
    "Clique num tamanho para usá-lo na lista", "{n} glifos nesta fonte", "Unicode", "Nome do glifo", "Sem caractere", "Não existe nesta fonte",
    "{n} regras", "+{n} mais", "Este recurso age por regras de contexto, então não há exemplos fixos.",
    "{n} conjuntos disponíveis", "Esta fonte não tem conjuntos estilísticos", "Conjunto {n}",
    "Acesso a todas as alternativas", "Mostra todas as formas alternativas de uma letra.",
    "Formas sensíveis a maiúsculas", "Ajusta pontuação e símbolos às letras maiúsculas.",
    "Composição/decomposição de glifos", "Junta ou separa letras e acentos para que apareçam corretamente.",
    "Ligaduras padrão", "Troca pares como fi ou fl por uma única forma ligada.",
    "Formas localizadas", "Usa as formas de letra preferidas por um idioma.",
    "Formas mediais", "Dá forma às letras no meio da palavra, como no árabe.",
    "Conjuntos estilísticos", "Grupos de desenhos alternativos criados pelo designer da fonte.",
    "Nome", "Família", "Estilo", "Formato", "Nome PostScript", "Nome único", "Local", "Conjuntos de fontes", "Variáveis da fonte", "Versão",
    "Número de glifos", "Etiquetas", "Nenhum", "As chaves mudam todas as prévias do FontCraft.", "Variável", "Negrito", "padrão {n}",
];

const NL: [&str; N2] = [
    "Voorvertoning", "Groottes", "Glyphs", "Functies", "Info", "Hoofdletters", "Kleine letters", "Cijfers", "Leestekens",
    "Klik op een grootte om die in de lijst te gebruiken", "{n} glyphs in dit lettertype", "Unicode", "Glyphnaam", "Geen teken", "Niet in dit lettertype",
    "{n} regels", "+{n} meer", "Deze functie werkt via contextregels, dus er zijn geen vaste voorbeelden.",
    "{n} sets beschikbaar", "Dit lettertype heeft geen stijlsets", "Set {n}",
    "Toegang tot alle alternatieven", "Toont elke alternatieve vorm van een letter.",
    "Hoofdlettergevoelige vormen", "Zet leestekens en symbolen goed bij hoofdletters.",
    "Glyphsamenstelling/-ontleding", "Voegt letters en accenten samen of splitst ze voor juiste weergave.",
    "Standaardligaturen", "Vervangt paren als fi of fl door één verbonden vorm.",
    "Gelokaliseerde vormen", "Gebruikt de lettervormen die een taal verkiest.",
    "Middenvormen", "Vormt letters midden in een woord, zoals in het Arabisch.",
    "Stijlsets", "Groepen alternatieve letterontwerpen van de ontwerper.",
    "Naam", "Familie", "Stijl", "Formaat", "PostScript-naam", "Unieke naam", "Locatie", "Lettertypesets", "Lettertypevariabelen", "Versie",
    "Aantal glyphs", "Tags", "Geen", "De schakelaars veranderen alle voorvertoningen in FontCraft.", "Variabel", "Vet", "standaard {n}",
];

const SV: [&str; N2] = [
    "Förhandsvisning", "Storlekar", "Glyfer", "Funktioner", "Info", "Versaler", "Gemener", "Siffror", "Skiljetecken",
    "Klicka på en storlek för att använda den i listan", "{n} glyfer i typsnittet", "Unicode", "Glyfnamn", "Inget tecken", "Finns inte i typsnittet",
    "{n} regler", "+{n} till", "Funktionen verkar via kontextregler, så det finns inga fasta exempel.",
    "{n} uppsättningar finns", "Typsnittet har inga stiluppsättningar", "Uppsättning {n}",
    "Åtkomst till alla alternativ", "Visar alla alternativa former för en bokstav.",
    "Versalanpassade former", "Anpassar skiljetecken och symboler till versaler.",
    "Glyfsammansättning/-uppdelning", "Sätter ihop eller delar bokstäver och accenter så de visas rätt.",
    "Standardligaturer", "Ersätter par som fi eller fl med en sammanbunden form.",
    "Lokaliserade former", "Använder bokstavsformer som ett visst språk föredrar.",
    "Mittformer", "Formar bokstäver mitt i ett ord, som i arabiska.",
    "Stiluppsättningar", "Grupper av alternativa bokstavsformer från formgivaren.",
    "Namn", "Familj", "Stil", "Format", "PostScript-namn", "Unikt namn", "Plats", "Typsnittssamlingar", "Typsnittsvariabler", "Version",
    "Antal glyfer", "Taggar", "Inga", "Reglagen ändrar alla förhandsvisningar i FontCraft.", "Variabel", "Fet", "standard {n}",
];

const DA: [&str; N2] = [
    "Eksempel", "Størrelser", "Glyffer", "Funktioner", "Info", "Store bogstaver", "Små bogstaver", "Tal", "Tegnsætning",
    "Klik på en størrelse for at bruge den i listen", "{n} glyffer i skrifttypen", "Unicode", "Glyfnavn", "Intet tegn", "Findes ikke i skrifttypen",
    "{n} regler", "+{n} mere", "Funktionen virker via kontekstregler, så der er ingen faste eksempler.",
    "{n} sæt tilgængelige", "Skrifttypen har ingen stilsæt", "Sæt {n}",
    "Adgang til alle alternativer", "Viser alle alternative former for et bogstav.",
    "Versalfølsomme former", "Tilpasser tegnsætning og symboler til store bogstaver.",
    "Glyfsammensætning/-opdeling", "Samler eller deler bogstaver og accenter, så de vises korrekt.",
    "Standardligaturer", "Erstatter par som fi eller fl med én sammenhængende form.",
    "Lokaliserede former", "Bruger de bogstavformer et bestemt sprog foretrækker.",
    "Midterformer", "Former bogstaver midt i et ord, som på arabisk.",
    "Stilsæt", "Grupper af alternative bogstavdesign fra designeren.",
    "Navn", "Familie", "Stil", "Format", "PostScript-navn", "Unikt navn", "Placering", "Skrifttypesæt", "Skrifttypevariabler", "Version",
    "Antal glyffer", "Tags", "Ingen", "Kontakterne ændrer alle eksempler i FontCraft.", "Variabel", "Fed", "standard {n}",
];

const NB: [&str; N2] = [
    "Forhåndsvisning", "Størrelser", "Glyfer", "Funksjoner", "Info", "Store bokstaver", "Små bokstaver", "Tall", "Tegnsetting",
    "Klikk på en størrelse for å bruke den i listen", "{n} glyfer i skriften", "Unicode", "Glyfnavn", "Inget tegn", "Finnes ikke i skriften",
    "{n} regler", "+{n} til", "Funksjonen virker via kontekstregler, så det finnes ingen faste eksempler.",
    "{n} sett tilgjengelig", "Skriften har ingen stilsett", "Sett {n}",
    "Tilgang til alle alternativer", "Viser alle alternative former for en bokstav.",
    "Versalfølsomme former", "Tilpasser tegnsetting og symboler til store bokstaver.",
    "Glyfsammensetning/-oppdeling", "Setter sammen eller deler bokstaver og aksenter så de vises riktig.",
    "Standardligaturer", "Erstatter par som fi eller fl med én sammenbundet form.",
    "Lokaliserte former", "Bruker bokstavformene et bestemt språk foretrekker.",
    "Midtformer", "Former bokstaver midt i et ord, som på arabisk.",
    "Stilsett", "Grupper av alternative bokstavdesign fra designeren.",
    "Navn", "Familie", "Stil", "Format", "PostScript-navn", "Unikt navn", "Plassering", "Skriftsett", "Skriftvariabler", "Versjon",
    "Antall glyfer", "Tagger", "Ingen", "Bryterne endrer alle forhåndsvisninger i FontCraft.", "Variabel", "Fet", "standard {n}",
];

const FI: [&str; N2] = [
    "Esikatselu", "Koot", "Merkit", "Ominaisuudet", "Tiedot", "Isot kirjaimet", "Pienet kirjaimet", "Numerot", "Välimerkit",
    "Valitse koko käyttääksesi sitä luettelossa", "Fontissa on {n} merkkiä", "Unicode", "Merkin nimi", "Ei merkkiä", "Ei tässä fontissa",
    "{n} sääntöä", "+{n} lisää", "Ominaisuus toimii kontekstisääntöjen kautta, joten kiinteitä esimerkkejä ei ole.",
    "{n} sarjaa saatavilla", "Fontissa ei ole tyylisarjoja", "Sarja {n}",
    "Kaikki vaihtoehdot", "Näyttää kirjaimen kaikki vaihtoehtoiset muodot.",
    "Versaaliherkät muodot", "Sovittaa välimerkit ja symbolit isoihin kirjaimiin.",
    "Merkkien yhdistäminen/jakaminen", "Yhdistää tai erottaa kirjaimet ja tarkkeet, jotta ne näkyvät oikein.",
    "Vakioligatuurit", "Korvaa parit kuten fi tai fl yhdellä yhdistetyllä muodolla.",
    "Lokalisoidut muodot", "Käyttää tietyn kielen suosimia kirjainmuotoja.",
    "Keskimuodot", "Muotoilee sanan keskellä olevat kirjaimet, kuten arabiassa.",
    "Tyylisarjat", "Suunnittelijan tekemiä vaihtoehtoisten kirjainmuotojen ryhmiä.",
    "Nimi", "Perhe", "Tyyli", "Muoto", "PostScript-nimi", "Yksilöllinen nimi", "Sijainti", "Fonttikokoelmat", "Fontin muuttujat", "Versio",
    "Merkkien määrä", "Tunnisteet", "Ei mitään", "Kytkimet muuttavat kaikkia FontCraftin esikatseluja.", "Muuttuva", "Lihavoitu", "oletus {n}",
];

const PL: [&str; N2] = [
    "Podgląd", "Rozmiary", "Glify", "Funkcje", "Informacje", "Wielkie litery", "Małe litery", "Cyfry", "Interpunkcja",
    "Kliknij rozmiar, aby użyć go na liście", "Glify w tej czcionce: {n}", "Unicode", "Nazwa glifu", "Brak znaku", "Brak w tej czcionce",
    "Reguły: {n}", "+{n} więcej", "Ta funkcja działa przez reguły kontekstowe, więc nie ma stałych przykładów.",
    "Dostępne zestawy: {n}", "Ta czcionka nie ma zestawów stylistycznych", "Zestaw {n}",
    "Dostęp do wszystkich wariantów", "Pokazuje każdy wariant kształtu litery.",
    "Formy zależne od wielkości liter", "Dopasowuje interpunkcję i symbole do wielkich liter.",
    "Składanie/rozkładanie glifów", "Łączy lub rozdziela litery i akcenty, by wyświetlały się poprawnie.",
    "Ligatury standardowe", "Zastępuje pary jak fi czy fl jednym połączonym kształtem.",
    "Formy lokalne", "Używa kształtów liter preferowanych przez dany język.",
    "Formy środkowe", "Kształtuje litery w środku słowa, jak w arabskim.",
    "Zestawy stylistyczne", "Grupy alternatywnych kształtów liter od projektanta.",
    "Nazwa", "Rodzina", "Styl", "Format", "Nazwa PostScript", "Unikalna nazwa", "Lokalizacja", "Zestawy czcionek", "Zmienne czcionki", "Wersja",
    "Liczba glifów", "Tagi", "Brak", "Przełączniki zmieniają wszystkie podglądy w FontCraft.", "Zmienna", "Pogrubiona", "domyślnie {n}",
];

const CS: [&str; N2] = [
    "Náhled", "Velikosti", "Glyfy", "Funkce", "Informace", "Velká písmena", "Malá písmena", "Číslice", "Interpunkce",
    "Klikněte na velikost a použije se v seznamu", "Glyfy v tomto písmu: {n}", "Unicode", "Název glyfu", "Žádný znak", "V tomto písmu není",
    "Pravidla: {n}", "+{n} dalších", "Tato funkce působí přes kontextová pravidla, proto nejsou pevné ukázky.",
    "Dostupné sady: {n}", "Toto písmo nemá stylistické sady", "Sada {n}",
    "Přístup ke všem variantám", "Ukáže všechny varianty tvaru písmene.",
    "Tvary citlivé na velikost písmen", "Přizpůsobí interpunkci a symboly velkým písmenům.",
    "Skládání/rozklad glyfů", "Spojí nebo rozdělí písmena a diakritiku pro správné zobrazení.",
    "Standardní ligatury", "Nahradí dvojice jako fi nebo fl jedním spojeným tvarem.",
    "Lokalizované tvary", "Použije tvary písmen, které preferuje daný jazyk.",
    "Střední tvary", "Tvaruje písmena uprostřed slova, jako v arabštině.",
    "Stylistické sady", "Skupiny alternativních tvarů písmen od autora písma.",
    "Název", "Rodina", "Řez", "Formát", "Název PostScript", "Jedinečný název", "Umístění", "Sady písem", "Proměnné písma", "Verze",
    "Počet glyfů", "Štítky", "Žádné", "Přepínače mění všechny náhledy ve FontCraft.", "Proměnné", "Tučné", "výchozí {n}",
];

const TR: [&str; N2] = [
    "Önizleme", "Boyutlar", "Glifler", "Özellikler", "Bilgi", "Büyük harfler", "Küçük harfler", "Rakamlar", "Noktalama",
    "Listede kullanmak için bir boyuta tıklayın", "Bu yazı tipinde {n} glif", "Unicode", "Glif adı", "Karakter yok", "Bu yazı tipinde yok",
    "{n} kural", "+{n} daha", "Bu özellik bağlam kurallarıyla çalışır, bu yüzden sabit örnek yok.",
    "{n} set mevcut", "Bu yazı tipinde stil seti yok", "Set {n}",
    "Tüm alternatiflere erişim", "Bir harfin tüm alternatif biçimlerini gösterir.",
    "Büyük harfe duyarlı biçimler", "Noktalama ve sembolleri büyük harflere uydurur.",
    "Glif birleştirme/ayırma", "Harfleri ve aksanları doğru görünmeleri için birleştirir veya ayırır.",
    "Standart bitişik harfler", "fi veya fl gibi çiftleri tek bir bitişik biçimle değiştirir.",
    "Yerelleştirilmiş biçimler", "Belirli bir dilin tercih ettiği harf biçimlerini kullanır.",
    "Orta biçimler", "Arapçadaki gibi kelime ortasındaki harfleri biçimlendirir.",
    "Stil setleri", "Tasarımcının hazırladığı alternatif harf tasarımı grupları.",
    "Ad", "Aile", "Stil", "Biçim", "PostScript adı", "Benzersiz ad", "Konum", "Yazı tipi setleri", "Yazı tipi değişkenleri", "Sürüm",
    "Glif sayısı", "Etiketler", "Yok", "Anahtarlar FontCraft'taki tüm önizlemeleri değiştirir.", "Değişken", "Kalın", "varsayılan {n}",
];

const RO: [&str; N2] = [
    "Previzualizare", "Mărimi", "Glife", "Funcții", "Informații", "Majuscule", "Minuscule", "Cifre", "Punctuație",
    "Dă clic pe o mărime ca s-o folosești în listă", "{n} glife în acest font", "Unicode", "Numele glifei", "Niciun caracter", "Nu există în acest font",
    "{n} reguli", "încă {n}", "Funcția acționează prin reguli de context, deci nu există exemple fixe.",
    "{n} seturi disponibile", "Fontul nu are seturi stilistice", "Setul {n}",
    "Acces la toate variantele", "Arată toate formele alternative ale unei litere.",
    "Forme sensibile la majuscule", "Potrivește punctuația și simbolurile cu majusculele.",
    "Compunere/descompunere de glife", "Unește sau desparte litere și accente ca să apară corect.",
    "Ligaturi standard", "Înlocuiește perechi ca fi sau fl cu o singură formă legată.",
    "Forme localizate", "Folosește formele de litere preferate de o anumită limbă.",
    "Forme mediale", "Modelează literele din mijlocul cuvântului, ca în arabă.",
    "Seturi stilistice", "Grupuri de forme alternative create de designerul fontului.",
    "Nume", "Familie", "Stil", "Format", "Nume PostScript", "Nume unic", "Locație", "Seturi de fonturi", "Variabile ale fontului", "Versiune",
    "Număr de glife", "Etichete", "Niciuna", "Comutatoarele schimbă toate previzualizările din FontCraft.", "Variabil", "Aldin", "implicit {n}",
];

const HU: [&str; N2] = [
    "Előnézet", "Méretek", "Jelek", "Funkciók", "Adatok", "Nagybetűk", "Kisbetűk", "Számok", "Írásjelek",
    "Kattints egy méretre, hogy a listában használd", "{n} jel ebben a betűtípusban", "Unicode", "Jel neve", "Nincs karakter", "Nincs ebben a betűtípusban",
    "{n} szabály", "még {n}", "Ez a funkció környezeti szabályokkal működik, ezért nincs fix példa.",
    "{n} készlet érhető el", "Ennek a betűtípusnak nincs stíluskészlete", "{n}. készlet",
    "Minden alternatíva", "Megmutatja egy betű összes alternatív alakját.",
    "Nagybetűhöz igazított formák", "A nagybetűkhöz igazítja az írásjeleket és szimbólumokat.",
    "Jelösszetétel/-bontás", "Összevonja vagy szétválasztja a betűket és ékezeteket a helyes megjelenésért.",
    "Szabványos ligatúrák", "Az fi vagy fl párokat egyetlen összekötött alakra cseréli.",
    "Nyelvi formák", "Egy adott nyelv által előnyben részesített betűalakokat használja.",
    "Szóközépi formák", "A szó közepén álló betűket alakítja, mint az arabban.",
    "Stíluskészletek", "A tervező alternatív betűterveinek csoportjai.",
    "Név", "Család", "Stílus", "Formátum", "PostScript-név", "Egyedi név", "Hely", "Betűtípus-készletek", "Betűtípus-változók", "Verzió",
    "Jelek száma", "Címkék", "Nincs", "A kapcsolók a FontCraft összes előnézetét módosítják.", "Változó", "Félkövér", "alapértelmezett {n}",
];

const EL: [&str; N2] = [
    "Προεπισκόπηση", "Μεγέθη", "Γλύφοι", "Λειτουργίες", "Πληροφορίες", "Κεφαλαία", "Πεζά", "Αριθμοί", "Στίξη",
    "Κάντε κλικ σε ένα μέγεθος για χρήση στη λίστα", "{n} γλύφοι σε αυτή τη γραμματοσειρά", "Unicode", "Όνομα γλύφου", "Χωρίς χαρακτήρα", "Δεν υπάρχει σε αυτή τη γραμματοσειρά",
    "{n} κανόνες", "+{n} ακόμη", "Η λειτουργία δρα μέσω κανόνων περιβάλλοντος, οπότε δεν υπάρχουν σταθερά παραδείγματα.",
    "{n} σύνολα διαθέσιμα", "Η γραμματοσειρά δεν έχει στυλιστικά σύνολα", "Σύνολο {n}",
    "Πρόσβαση σε όλες τις εναλλακτικές", "Δείχνει κάθε εναλλακτικό σχήμα ενός γράμματος.",
    "Μορφές ευαίσθητες σε κεφαλαία", "Προσαρμόζει στίξη και σύμβολα στα κεφαλαία.",
    "Σύνθεση/αποσύνθεση γλύφων", "Ενώνει ή χωρίζει γράμματα και τόνους ώστε να εμφανίζονται σωστά.",
    "Τυπικά συμπλέγματα", "Αντικαθιστά ζεύγη όπως fi ή fl με ένα ενωμένο σχήμα.",
    "Τοπικές μορφές", "Χρησιμοποιεί τα σχήματα γραμμάτων που προτιμά μια γλώσσα.",
    "Μεσαίες μορφές", "Διαμορφώνει γράμματα στη μέση της λέξης, όπως στα αραβικά.",
    "Στυλιστικά σύνολα", "Ομάδες εναλλακτικών σχεδίων γραμμάτων από τον σχεδιαστή.",
    "Όνομα", "Οικογένεια", "Στυλ", "Μορφή", "Όνομα PostScript", "Μοναδικό όνομα", "Θέση", "Σύνολα γραμματοσειρών", "Μεταβλητές γραμματοσειράς", "Έκδοση",
    "Πλήθος γλύφων", "Ετικέτες", "Κανένα", "Οι διακόπτες αλλάζουν όλες τις προεπισκοπήσεις στο FontCraft.", "Μεταβλητή", "Έντονη", "προεπιλογή {n}",
];

const RU: [&str; N2] = [
    "Просмотр", "Размеры", "Глифы", "Функции", "Сведения", "Прописные", "Строчные", "Цифры", "Знаки препинания",
    "Нажмите на размер, чтобы использовать его в списке", "Глифов в шрифте: {n}", "Юникод", "Имя глифа", "Нет символа", "Нет в этом шрифте",
    "Правил: {n}", "ещё {n}", "Функция работает через контекстные правила, поэтому готовых примеров нет.",
    "Доступно наборов: {n}", "В шрифте нет стилистических наборов", "Набор {n}",
    "Доступ ко всем вариантам", "Показывает все альтернативные формы буквы.",
    "Формы для прописных", "Подстраивает знаки и символы под прописные буквы.",
    "Сборка/разборка глифов", "Соединяет или разделяет буквы и диакритику для правильного вида.",
    "Стандартные лигатуры", "Заменяет пары вроде fi или fl одной слитной формой.",
    "Локализованные формы", "Использует формы букв, принятые в конкретном языке.",
    "Срединные формы", "Формирует буквы в середине слова, как в арабском.",
    "Стилистические наборы", "Группы альтернативных букв, созданные автором шрифта.",
    "Название", "Семейство", "Начертание", "Формат", "Имя PostScript", "Уникальное имя", "Расположение", "Наборы шрифтов", "Переменные шрифта", "Версия",
    "Число глифов", "Теги", "Нет", "Переключатели меняют все образцы в FontCraft.", "Вариативный", "Жирный", "по умолчанию {n}",
];

const UK: [&str; N2] = [
    "Перегляд", "Розміри", "Гліфи", "Функції", "Відомості", "Великі літери", "Малі літери", "Цифри", "Розділові знаки",
    "Натисніть розмір, щоб використати його у списку", "Гліфів у шрифті: {n}", "Юнікод", "Назва гліфа", "Немає символу", "Немає в цьому шрифті",
    "Правил: {n}", "ще {n}", "Функція діє через контекстні правила, тому готових прикладів немає.",
    "Доступно наборів: {n}", "У шрифті немає стилістичних наборів", "Набір {n}",
    "Доступ до всіх варіантів", "Показує всі альтернативні форми літери.",
    "Форми для великих літер", "Підлаштовує знаки й символи під великі літери.",
    "Складання/розкладання гліфів", "Поєднує або розділяє літери й діакритику для правильного вигляду.",
    "Стандартні лігатури", "Замінює пари на кшталт fi чи fl однією злитою формою.",
    "Локалізовані форми", "Використовує форми літер, прийняті в певній мові.",
    "Серединні форми", "Формує літери всередині слова, як в арабській.",
    "Стилістичні набори", "Групи альтернативних літер від автора шрифту.",
    "Назва", "Родина", "Накреслення", "Формат", "Ім'я PostScript", "Унікальна назва", "Розташування", "Набори шрифтів", "Змінні шрифту", "Версія",
    "Кількість гліфів", "Теги", "Немає", "Перемикачі змінюють усі зразки у FontCraft.", "Варіативний", "Жирний", "типово {n}",
];

const ID: [&str; N2] = [
    "Pratinjau", "Ukuran", "Glif", "Fitur", "Info", "Huruf besar", "Huruf kecil", "Angka", "Tanda baca",
    "Klik ukuran untuk dipakai di daftar", "{n} glif di font ini", "Unicode", "Nama glif", "Tanpa karakter", "Tidak ada di font ini",
    "{n} aturan", "+{n} lagi", "Fitur ini bekerja lewat aturan konteks, jadi tidak ada contoh tetap.",
    "{n} set tersedia", "Font ini tidak punya set gaya", "Set {n}",
    "Akses semua alternatif", "Menampilkan semua bentuk alternatif sebuah huruf.",
    "Bentuk peka huruf besar", "Menyesuaikan tanda baca dan simbol dengan huruf besar.",
    "Komposisi/dekomposisi glif", "Menggabung atau memisah huruf dan aksen agar tampil benar.",
    "Ligatur standar", "Mengganti pasangan seperti fi atau fl dengan satu bentuk tersambung.",
    "Bentuk lokal", "Memakai bentuk huruf yang disukai bahasa tertentu.",
    "Bentuk tengah", "Membentuk huruf di tengah kata, seperti dalam bahasa Arab.",
    "Set gaya", "Kelompok desain huruf alternatif dari perancang font.",
    "Nama", "Keluarga", "Gaya", "Format", "Nama PostScript", "Nama unik", "Lokasi", "Set font", "Variabel font", "Versi",
    "Jumlah glif", "Tag", "Tidak ada", "Sakelar mengubah semua pratinjau di FontCraft.", "Variabel", "Tebal", "bawaan {n}",
];

const VI: [&str; N2] = [
    "Xem trước", "Cỡ chữ", "Glyph", "Tính năng", "Thông tin", "Chữ hoa", "Chữ thường", "Chữ số", "Dấu câu",
    "Bấm một cỡ để dùng trong danh sách", "{n} glyph trong phông này", "Unicode", "Tên glyph", "Không có ký tự", "Không có trong phông này",
    "{n} quy tắc", "+{n} nữa", "Tính năng này hoạt động qua quy tắc ngữ cảnh nên không có ví dụ cố định.",
    "Có {n} bộ", "Phông này không có bộ kiểu", "Bộ {n}",
    "Truy cập mọi kiểu thay thế", "Hiện mọi hình dạng thay thế của một chữ.",
    "Dạng theo chữ hoa", "Chỉnh dấu câu và ký hiệu cho hợp với chữ hoa.",
    "Ghép/tách glyph", "Ghép hoặc tách chữ và dấu để hiển thị đúng.",
    "Chữ ghép chuẩn", "Thay cặp như fi hoặc fl bằng một hình nối liền.",
    "Dạng bản địa", "Dùng hình chữ mà một ngôn ngữ ưa dùng.",
    "Dạng giữa từ", "Tạo hình chữ ở giữa từ, như trong tiếng Ả Rập.",
    "Bộ kiểu", "Các nhóm thiết kế chữ thay thế do nhà thiết kế tạo.",
    "Tên", "Họ", "Kiểu", "Định dạng", "Tên PostScript", "Tên duy nhất", "Vị trí", "Bộ phông", "Biến của phông", "Phiên bản",
    "Số glyph", "Thẻ", "Không có", "Các công tắc thay đổi mọi bản xem trước trong FontCraft.", "Biến thiên", "Đậm", "mặc định {n}",
];

const ZH_HANS: [&str; N2] = [
    "预览", "字号", "字形", "功能", "信息", "大写字母", "小写字母", "数字", "标点",
    "点击字号即可在列表中使用", "此字体有 {n} 个字形", "Unicode", "字形名称", "无对应字符", "此字体不包含",
    "{n} 条规则", "还有 {n} 个", "此功能通过上下文规则生效，没有固定示例。",
    "共 {n} 组", "此字体没有风格集", "第 {n} 组",
    "全部替代字形", "显示一个字母的所有替代形状。",
    "大写敏感形式", "让标点和符号与大写字母更协调。",
    "字形组合/分解", "合并或拆分字母与重音符号，使其正确显示。",
    "标准连字", "把 fi、fl 等字母组合替换为一个连写字形。",
    "本地化形式", "使用特定语言偏好的字母形状。",
    "词中形式", "为词中间的字母选用对应字形，如阿拉伯文。",
    "风格集", "字体设计师提供的替代字形组。",
    "名称", "字族", "样式", "格式", "PostScript 名称", "唯一名称", "位置", "字体集", "字体变量", "版本",
    "字形数", "标签", "无", "开关会改变 FontCraft 中的所有预览。", "可变", "粗体", "默认 {n}",
];

const ZH_HANT: [&str; N2] = [
    "預覽", "字級", "字符", "功能", "資訊", "大寫字母", "小寫字母", "數字", "標點",
    "按一下字級即可在列表中使用", "此字體有 {n} 個字符", "Unicode", "字符名稱", "無對應字元", "此字體不包含",
    "{n} 條規則", "還有 {n} 個", "此功能透過上下文規則生效，沒有固定範例。",
    "共 {n} 組", "此字體沒有風格集", "第 {n} 組",
    "所有替代字符", "顯示一個字母的所有替代形狀。",
    "大寫敏感形式", "讓標點和符號與大寫字母更協調。",
    "字符組合/分解", "合併或拆分字母與重音符號，使其正確顯示。",
    "標準連字", "把 fi、fl 等字母組合替換為一個連寫字符。",
    "在地化形式", "使用特定語言偏好的字母形狀。",
    "詞中形式", "為詞中間的字母選用對應字符，如阿拉伯文。",
    "風格集", "字體設計師提供的替代字符組。",
    "名稱", "字族", "樣式", "格式", "PostScript 名稱", "唯一名稱", "位置", "字體集", "字體變數", "版本",
    "字符數", "標籤", "無", "開關會改變 FontCraft 中的所有預覽。", "可變", "粗體", "預設 {n}",
];

const JA: [&str; N2] = [
    "プレビュー", "サイズ", "グリフ", "機能", "情報", "大文字", "小文字", "数字", "約物",
    "サイズをクリックすると一覧に反映されます", "このフォントのグリフ数：{n}", "Unicode", "グリフ名", "文字なし", "このフォントにはありません",
    "{n} 件のルール", "ほか {n} 件", "この機能は文脈ルールで働くため、固定の例はありません。",
    "{n} 組あります", "このフォントにはスタイルセットがありません", "セット {n}",
    "すべての異体字", "文字のすべての異体字を表示します。",
    "大文字用字形", "句読点や記号を大文字に合う位置に調整します。",
    "グリフの合成・分解", "文字とアクセントを正しく表示するために結合・分解します。",
    "標準合字", "fi や fl などの組み合わせを 1 つの合字に置き換えます。",
    "ローカライズ字形", "特定の言語で好まれる字形を使います。",
    "語中形", "アラビア文字のように語の途中の文字を整形します。",
    "スタイルセット", "書体デザイナーが用意した異体字のグループです。",
    "名前", "ファミリー", "スタイル", "形式", "PostScript 名", "固有名", "場所", "フォントセット", "フォント変数", "バージョン",
    "グリフ数", "タグ", "なし", "スイッチは FontCraft のすべてのプレビューに反映されます。", "バリアブル", "太字", "既定値 {n}",
];

const KO: [&str; N2] = [
    "미리보기", "크기", "글리프", "기능", "정보", "대문자", "소문자", "숫자", "문장 부호",
    "크기를 클릭하면 목록에 적용됩니다", "이 서체의 글리프 {n}개", "유니코드", "글리프 이름", "문자 없음", "이 서체에 없음",
    "규칙 {n}개", "{n}개 더", "이 기능은 문맥 규칙으로 작동하므로 고정된 예시가 없습니다.",
    "{n}개 세트 사용 가능", "이 서체에는 스타일 세트가 없습니다", "세트 {n}",
    "모든 대체 글자", "글자의 모든 대체 모양을 보여 줍니다.",
    "대문자용 형태", "문장 부호와 기호를 대문자에 맞게 조정합니다.",
    "글리프 조합/분해", "글자와 악센트를 올바르게 보이도록 합치거나 나눕니다.",
    "표준 합자", "fi, fl 같은 글자 쌍을 하나의 연결된 모양으로 바꿉니다.",
    "지역화 형태", "특정 언어가 선호하는 글자 모양을 사용합니다.",
    "중간 형태", "아랍어처럼 단어 중간에 오는 글자의 모양을 만듭니다.",
    "스타일 세트", "서체 디자이너가 만든 대체 글자 디자인 묶음입니다.",
    "이름", "패밀리", "스타일", "형식", "PostScript 이름", "고유 이름", "위치", "서체 세트", "서체 변수", "버전",
    "글리프 수", "태그", "없음", "스위치는 FontCraft의 모든 미리보기를 바꿉니다.", "가변", "굵게", "기본값 {n}",
];
