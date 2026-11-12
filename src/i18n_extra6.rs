//! Words for My Folders, FontCraft libraries (.fcl), the Import Fonts window, sharing and
//! permissions, in the same language order as `i18n::LANGS`.

pub const N7: usize = 18;

// MyFoldersHead, AddFolder, AddMore, CreateNewHead, FclHint, ShareWith, AllUsers, ApplyBtn, SharedWith,
// ShareFailed, NoOtherUsers, ShareNote, PermissionsHead, AccessAllowed, AccessNotAllowed, OpenSecurity,
// AllowAutomation, Done
const EN: [&str; N7] = [
    "MY FOLDERS", "Add Folder", "Add…", "Create new",
    "Saved as {file} in {dir}. Its fonts are sorted into folders by first letter (A, B, C…).",
    "Share settings with", "All users", "Apply", "Shared with {who}.", "Couldn't share: {err}",
    "No other users on this computer", "FontCraft keeps its libraries in {dir}.", "Permissions",
    "Control other apps: allowed", "Control other apps: not allowed yet", "Open Security Settings", "Automation…", "Done",
];
const ES: [&str; N7] = [
    "MIS CARPETAS", "Añadir carpeta", "Añadir…", "Crear nueva",
    "Se guarda como {file} en {dir}. Sus fuentes se ordenan en carpetas por la primera letra (A, B, C…).",
    "Compartir configuración con", "Con todos los usuarios", "Aplicar", "Compartido con {who}.", "No se pudo compartir: {err}",
    "No hay otros usuarios en este equipo", "FontCraft guarda sus bibliotecas en {dir}.", "Permisos",
    "Controlar otras apps: permitido", "Controlar otras apps: aún no permitido", "Abrir ajustes de seguridad", "Automatización…", "Listo",
];
const FR: [&str; N7] = [
    "MES DOSSIERS", "Ajouter un dossier", "Ajouter…", "Créer",
    "Enregistrée sous {file} dans {dir}. Ses polices sont rangées dans des dossiers par première lettre (A, B, C…).",
    "Partager les réglages avec", "Tous les utilisateurs", "Appliquer", "Partagé avec {who}.", "Partage impossible : {err}",
    "Aucun autre utilisateur sur cet ordinateur", "FontCraft range ses bibliothèques dans {dir}.", "Autorisations",
    "Contrôler d’autres apps : autorisé", "Contrôler d’autres apps : pas encore autorisé", "Ouvrir les réglages de sécurité", "Automatisation…", "OK",
];
const DE: [&str; N7] = [
    "MEINE ORDNER", "Ordner hinzufügen", "Hinzufügen …", "Neu erstellen",
    "Gespeichert als {file} in {dir}. Die Schriften werden nach Anfangsbuchstaben in Ordner sortiert (A, B, C …).",
    "Einstellungen teilen mit", "Allen Benutzern", "Anwenden", "Geteilt mit {who}.", "Teilen nicht möglich: {err}",
    "Keine anderen Benutzer auf diesem Computer", "FontCraft speichert seine Bibliotheken in {dir}.", "Berechtigungen",
    "Andere Apps steuern: erlaubt", "Andere Apps steuern: noch nicht erlaubt", "Sicherheitseinstellungen öffnen", "Automation …", "Fertig",
];
const IT: [&str; N7] = [
    "LE MIE CARTELLE", "Aggiungi cartella", "Aggiungi…", "Crea nuova",
    "Salvata come {file} in {dir}. I suoi font sono ordinati in cartelle per iniziale (A, B, C…).",
    "Condividi impostazioni con", "Tutti gli utenti", "Applica", "Condiviso con {who}.", "Impossibile condividere: {err}",
    "Nessun altro utente su questo computer", "FontCraft tiene le sue librerie in {dir}.", "Permessi",
    "Controllare altre app: consentito", "Controllare altre app: non ancora consentito", "Apri impostazioni di sicurezza", "Automazione…", "Fine",
];
const PT: [&str; N7] = [
    "MINHAS PASTAS", "Adicionar pasta", "Adicionar…", "Criar nova",
    "Salva como {file} em {dir}. As fontes são organizadas em pastas pela primeira letra (A, B, C…).",
    "Compartilhar ajustes com", "Todos os usuários", "Aplicar", "Compartilhado com {who}.", "Não foi possível compartilhar: {err}",
    "Não há outros usuários neste computador", "O FontCraft guarda suas bibliotecas em {dir}.", "Permissões",
    "Controlar outros apps: permitido", "Controlar outros apps: ainda não permitido", "Abrir ajustes de segurança", "Automação…", "OK",
];
const NL: [&str; N7] = [
    "MIJN MAPPEN", "Map toevoegen", "Voeg toe…", "Nieuw maken",
    "Bewaard als {file} in {dir}. De lettertypen worden per beginletter in mappen gezet (A, B, C…).",
    "Instellingen delen met", "Alle gebruikers", "Pas toe", "Gedeeld met {who}.", "Delen mislukt: {err}",
    "Geen andere gebruikers op deze computer", "FontCraft bewaart zijn bibliotheken in {dir}.", "Toestemmingen",
    "Andere apps bedienen: toegestaan", "Andere apps bedienen: nog niet toegestaan", "Open beveiligingsinstellingen", "Automatisering…", "Gereed",
];
const SV: [&str; N7] = [
    "MINA MAPPAR", "Lägg till mapp", "Lägg till …", "Skapa ny",
    "Sparas som {file} i {dir}. Typsnitten sorteras i mappar efter första bokstav (A, B, C …).",
    "Dela inställningar med", "Alla användare", "Verkställ", "Delas med {who}.", "Kunde inte dela: {err}",
    "Inga andra användare på den här datorn", "FontCraft sparar sina bibliotek i {dir}.", "Behörigheter",
    "Styra andra appar: tillåtet", "Styra andra appar: inte tillåtet än", "Öppna säkerhetsinställningar", "Automation …", "Klar",
];
const DA: [&str; N7] = [
    "MINE MAPPER", "Tilføj mappe", "Tilføj …", "Opret ny",
    "Gemmes som {file} i {dir}. Skrifterne sorteres i mapper efter forbogstav (A, B, C …).",
    "Del indstillinger med", "Alle brugere", "Anvend", "Delt med {who}.", "Kunne ikke dele: {err}",
    "Ingen andre brugere på denne computer", "FontCraft gemmer sine biblioteker i {dir}.", "Tilladelser",
    "Styre andre apps: tilladt", "Styre andre apps: endnu ikke tilladt", "Åbn sikkerhedsindstillinger", "Automatisering …", "Færdig",
];
const NB: [&str; N7] = [
    "MINE MAPPER", "Legg til mappe", "Legg til …", "Opprett ny",
    "Lagres som {file} i {dir}. Skriftene sorteres i mapper etter forbokstav (A, B, C …).",
    "Del innstillinger med", "Alle brukere", "Bruk", "Delt med {who}.", "Kunne ikke dele: {err}",
    "Ingen andre brukere på denne datamaskinen", "FontCraft lagrer bibliotekene sine i {dir}.", "Tillatelser",
    "Styre andre apper: tillatt", "Styre andre apper: ikke tillatt ennå", "Åpne sikkerhetsinnstillinger", "Automatisering …", "Ferdig",
];
const FI: [&str; N7] = [
    "OMAT KANSIOT", "Lisää kansio", "Lisää…", "Luo uusi",
    "Tallennetaan nimellä {file} kansioon {dir}. Fontit järjestetään kansioihin alkukirjaimen mukaan (A, B, C…).",
    "Jaa asetukset", "Kaikille käyttäjille", "Käytä", "Jaettu: {who}.", "Jakaminen epäonnistui: {err}",
    "Tällä tietokoneella ei ole muita käyttäjiä", "FontCraft säilyttää kirjastonsa kansiossa {dir}.", "Luvat",
    "Muiden apien ohjaus: sallittu", "Muiden apien ohjaus: ei vielä sallittu", "Avaa suojausasetukset", "Automaatio…", "Valmis",
];
const PL: [&str; N7] = [
    "MOJE FOLDERY", "Dodaj folder", "Dodaj…", "Utwórz nową",
    "Zapisana jako {file} w {dir}. Fonty są układane w folderach według pierwszej litery (A, B, C…).",
    "Udostępnij ustawienia", "Wszystkim użytkownikom", "Zastosuj", "Udostępniono: {who}.", "Nie udało się udostępnić: {err}",
    "Brak innych użytkowników na tym komputerze", "FontCraft trzyma biblioteki w {dir}.", "Uprawnienia",
    "Sterowanie innymi aplikacjami: dozwolone", "Sterowanie innymi aplikacjami: jeszcze niedozwolone", "Otwórz ustawienia zabezpieczeń", "Automatyzacja…", "Gotowe",
];
const CS: [&str; N7] = [
    "MOJE SLOŽKY", "Přidat složku", "Přidat…", "Vytvořit novou",
    "Uloží se jako {file} do {dir}. Písma se řadí do složek podle prvního písmene (A, B, C…).",
    "Sdílet nastavení s", "Všemi uživateli", "Použít", "Sdíleno s {who}.", "Sdílení se nezdařilo: {err}",
    "Na tomto počítači nejsou jiní uživatelé", "FontCraft ukládá knihovny do {dir}.", "Oprávnění",
    "Ovládání jiných aplikací: povoleno", "Ovládání jiných aplikací: zatím nepovoleno", "Otevřít nastavení zabezpečení", "Automatizace…", "Hotovo",
];
const TR: [&str; N7] = [
    "KLASÖRLERİM", "Klasör ekle", "Ekle…", "Yeni oluştur",
    "{dir} içinde {file} olarak kaydedilir. Fontlar ilk harfe göre klasörlere ayrılır (A, B, C…).",
    "Ayarları paylaş", "Tüm kullanıcılar", "Uygula", "{who} ile paylaşıldı.", "Paylaşılamadı: {err}",
    "Bu bilgisayarda başka kullanıcı yok", "FontCraft kitaplıklarını {dir} içinde tutar.", "İzinler",
    "Diğer uygulamaları denetleme: izinli", "Diğer uygulamaları denetleme: henüz izin yok", "Güvenlik ayarlarını aç", "Otomasyon…", "Bitti",
];
const RO: [&str; N7] = [
    "DOSARELE MELE", "Adaugă dosar", "Adaugă…", "Creează nouă",
    "Se salvează ca {file} în {dir}. Fonturile sunt așezate în dosare după prima literă (A, B, C…).",
    "Partajează configurarea cu", "Toți utilizatorii", "Aplică", "Partajat cu {who}.", "Nu s-a putut partaja: {err}",
    "Nu există alți utilizatori pe acest computer", "FontCraft își ține bibliotecile în {dir}.", "Permisiuni",
    "Controlul altor aplicații: permis", "Controlul altor aplicații: încă nepermis", "Deschide setările de securitate", "Automatizare…", "Gata",
];
const HU: [&str; N7] = [
    "MAPPÁIM", "Mappa hozzáadása", "Hozzáadás…", "Új létrehozása",
    "Mentés {file} néven ide: {dir}. A betűtípusok kezdőbetű szerint mappákba kerülnek (A, B, C…).",
    "Beállítások megosztása", "Minden felhasználóval", "Alkalmaz", "Megosztva: {who}.", "A megosztás nem sikerült: {err}",
    "Nincs más felhasználó ezen a számítógépen", "A FontCraft itt tárolja a könyvtárait: {dir}.", "Engedélyek",
    "Más appok vezérlése: engedélyezve", "Más appok vezérlése: még nincs engedélyezve", "Biztonsági beállítások megnyitása", "Automatizálás…", "Kész",
];
const EL: [&str; N7] = [
    "ΟΙ ΦΑΚΕΛΟΙ ΜΟΥ", "Προσθήκη φακέλου", "Προσθήκη…", "Δημιουργία νέας",
    "Αποθηκεύεται ως {file} στο {dir}. Οι γραμματοσειρές ταξινομούνται σε φακέλους ανά αρχικό γράμμα (A, B, C…).",
    "Κοινή χρήση ρυθμίσεων με", "Όλους τους χρήστες", "Εφαρμογή", "Κοινόχρηστο με {who}.", "Αδυναμία κοινής χρήσης: {err}",
    "Δεν υπάρχουν άλλοι χρήστες σε αυτόν τον υπολογιστή", "Το FontCraft κρατά τις βιβλιοθήκες του στο {dir}.", "Άδειες",
    "Έλεγχος άλλων εφαρμογών: επιτρέπεται", "Έλεγχος άλλων εφαρμογών: δεν επιτρέπεται ακόμη", "Άνοιγμα ρυθμίσεων ασφαλείας", "Αυτοματοποίηση…", "Τέλος",
];
const RU: [&str; N7] = [
    "МОИ ПАПКИ", "Добавить папку", "Добавить…", "Создать",
    "Сохраняется как {file} в {dir}. Шрифты раскладываются по папкам по первой букве (A, B, C…).",
    "Поделиться настройками с", "Всеми пользователями", "Применить", "Доступно: {who}.", "Не удалось поделиться: {err}",
    "На этом компьютере нет других пользователей", "FontCraft хранит библиотеки в {dir}.", "Разрешения",
    "Управление другими приложениями: разрешено", "Управление другими приложениями: пока не разрешено", "Открыть настройки безопасности", "Автоматизация…", "Готово",
];
const UK: [&str; N7] = [
    "МОЇ ТЕКИ", "Додати теку", "Додати…", "Створити",
    "Зберігається як {file} у {dir}. Шрифти розкладаються по теках за першою літерою (A, B, C…).",
    "Поділитися налаштуваннями з", "Усіма користувачами", "Застосувати", "Доступно: {who}.", "Не вдалося поділитися: {err}",
    "На цьому комп’ютері немає інших користувачів", "FontCraft зберігає бібліотеки в {dir}.", "Дозволи",
    "Керування іншими програмами: дозволено", "Керування іншими програмами: ще не дозволено", "Відкрити параметри безпеки", "Автоматизація…", "Готово",
];
const ID: [&str; N7] = [
    "FOLDER SAYA", "Tambah folder", "Tambah…", "Buat baru",
    "Disimpan sebagai {file} di {dir}. Fonnya diurutkan ke folder menurut huruf pertama (A, B, C…).",
    "Bagikan pengaturan dengan", "Semua pengguna", "Terapkan", "Dibagikan dengan {who}.", "Gagal membagikan: {err}",
    "Tidak ada pengguna lain di komputer ini", "FontCraft menyimpan pustakanya di {dir}.", "Izin",
    "Mengendalikan app lain: diizinkan", "Mengendalikan app lain: belum diizinkan", "Buka pengaturan keamanan", "Otomatisasi…", "Selesai",
];
const VI: [&str; N7] = [
    "THƯ MỤC CỦA TÔI", "Thêm thư mục", "Thêm…", "Tạo mới",
    "Lưu thành {file} trong {dir}. Các phông được xếp vào thư mục theo chữ cái đầu (A, B, C…).",
    "Chia sẻ cài đặt với", "Tất cả người dùng", "Áp dụng", "Đã chia sẻ với {who}.", "Không thể chia sẻ: {err}",
    "Không có người dùng khác trên máy này", "FontCraft lưu thư viện trong {dir}.", "Quyền",
    "Điều khiển ứng dụng khác: đã cho phép", "Điều khiển ứng dụng khác: chưa cho phép", "Mở cài đặt bảo mật", "Tự động hóa…", "Xong",
];
const ZH_HANS: [&str; N7] = [
    "我的文件夹", "添加文件夹", "添加…", "新建",
    "存储为 {dir} 中的 {file}。字体按首字母放入文件夹（A、B、C…）。",
    "共享设置给", "所有用户", "应用", "已与 {who} 共享。", "无法共享：{err}",
    "这台电脑上没有其他用户", "FontCraft 将资源库存放在 {dir}。", "权限",
    "控制其他应用：已允许", "控制其他应用：尚未允许", "打开安全性设置", "自动化…", "完成",
];
const ZH_HANT: [&str; N7] = [
    "我的檔案夾", "加入檔案夾", "加入…", "新增",
    "儲存為 {dir} 中的 {file}。字體依首字母放入檔案夾（A、B、C…）。",
    "共享設定給", "所有使用者", "套用", "已與 {who} 共享。", "無法共享：{err}",
    "這部電腦上沒有其他使用者", "FontCraft 將資料庫存放在 {dir}。", "權限",
    "控制其他 App：已允許", "控制其他 App：尚未允許", "打開安全性設定", "自動化…", "完成",
];
const JA: [&str; N7] = [
    "マイフォルダ", "フォルダを追加", "追加…", "新規作成",
    "{dir} に {file} として保存されます。フォントは頭文字ごとのフォルダに分けられます（A、B、C…）。",
    "設定を共有", "すべてのユーザ", "適用", "{who} と共有しました。", "共有できませんでした：{err}",
    "このコンピュータにほかのユーザはいません", "FontCraft はライブラリを {dir} に保存します。", "アクセス権",
    "ほかのアプリの操作：許可済み", "ほかのアプリの操作：まだ許可されていません", "セキュリティ設定を開く", "オートメーション…", "完了",
];
const KO: [&str; N7] = [
    "나의 폴더", "폴더 추가", "추가…", "새로 만들기",
    "{dir}에 {file}(으)로 저장됩니다. 서체는 첫 글자별 폴더로 정리됩니다(A, B, C…).",
    "설정 공유 대상", "모든 사용자", "적용", "{who}와(과) 공유했습니다.", "공유할 수 없음: {err}",
    "이 컴퓨터에 다른 사용자가 없습니다", "FontCraft는 라이브러리를 {dir}에 보관합니다.", "권한",
    "다른 앱 제어: 허용됨", "다른 앱 제어: 아직 허용되지 않음", "보안 설정 열기", "자동화…", "완료",
];

pub const EXTRA6: [&[&str; N7]; 25] = [
    &EN, &ES, &FR, &DE, &IT, &PT, &NL, &SV, &DA, &NB, &FI, &PL, &CS, &TR, &RO, &HU, &EL, &RU, &UK, &ID, &VI, &ZH_HANS,
    &ZH_HANT, &JA, &KO,
];
