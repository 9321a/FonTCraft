//! Words for style lists, custom libraries and activation, in the same language order as `i18n::LANGS`.

pub const N3: usize = 22;

pub const EXTRA2: [&[&str; N3]; 25] = [
    &EN, &ES, &FR, &DE, &IT, &PT, &NL, &SV, &DA, &NB, &FI, &PL, &CS, &TR, &RO, &HU, &EL, &RU, &UK, &ID, &VI, &ZH_HANS,
    &ZH_HANT, &JA, &KO,
];

const EN: [&str; N3] = [
    "With Serifs", "Sans-Serif", "Script", "Symbols & Icons", "MY LIBRARIES", "New Library",
    "Pick a folder anywhere: every font inside it is added to the library.", "Folder", "Choose…", "Library name",
    "Use the folder's name", "Custom name", "{n} fonts found", "Create Library", "Cancel", "Remove Library",
    "Inactive — click to activate it so other apps can use it", "Active — click to deactivate",
    "Always available: a system font that can't be turned off", "Always available: installed on this Mac",
    "Couldn't activate this font", "That folder doesn't exist",
];

const ES: [&str; N3] = [
    "Con serifa", "Sin serifa", "Manuscritas", "Símbolos e iconos", "MIS BIBLIOTECAS", "Nueva biblioteca",
    "Elige una carpeta en cualquier lugar: se añadirán todas las fuentes que contenga.", "Carpeta", "Elegir…", "Nombre de la biblioteca",
    "Usar el nombre de la carpeta", "Nombre personalizado", "{n} fuentes encontradas", "Crear biblioteca", "Cancelar", "Eliminar biblioteca",
    "Inactiva: haz clic para activarla y que otras apps puedan usarla", "Activa: haz clic para desactivarla",
    "Siempre disponible: fuente del sistema que no se puede desactivar", "Siempre disponible: instalada en este Mac",
    "No se pudo activar esta fuente", "Esa carpeta no existe",
];

const FR: [&str; N3] = [
    "Avec empattements", "Sans empattements", "Scriptes", "Symboles et icônes", "MES BIBLIOTHÈQUES", "Nouvelle bibliothèque",
    "Choisissez un dossier n'importe où : toutes ses polices seront ajoutées.", "Dossier", "Choisir…", "Nom de la bibliothèque",
    "Utiliser le nom du dossier", "Nom personnalisé", "{n} polices trouvées", "Créer la bibliothèque", "Annuler", "Supprimer la bibliothèque",
    "Inactive : cliquez pour l'activer et la rendre disponible aux autres apps", "Active : cliquez pour la désactiver",
    "Toujours disponible : police système qui ne peut pas être désactivée", "Toujours disponible : installée sur ce Mac",
    "Impossible d'activer cette police", "Ce dossier n'existe pas",
];

const DE: [&str; N3] = [
    "Mit Serifen", "Serifenlos", "Schreibschrift", "Symbole & Icons", "MEINE BIBLIOTHEKEN", "Neue Bibliothek",
    "Wähle einen beliebigen Ordner: Alle Schriften darin werden hinzugefügt.", "Ordner", "Auswählen …", "Name der Bibliothek",
    "Ordnernamen verwenden", "Eigener Name", "{n} Schriften gefunden", "Bibliothek erstellen", "Abbrechen", "Bibliothek entfernen",
    "Inaktiv – klicken zum Aktivieren, damit andere Apps sie nutzen können", "Aktiv – klicken zum Deaktivieren",
    "Immer verfügbar: Systemschrift, nicht abschaltbar", "Immer verfügbar: auf diesem Mac installiert",
    "Schrift konnte nicht aktiviert werden", "Dieser Ordner existiert nicht",
];

const IT: [&str; N3] = [
    "Con grazie", "Senza grazie", "Calligrafici", "Simboli e icone", "LE MIE LIBRERIE", "Nuova libreria",
    "Scegli una cartella ovunque: verranno aggiunti tutti i font al suo interno.", "Cartella", "Scegli…", "Nome della libreria",
    "Usa il nome della cartella", "Nome personalizzato", "{n} font trovati", "Crea libreria", "Annulla", "Rimuovi libreria",
    "Inattivo: fai clic per attivarlo e renderlo disponibile alle altre app", "Attivo: fai clic per disattivarlo",
    "Sempre disponibile: font di sistema non disattivabile", "Sempre disponibile: installato su questo Mac",
    "Impossibile attivare questo font", "La cartella non esiste",
];

