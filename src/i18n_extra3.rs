//! Words for importing RightFont libraries and previewing subfolders, in the same language order as `i18n::LANGS`.

pub const N4: usize = 6;

pub const EXTRA3: [&[&str; N4]; 25] = [
    &["Import RightFont Library…", "Choose a RightFont library or its folder: its font lists and their order are kept.", "Subfolders", "In the main folder", "No subfolders with fonts", "Import Library"],
    &["Importar biblioteca de RightFont…", "Elige una biblioteca de RightFont o su carpeta: se conservan sus listas y su orden.", "Subcarpetas", "En la carpeta principal", "No hay subcarpetas con fuentes", "Importar biblioteca"],
    &["Importer une bibliothèque RightFont…", "Choisissez une bibliothèque RightFont ou son dossier : ses listes et leur ordre sont conservés.", "Sous-dossiers", "Dans le dossier principal", "Aucun sous-dossier avec des polices", "Importer la bibliothèque"],
    &["RightFont-Bibliothek importieren …", "Wähle eine RightFont-Bibliothek oder ihren Ordner: Listen und Reihenfolge bleiben erhalten.", "Unterordner", "Im Hauptordner", "Keine Unterordner mit Schriften", "Bibliothek importieren"],
    &["Importa libreria RightFont…", "Scegli una libreria RightFont o la sua cartella: elenchi e ordine vengono mantenuti.", "Sottocartelle", "Nella cartella principale", "Nessuna sottocartella con font", "Importa libreria"],
    &["Importar biblioteca do RightFont…", "Escolha uma biblioteca do RightFont ou a pasta dela: as listas e a ordem são mantidas.", "Subpastas", "Na pasta principal", "Nenhuma subpasta com fontes", "Importar biblioteca"],
    &["RightFont-bibliotheek importeren…", "Kies een RightFont-bibliotheek of de map ervan: lijsten en volgorde blijven behouden.", "Submappen", "In de hoofdmap", "Geen submappen met lettertypen", "Bibliotheek importeren"],
    &["Importera RightFont-bibliotek…", "Välj ett RightFont-bibliotek eller dess mapp: listor och ordning behålls.", "Undermappar", "I huvudmappen", "Inga undermappar med typsnitt", "Importera bibliotek"],
    &["Importér RightFont-bibliotek…", "Vælg et RightFont-bibliotek eller dets mappe: lister og rækkefølge bevares.", "Undermapper", "I hovedmappen", "Ingen undermapper med skrifttyper", "Importér bibliotek"],
    &["Importer RightFont-bibliotek …", "Velg et RightFont-bibliotek eller mappen dets: lister og rekkefølge beholdes.", "Undermapper", "I hovedmappen", "Ingen undermapper med skrifter", "Importer bibliotek"],
    &["Tuo RightFont-kirjasto…", "Valitse RightFont-kirjasto tai sen kansio: luettelot ja järjestys säilyvät.", "Alikansiot", "Pääkansiossa", "Ei fontteja sisältäviä alikansioita", "Tuo kirjasto"],
    &["Importuj bibliotekę RightFont…", "Wybierz bibliotekę RightFont lub jej folder: listy i ich kolejność zostaną zachowane.", "Podfoldery", "W folderze głównym", "Brak podfolderów z czcionkami", "Importuj bibliotekę"],
    &["Importovat knihovnu RightFont…", "Vyberte knihovnu RightFont nebo její složku: seznamy i jejich pořadí zůstanou zachovány.", "Podsložky", "V hlavní složce", "Žádné podsložky s písmy", "Importovat knihovnu"],
    &["RightFont kitaplığını içe aktar…", "Bir RightFont kitaplığı ya da klasörünü seçin: listeler ve sıraları korunur.", "Alt klasörler", "Ana klasörde", "Yazı tipi içeren alt klasör yok", "Kitaplığı içe aktar"],
    &["Importă bibliotecă RightFont…", "Alege o bibliotecă RightFont sau dosarul ei: listele și ordinea lor se păstrează.", "Subdosare", "În dosarul principal", "Niciun subdosar cu fonturi", "Importă biblioteca"],
    &["RightFont-könyvtár importálása…", "Válassz egy RightFont-könyvtárat vagy a mappáját: a listák és sorrendjük megmarad.", "Almappák", "A fő mappában", "Nincs betűtípust tartalmazó almappa", "Könyvtár importálása"],
    &["Εισαγωγή βιβλιοθήκης RightFont…", "Επιλέξτε μια βιβλιοθήκη RightFont ή τον φάκελό της: οι λίστες και η σειρά τους διατηρούνται.", "Υποφάκελοι", "Στον κύριο φάκελο", "Κανένας υποφάκελος με γραμματοσειρές", "Εισαγωγή βιβλιοθήκης"],
    &["Импортировать библиотеку RightFont…", "Выберите библиотеку RightFont или её папку: списки и их порядок сохранятся.", "Подпапки", "В основной папке", "Нет подпапок со шрифтами", "Импортировать библиотеку"],
    &["Імпортувати бібліотеку RightFont…", "Виберіть бібліотеку RightFont або її папку: списки та їх порядок збережуться.", "Підпапки", "В основній папці", "Немає підпапок зі шрифтами", "Імпортувати бібліотеку"],
    &["Impor pustaka RightFont…", "Pilih pustaka RightFont atau foldernya: daftar dan urutannya dipertahankan.", "Subfolder", "Di folder utama", "Tidak ada subfolder berisi font", "Impor pustaka"],
    &["Nhập thư viện RightFont…", "Chọn một thư viện RightFont hoặc thư mục của nó: các danh sách và thứ tự được giữ nguyên.", "Thư mục con", "Trong thư mục chính", "Không có thư mục con chứa phông", "Nhập thư viện"],
    &["导入 RightFont 字体库…", "选择 RightFont 字体库或其文件夹：其中的列表及顺序都会保留。", "子文件夹", "在主文件夹中", "没有包含字体的子文件夹", "导入字体库"],
    &["匯入 RightFont 字體庫…", "選擇 RightFont 字體庫或其資料夾：其中的列表及順序都會保留。", "子資料夾", "在主資料夾中", "沒有包含字體的子資料夾", "匯入字體庫"],
    &["RightFont ライブラリを読み込む…", "RightFont のライブラリまたはそのフォルダを選ぶと、リストと並び順が保たれます。", "サブフォルダ", "メインフォルダ内", "フォントを含むサブフォルダはありません", "ライブラリを読み込む"],
    &["RightFont 보관함 가져오기…", "RightFont 보관함이나 그 폴더를 고르세요. 목록과 순서가 유지됩니다.", "하위 폴더", "기본 폴더", "서체가 있는 하위 폴더 없음", "보관함 가져오기"],
];
