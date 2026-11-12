//! Which languages a font can set: a language counts when the font has every letter it needs.
//! Languages are named in their own language, so the list reads the same in every interface language.

/// (name in its own language, the letters it needs beyond nothing else).
const LANGUAGES: &[(&str, &str)] = &[
    ("English", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"),
    ("Español", "abcdefghijklmnopqrstuvwxyzáéíóúüñÁÉÍÓÚÜÑ¿¡"),
    ("Français", "abcdefghijklmnopqrstuvwxyzàâçéèêëîïôûùüÿœæÀÂÇÉÈÊËÎÏÔÛÙÜŸŒÆ"),
    ("Deutsch", "abcdefghijklmnopqrstuvwxyzäöüßÄÖÜ"),
    ("Italiano", "abcdefghijklmnopqrstuvwxyzàèéìíîòóùúÀÈÉÌÒÙ"),
    ("Português", "abcdefghijklmnopqrstuvwxyzáâãàçéêíóôõúÁÂÃÀÇÉÊÍÓÔÕÚ"),
    ("Català", "abcdefghijklmnopqrstuvwxyzàçèéíïòóúüÀÇÈÉÍÏÒÓÚÜ·"),
    ("Nederlands", "abcdefghijklmnopqrstuvwxyzëïéèÉË"),
    ("Svenska", "abcdefghijklmnopqrstuvwxyzåäöÅÄÖ"),
    ("Dansk", "abcdefghijklmnopqrstuvwxyzæøåÆØÅ"),
    ("Norsk", "abcdefghijklmnopqrstuvwxyzæøåÆØÅ"),
    ("Suomi", "abcdefghijklmnopqrstuvwxyzäöÄÖ"),
    ("Íslenska", "abcdefghijklmnopqrstuvwxyzáðéíóúýþæöÁÐÉÍÓÚÝÞÆÖ"),
    ("Polski", "abcdefghijklmnopqrstuvwxyząćęłńóśźżĄĆĘŁŃÓŚŹŻ"),
    ("Čeština", "abcdefghijklmnopqrstuvwxyzáčďéěíňóřšťúůýžÁČĎÉĚÍŇÓŘŠŤÚŮÝŽ"),
    ("Slovenčina", "abcdefghijklmnopqrstuvwxyzáäčďéíĺľňóôŕšťúýžÁÄČĎÉÍĹĽŇÓÔŔŠŤÚÝŽ"),
    ("Magyar", "abcdefghijklmnopqrstuvwxyzáéíóöőúüűÁÉÍÓÖŐÚÜŰ"),
    ("Română", "abcdefghijklmnopqrstuvwxyzăâîșțĂÂÎȘȚ"),
    ("Hrvatski", "abcdefghijklmnopqrstuvwxyzčćđšžČĆĐŠŽ"),
    ("Slovenščina", "abcdefghijklmnopqrstuvwxyzčšžČŠŽ"),
    ("Türkçe", "abcdefghijklmnopqrstuvwxyzçğıöşüÇĞİÖŞÜ"),
    ("Lietuvių", "abcdefghijklmnopqrstuvwxyząčęėįšųūžĄČĘĖĮŠŲŪŽ"),
    ("Latviešu", "abcdefghijklmnopqrstuvwxyzāčēģīķļņšūžĀČĒĢĪĶĻŅŠŪŽ"),
    ("Eesti", "abcdefghijklmnopqrstuvwxyzäöõüšžÄÖÕÜŠŽ"),
    ("Tiếng Việt", "abcdefghijklmnopqrstuvwxyzăâđêôơưàảãáạằẳẵắặầẩẫấậèẻẽéẹềểễếệìỉĩíịòỏõóọồổỗốộờởỡớợùủũúụừửữứựỳỷỹýỵĐ"),
    ("Bahasa Indonesia", "abcdefghijklmnopqrstuvwxyz"),
    ("Kiswahili", "abcdefghijklmnopqrstuvwxyz"),
    ("Ελληνικά", "ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩαβγδεζηθικλμνξοπρσςτυφχψωάέήίόύώ"),
    ("Русский", "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя"),
    ("Українська", "АБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯабвгґдеєжзиіїйклмнопрстуфхцчшщьюя"),
    ("Български", "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЬЮЯабвгдежзийклмнопрстуфхцчшщъьюя"),
    ("Српски", "АБВГДЂЕЖЗИЈКЛЉМНЊОПРСТЋУФХЦЧЏШабвгдђежзијклљмнњопрстћуфхцчџш"),
    ("Қазақ", "ӘҒҚҢӨҰҮҺІәғқңөұүһі"),
    ("עברית", "אבגדהוזחטיכלמנסעפצקרשתךםןףץ"),
    ("العربية", "ابتثجحخدذرزسشصضطظعغفقكلمنهوي"),
    ("فارسی", "ابپتثجچحخدذرزژسشصضطظعغفقکگلمنوهی"),
    ("हिन्दी", "अआइईउऊएऐओऔकखगघचछजझटठडढणतथदधनपफबभमयरलवशषसह"),
    ("বাংলা", "অআইঈউঊএঐওঔকখগঘচছজঝটঠডঢণতথদধনপফবভমযরলশষসহ"),
    ("ไทย", "กขคฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ"),
    ("ქართული", "აბგდევზთიკლმნოპჟრსტუფქღყშჩცძწჭხჯჰ"),
    ("Հայերեն", "աբգդեզէըթժիլխծկհձղճմյնշոչպջռսվտրցւփքօֆ"),
    ("한국어", "가나다라마바사아자차카타파하한글"),
    ("日本語", "あいうえおかきくけこさしすせそアイウエオカキクケコ日本語"),
    ("简体中文", "的一是在不了有和人这中大为上个国我以要他"),
    ("繁體中文", "的一是在不了有和人這中大為上個國我以要他"),
];

pub fn supported(face: &ttf_parser::Face) -> Vec<&'static str> {
    LANGUAGES
        .iter()
        .filter(|(_, letters)| letters.chars().all(|c| face.glyph_index(c).is_some_and(|g| g.0 != 0)))
        .map(|(name, _)| *name)
        .collect()
}