const PT: [&str; N3] = [
    "Com serifa", "Sem serifa", "Manuscritas", "Símbolos e ícones", "MINHAS BIBLIOTECAS", "Nova biblioteca",
    "Escolha uma pasta em qualquer lugar: todas as fontes nela serão adicionadas.", "Pasta", "Escolher…", "Nome da biblioteca",
    "Usar o nome da pasta", "Nome personalizado", "{n} fontes encontradas", "Criar biblioteca", "Cancelar", "Remover biblioteca",
    "Inativa: clique para ativar e outros apps poderem usá-la", "Ativa: clique para desativar",
    "Sempre disponível: fonte do sistema que não pode ser desligada", "Sempre disponível: instalada neste Mac",
    "Não foi possível ativar esta fonte", "Essa pasta não existe",
];

const NL: [&str; N3] = [
    "Schreef", "Schreefloos", "Schrijfletters", "Symbolen en iconen", "MIJN BIBLIOTHEKEN", "Nieuwe bibliotheek",
    "Kies een map waar dan ook: alle lettertypen erin worden toegevoegd.", "Map", "Kies…", "Naam van bibliotheek",
    "Mapnaam gebruiken", "Eigen naam", "{n} lettertypen gevonden", "Bibliotheek maken", "Annuleer", "Bibliotheek verwijderen",
    "Inactief – klik om te activeren zodat andere apps het kunnen gebruiken", "Actief – klik om te deactiveren",
    "Altijd beschikbaar: systeemlettertype dat niet uit kan", "Altijd beschikbaar: geïnstalleerd op deze Mac",
    "Kon dit lettertype niet activeren", "Die map bestaat niet",
];

const SV: [&str; N3] = [
    "Med seriffer", "Sans serif", "Skrivstil", "Symboler och ikoner", "MINA BIBLIOTEK", "Nytt bibliotek",
    "Välj en mapp var som helst: alla typsnitt i den läggs till.", "Mapp", "Välj…", "Bibliotekets namn",
    "Använd mappens namn", "Eget namn", "{n} typsnitt hittades", "Skapa bibliotek", "Avbryt", "Ta bort bibliotek",
    "Inaktivt – klicka för att aktivera så att andra appar kan använda det", "Aktivt – klicka för att avaktivera",
    "Alltid tillgängligt: systemtypsnitt som inte kan stängas av", "Alltid tillgängligt: installerat på den här Macen",
    "Kunde inte aktivera typsnittet", "Mappen finns inte",
];

const DA: [&str; N3] = [
    "Med seriffer", "Uden seriffer", "Skriveskrift", "Symboler og ikoner", "MINE BIBLIOTEKER", "Nyt bibliotek",
    "Vælg en mappe hvor som helst: alle skrifttyper i den tilføjes.", "Mappe", "Vælg…", "Bibliotekets navn",
    "Brug mappens navn", "Eget navn", "{n} skrifttyper fundet", "Opret bibliotek", "Annuller", "Fjern bibliotek",
    "Inaktiv – klik for at aktivere, så andre apps kan bruge den", "Aktiv – klik for at deaktivere",
    "Altid tilgængelig: systemskrifttype, der ikke kan slås fra", "Altid tilgængelig: installeret på denne Mac",
    "Kunne ikke aktivere skrifttypen", "Mappen findes ikke",
];

const NB: [&str; N3] = [
    "Med seriffer", "Uten seriffer", "Skriveskrift", "Symboler og ikoner", "MINE BIBLIOTEKER", "Nytt bibliotek",
    "Velg en mappe hvor som helst: alle skriftene i den legges til.", "Mappe", "Velg …", "Bibliotekets navn",
    "Bruk mappenavnet", "Eget navn", "{n} skrifter funnet", "Opprett bibliotek", "Avbryt", "Fjern bibliotek",
    "Inaktiv – klikk for å aktivere så andre apper kan bruke den", "Aktiv – klikk for å deaktivere",
    "Alltid tilgjengelig: systemskrift som ikke kan slås av", "Alltid tilgjengelig: installert på denne Macen",
    "Kunne ikke aktivere skriften", "Mappen finnes ikke",
];

const FI: [&str; N3] = [
    "Päätteelliset", "Päätteettömät", "Käsialafontit", "Symbolit ja kuvakkeet", "OMAT KIRJASTOT", "Uusi kirjasto",
    "Valitse kansio mistä tahansa: kaikki sen fontit lisätään.", "Kansio", "Valitse…", "Kirjaston nimi",
    "Käytä kansion nimeä", "Oma nimi", "{n} fonttia löytyi", "Luo kirjasto", "Kumoa", "Poista kirjasto",
    "Ei käytössä – ota käyttöön klikkaamalla, jotta muut apit voivat käyttää sitä", "Käytössä – poista käytöstä klikkaamalla",
    "Aina saatavilla: järjestelmäfontti, jota ei voi poistaa käytöstä", "Aina saatavilla: asennettu tähän Maciin",
    "Fontin käyttöönotto epäonnistui", "Kansiota ei ole",
];

const PL: [&str; N3] = [
    "Szeryfowe", "Bezszeryfowe", "Pisane", "Symbole i ikony", "MOJE BIBLIOTEKI", "Nowa biblioteka",
    "Wybierz dowolny folder: zostaną dodane wszystkie czcionki z niego.", "Folder", "Wybierz…", "Nazwa biblioteki",
    "Użyj nazwy folderu", "Własna nazwa", "Znalezione czcionki: {n}", "Utwórz bibliotekę", "Anuluj", "Usuń bibliotekę",
    "Nieaktywna – kliknij, aby aktywować i udostępnić innym aplikacjom", "Aktywna – kliknij, aby dezaktywować",
    "Zawsze dostępna: czcionka systemowa, której nie można wyłączyć", "Zawsze dostępna: zainstalowana na tym Macu",
    "Nie udało się aktywować czcionki", "Ten folder nie istnieje",
];

const CS: [&str; N3] = [
    "Patková", "Bezpatková", "Psací", "Symboly a ikony", "MOJE KNIHOVNY", "Nová knihovna",
    "Vyberte libovolnou složku: přidají se všechna písma v ní.", "Složka", "Vybrat…", "Název knihovny",
    "Použít název složky", "Vlastní název", "Nalezená písma: {n}", "Vytvořit knihovnu", "Zrušit", "Odstranit knihovnu",
    "Neaktivní – kliknutím aktivujete, aby ho mohly používat jiné aplikace", "Aktivní – kliknutím deaktivujete",
    "Vždy dostupné: systémové písmo, které nelze vypnout", "Vždy dostupné: nainstalováno v tomto Macu",
    "Písmo se nepodařilo aktivovat", "Tato složka neexistuje",
];

const TR: [&str; N3] = [
    "Tırnaklı", "Tırnaksız", "El yazısı", "Semboller ve simgeler", "KİTAPLIKLARIM", "Yeni kitaplık",
    "Herhangi bir klasör seçin: içindeki tüm yazı tipleri eklenir.", "Klasör", "Seç…", "Kitaplık adı",
    "Klasörün adını kullan", "Özel ad", "{n} yazı tipi bulundu", "Kitaplık oluştur", "Vazgeç", "Kitaplığı kaldır",
    "Etkin değil – diğer uygulamalar kullanabilsin diye etkinleştirmek için tıklayın", "Etkin – devre dışı bırakmak için tıklayın",
    "Her zaman kullanılabilir: kapatılamayan sistem yazı tipi", "Her zaman kullanılabilir: bu Mac'te yüklü",
    "Yazı tipi etkinleştirilemedi", "Bu klasör yok",
];

const RO: [&str; N3] = [
    "Cu serife", "Fără serife", "Scris de mână", "Simboluri și pictograme", "BIBLIOTECILE MELE", "Bibliotecă nouă",
    "Alege un dosar de oriunde: toate fonturile din el vor fi adăugate.", "Dosar", "Alege…", "Numele bibliotecii",
    "Folosește numele dosarului", "Nume personalizat", "{n} fonturi găsite", "Creează biblioteca", "Anulează", "Elimină biblioteca",
    "Inactiv – dă clic ca să-l activezi și să-l poată folosi alte aplicații", "Activ – dă clic ca să-l dezactivezi",
    "Mereu disponibil: font de sistem care nu poate fi oprit", "Mereu disponibil: instalat pe acest Mac",
    "Fontul nu a putut fi activat", "Dosarul nu există",
];

const HU: [&str; N3] = [
    "Talpas", "Talpatlan", "Kézírásos", "Szimbólumok és ikonok", "SAJÁT KÖNYVTÁRAK", "Új könyvtár",
    "Válassz bármilyen mappát: a benne lévő összes betűtípus bekerül.", "Mappa", "Kiválasztás…", "Könyvtár neve",
    "A mappa nevének használata", "Egyéni név", "{n} betűtípus található", "Könyvtár létrehozása", "Mégse", "Könyvtár eltávolítása",
    "Inaktív – kattints az aktiváláshoz, hogy más appok is használhassák", "Aktív – kattints a kikapcsoláshoz",
    "Mindig elérhető: nem kikapcsolható rendszer-betűtípus", "Mindig elérhető: telepítve ezen a Macen",
    "Nem sikerült aktiválni a betűtípust", "Ez a mappa nem létezik",
];

const EL: [&str; N3] = [
    "Με πατούρες", "Χωρίς πατούρες", "Καλλιγραφικές", "Σύμβολα και εικονίδια", "ΟΙ ΒΙΒΛΙΟΘΗΚΕΣ ΜΟΥ", "Νέα βιβλιοθήκη",
    "Επιλέξτε οποιονδήποτε φάκελο: θα προστεθούν όλες οι γραμματοσειρές του.", "Φάκελος", "Επιλογή…", "Όνομα βιβλιοθήκης",
    "Χρήση του ονόματος του φακέλου", "Προσαρμοσμένο όνομα", "Βρέθηκαν {n} γραμματοσειρές", "Δημιουργία βιβλιοθήκης", "Ακύρωση", "Αφαίρεση βιβλιοθήκης",
    "Ανενεργή – κάντε κλικ για ενεργοποίηση ώστε να τη χρησιμοποιούν άλλες εφαρμογές", "Ενεργή – κάντε κλικ για απενεργοποίηση",
    "Πάντα διαθέσιμη: γραμματοσειρά συστήματος που δεν απενεργοποιείται", "Πάντα διαθέσιμη: εγκατεστημένη σε αυτό το Mac",
    "Δεν ήταν δυνατή η ενεργοποίηση", "Ο φάκελος δεν υπάρχει",
];

const RU: [&str; N3] = [
    "С засечками", "Без засечек", "Рукописные", "Символы и значки", "МОИ БИБЛИОТЕКИ", "Новая библиотека",
    "Выберите любую папку: все шрифты из неё будут добавлены.", "Папка", "Выбрать…", "Название библиотеки",
    "Использовать имя папки", "Своё название", "Найдено шрифтов: {n}", "Создать библиотеку", "Отменить", "Удалить библиотеку",
    "Не активен — нажмите, чтобы активировать и дать доступ другим приложениям", "Активен — нажмите, чтобы отключить",
    "Всегда доступен: системный шрифт, его нельзя отключить", "Всегда доступен: установлен на этом Mac",
    "Не удалось активировать шрифт", "Такой папки нет",
];

const UK: [&str; N3] = [
    "Із зарубками", "Без зарубок", "Рукописні", "Символи та значки", "МОЇ БІБЛІОТЕКИ", "Нова бібліотека",
    "Виберіть будь-яку папку: усі шрифти з неї буде додано.", "Папка", "Вибрати…", "Назва бібліотеки",
    "Використати назву папки", "Власна назва", "Знайдено шрифтів: {n}", "Створити бібліотеку", "Скасувати", "Видалити бібліотеку",
    "Неактивний — натисніть, щоб активувати й дати доступ іншим програмам", "Активний — натисніть, щоб вимкнути",
    "Завжди доступний: системний шрифт, його не можна вимкнути", "Завжди доступний: встановлено на цьому Mac",
    "Не вдалося активувати шрифт", "Такої папки немає",
];

const ID: [&str; N3] = [
    "Berkait", "Tanpa kait", "Tulisan tangan", "Simbol & ikon", "PUSTAKA SAYA", "Pustaka baru",
    "Pilih folder di mana saja: semua font di dalamnya akan ditambahkan.", "Folder", "Pilih…", "Nama pustaka",
    "Pakai nama folder", "Nama khusus", "{n} font ditemukan", "Buat pustaka", "Batal", "Hapus pustaka",
    "Tidak aktif — klik untuk mengaktifkan agar app lain bisa memakainya", "Aktif — klik untuk menonaktifkan",
    "Selalu tersedia: font sistem yang tidak bisa dimatikan", "Selalu tersedia: terpasang di Mac ini",
    "Font ini tidak bisa diaktifkan", "Folder itu tidak ada",
];

const VI: [&str; N3] = [
    "Có chân", "Không chân", "Chữ viết tay", "Ký hiệu & biểu tượng", "THƯ VIỆN CỦA TÔI", "Thư viện mới",
    "Chọn một thư mục bất kỳ: mọi phông bên trong sẽ được thêm vào.", "Thư mục", "Chọn…", "Tên thư viện",
    "Dùng tên thư mục", "Tên tùy chọn", "Tìm thấy {n} phông", "Tạo thư viện", "Hủy", "Xóa thư viện",
    "Chưa kích hoạt — bấm để kích hoạt cho ứng dụng khác dùng", "Đang kích hoạt — bấm để tắt",
    "Luôn sẵn có: phông hệ thống không thể tắt", "Luôn sẵn có: đã cài trên máy Mac này",
    "Không kích hoạt được phông này", "Thư mục đó không tồn tại",
];

const ZH_HANS: [&str; N3] = [
    "衬线体", "无衬线体", "手写体", "符号与图标", "我的字体库", "新建字体库",
    "选择任意文件夹：其中的所有字体都会被添加。", "文件夹", "选择…", "字体库名称",
    "使用文件夹名称", "自定义名称", "找到 {n} 个字体", "创建字体库", "取消", "移除字体库",
    "未激活 — 点按即可激活，供其他应用使用", "已激活 — 点按即可停用",
    "始终可用：无法停用的系统字体", "始终可用：已安装在此 Mac 上",
    "无法激活此字体", "该文件夹不存在",
];

const ZH_HANT: [&str; N3] = [
    "襯線體", "無襯線體", "手寫體", "符號與圖示", "我的字體庫", "新增字體庫",
    "選擇任意資料夾：其中所有字體都會加入。", "資料夾", "選擇…", "字體庫名稱",
    "使用資料夾名稱", "自訂名稱", "找到 {n} 個字體", "建立字體庫", "取消", "移除字體庫",
    "未啟用 — 按一下即可啟用，供其他 App 使用", "已啟用 — 按一下即可停用",
    "永遠可用：無法停用的系統字體", "永遠可用：已安裝在這台 Mac 上",
    "無法啟用此字體", "該資料夾不存在",
];

const JA: [&str; N3] = [
    "セリフ体", "サンセリフ体", "スクリプト体", "記号とアイコン", "マイライブラリ", "新規ライブラリ",
    "任意のフォルダを選ぶと、中のフォントがすべて追加されます。", "フォルダ", "選択…", "ライブラリ名",
    "フォルダ名を使う", "名前を指定", "{n} 個のフォントが見つかりました", "ライブラリを作成", "キャンセル", "ライブラリを削除",
    "無効 — クリックで有効にすると他のアプリでも使えます", "有効 — クリックで無効にします",
    "常に利用可能：無効にできないシステムフォント", "常に利用可能：この Mac にインストール済み",
    "このフォントを有効にできませんでした", "そのフォルダは存在しません",
];

const KO: [&str; N3] = [
    "세리프", "산세리프", "스크립트", "기호 및 아이콘", "내 보관함", "새 보관함",
    "아무 폴더나 고르면 그 안의 모든 서체가 추가됩니다.", "폴더", "선택…", "보관함 이름",
    "폴더 이름 사용", "직접 입력", "서체 {n}개 찾음", "보관함 만들기", "취소", "보관함 제거",
    "비활성 — 클릭하면 활성화되어 다른 앱에서 쓸 수 있습니다", "활성 — 클릭하면 비활성화합니다",
    "항상 사용 가능: 끌 수 없는 시스템 서체", "항상 사용 가능: 이 Mac에 설치됨",
    "이 서체를 활성화할 수 없습니다", "해당 폴더가 없습니다",
];
