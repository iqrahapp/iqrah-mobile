// Complete list of all 114 Surahs with ayah counts
const SURAHS = [
    { number: 1, name: "Al-Fatihah", nameAr: "الفاتحة", ayahs: 7 },
    { number: 2, name: "Al-Baqarah", nameAr: "البقرة", ayahs: 286 },
    { number: 3, name: "Ali 'Imran", nameAr: "آل عمران", ayahs: 200 },
    { number: 4, name: "An-Nisa", nameAr: "النساء", ayahs: 176 },
    { number: 5, name: "Al-Ma'idah", nameAr: "المائدة", ayahs: 120 },
    { number: 6, name: "Al-An'am", nameAr: "الأنعام", ayahs: 165 },
    { number: 7, name: "Al-A'raf", nameAr: "الأعراف", ayahs: 206 },
    { number: 8, name: "Al-Anfal", nameAr: "الأنفال", ayahs: 75 },
    { number: 9, name: "At-Tawbah", nameAr: "التوبة", ayahs: 129 },
    { number: 10, name: "Yunus", nameAr: "يونس", ayahs: 109 },
    { number: 11, name: "Hud", nameAr: "هود", ayahs: 123 },
    { number: 12, name: "Yusuf", nameAr: "يوسف", ayahs: 111 },
    { number: 13, name: "Ar-Ra'd", nameAr: "الرعد", ayahs: 43 },
    { number: 14, name: "Ibrahim", nameAr: "ابراهيم", ayahs: 52 },
    { number: 15, name: "Al-Hijr", nameAr: "الحجر", ayahs: 99 },
    { number: 16, name: "An-Nahl", nameAr: "النحل", ayahs: 128 },
    { number: 17, name: "Al-Isra", nameAr: "الإسراء", ayahs: 111 },
    { number: 18, name: "Al-Kahf", nameAr: "الكهف", ayahs: 110 },
    { number: 19, name: "Maryam", nameAr: "مريم", ayahs: 98 },
    { number: 20, name: "Taha", nameAr: "طه", ayahs: 135 },
    { number: 21, name: "Al-Anbya", nameAr: "الأنبياء", ayahs: 112 },
    { number: 22, name: "Al-Hajj", nameAr: "الحج", ayahs: 78 },
    { number: 23, name: "Al-Mu'minun", nameAr: "المؤمنون", ayahs: 118 },
    { number: 24, name: "An-Nur", nameAr: "النور", ayahs: 64 },
    { number: 25, name: "Al-Furqan", nameAr: "الفرقان", ayahs: 77 },
    { number: 26, name: "Ash-Shu'ara", nameAr: "الشعراء", ayahs: 227 },
    { number: 27, name: "An-Naml", nameAr: "النمل", ayahs: 93 },
    { number: 28, name: "Al-Qasas", nameAr: "القصص", ayahs: 88 },
    { number: 29, name: "Al-'Ankabut", nameAr: "العنكبوت", ayahs: 69 },
    { number: 30, name: "Ar-Rum", nameAr: "الروم", ayahs: 60 },
    { number: 31, name: "Luqman", nameAr: "لقمان", ayahs: 34 },
    { number: 32, name: "As-Sajdah", nameAr: "السجدة", ayahs: 30 },
    { number: 33, name: "Al-Ahzab", nameAr: "الأحزاب", ayahs: 73 },
    { number: 34, name: "Saba", nameAr: "سبإ", ayahs: 54 },
    { number: 35, name: "Fatir", nameAr: "فاطر", ayahs: 45 },
    { number: 36, name: "Ya-Sin", nameAr: "يس", ayahs: 83 },
    { number: 37, name: "As-Saffat", nameAr: "الصافات", ayahs: 182 },
    { number: 38, name: "Sad", nameAr: "ص", ayahs: 88 },
    { number: 39, name: "Az-Zumar", nameAr: "الزمر", ayahs: 75 },
    { number: 40, name: "Ghafir", nameAr: "غافر", ayahs: 85 },
    { number: 41, name: "Fussilat", nameAr: "فصلت", ayahs: 54 },
    { number: 42, name: "Ash-Shuraa", nameAr: "الشورى", ayahs: 53 },
    { number: 43, name: "Az-Zukhruf", nameAr: "الزخرف", ayahs: 89 },
    { number: 44, name: "Ad-Dukhan", nameAr: "الدخان", ayahs: 59 },
    { number: 45, name: "Al-Jathiyah", nameAr: "الجاثية", ayahs: 37 },
    { number: 46, name: "Al-Ahqaf", nameAr: "الأحقاف", ayahs: 35 },
    { number: 47, name: "Muhammad", nameAr: "محمد", ayahs: 38 },
    { number: 48, name: "Al-Fath", nameAr: "الفتح", ayahs: 29 },
    { number: 49, name: "Al-Hujurat", nameAr: "الحجرات", ayahs: 18 },
    { number: 50, name: "Qaf", nameAr: "ق", ayahs: 45 },
    { number: 51, name: "Adh-Dhariyat", nameAr: "الذاريات", ayahs: 60 },
    { number: 52, name: "At-Tur", nameAr: "الطور", ayahs: 49 },
    { number: 53, name: "An-Najm", nameAr: "النجم", ayahs: 62 },
    { number: 54, name: "Al-Qamar", nameAr: "القمر", ayahs: 55 },
    { number: 55, name: "Ar-Rahman", nameAr: "الرحمن", ayahs: 78 },
    { number: 56, name: "Al-Waqi'ah", nameAr: "الواقعة", ayahs: 96 },
    { number: 57, name: "Al-Hadid", nameAr: "الحديد", ayahs: 29 },
    { number: 58, name: "Al-Mujadila", nameAr: "المجادلة", ayahs: 22 },
    { number: 59, name: "Al-Hashr", nameAr: "الحشر", ayahs: 24 },
    { number: 60, name: "Al-Mumtahanah", nameAr: "الممتحنة", ayahs: 13 },
    { number: 61, name: "As-Saf", nameAr: "الصف", ayahs: 14 },
    { number: 62, name: "Al-Jumu'ah", nameAr: "الجمعة", ayahs: 11 },
    { number: 63, name: "Al-Munafiqun", nameAr: "المنافقون", ayahs: 11 },
    { number: 64, name: "At-Taghabun", nameAr: "التغابن", ayahs: 18 },
    { number: 65, name: "At-Talaq", nameAr: "الطلاق", ayahs: 12 },
    { number: 66, name: "At-Tahrim", nameAr: "التحريم", ayahs: 12 },
    { number: 67, name: "Al-Mulk", nameAr: "الملك", ayahs: 30 },
    { number: 68, name: "Al-Qalam", nameAr: "القلم", ayahs: 52 },
    { number: 69, name: "Al-Haqqah", nameAr: "الحاقة", ayahs: 52 },
    { number: 70, name: "Al-Ma'arij", nameAr: "المعارج", ayahs: 44 },
    { number: 71, name: "Nuh", nameAr: "نوح", ayahs: 28 },
    { number: 72, name: "Al-Jinn", nameAr: "الجن", ayahs: 28 },
    { number: 73, name: "Al-Muzzammil", nameAr: "المزمل", ayahs: 20 },
    { number: 74, name: "Al-Muddaththir", nameAr: "المدثر", ayahs: 56 },
    { number: 75, name: "Al-Qiyamah", nameAr: "القيامة", ayahs: 40 },
    { number: 76, name: "Al-Insan", nameAr: "الانسان", ayahs: 31 },
    { number: 77, name: "Al-Mursalat", nameAr: "المرسلات", ayahs: 50 },
    { number: 78, name: "An-Naba", nameAr: "النبإ", ayahs: 40 },
    { number: 79, name: "An-Nazi'at", nameAr: "النازعات", ayahs: 46 },
    { number: 80, name: "'Abasa", nameAr: "عبس", ayahs: 42 },
    { number: 81, name: "At-Takwir", nameAr: "التكوير", ayahs: 29 },
    { number: 82, name: "Al-Infitar", nameAr: "الإنفطار", ayahs: 19 },
    { number: 83, name: "Al-Mutaffifin", nameAr: "المطففين", ayahs: 36 },
    { number: 84, name: "Al-Inshiqaq", nameAr: "الإنشقاق", ayahs: 25 },
    { number: 85, name: "Al-Buruj", nameAr: "البروج", ayahs: 22 },
    { number: 86, name: "At-Tariq", nameAr: "الطارق", ayahs: 17 },
    { number: 87, name: "Al-A'la", nameAr: "الأعلى", ayahs: 19 },
    { number: 88, name: "Al-Ghashiyah", nameAr: "الغاشية", ayahs: 26 },
    { number: 89, name: "Al-Fajr", nameAr: "الفجر", ayahs: 30 },
    { number: 90, name: "Al-Balad", nameAr: "البلد", ayahs: 20 },
    { number: 91, name: "Ash-Shams", nameAr: "الشمس", ayahs: 15 },
    { number: 92, name: "Al-Layl", nameAr: "الليل", ayahs: 21 },
    { number: 93, name: "Ad-Duhaa", nameAr: "الضحى", ayahs: 11 },
    { number: 94, name: "Ash-Sharh", nameAr: "الشرح", ayahs: 8 },
    { number: 95, name: "At-Tin", nameAr: "التين", ayahs: 8 },
    { number: 96, name: "Al-'Alaq", nameAr: "العلق", ayahs: 19 },
    { number: 97, name: "Al-Qadr", nameAr: "القدر", ayahs: 5 },
    { number: 98, name: "Al-Bayyinah", nameAr: "البينة", ayahs: 8 },
    { number: 99, name: "Az-Zalzalah", nameAr: "الزلزلة", ayahs: 8 },
    { number: 100, name: "Al-'Adiyat", nameAr: "العاديات", ayahs: 11 },
    { number: 101, name: "Al-Qari'ah", nameAr: "القارعة", ayahs: 11 },
    { number: 102, name: "At-Takathur", nameAr: "التكاثر", ayahs: 8 },
    { number: 103, name: "Al-'Asr", nameAr: "العصر", ayahs: 3 },
    { number: 104, name: "Al-Humazah", nameAr: "الهمزة", ayahs: 9 },
    { number: 105, name: "Al-Fil", nameAr: "الفيل", ayahs: 5 },
    { number: 106, name: "Quraysh", nameAr: "قريش", ayahs: 4 },
    { number: 107, name: "Al-Ma'un", nameAr: "الماعون", ayahs: 7 },
    { number: 108, name: "Al-Kawthar", nameAr: "الكوثر", ayahs: 3 },
    { number: 109, name: "Al-Kafirun", nameAr: "الكافرون", ayahs: 6 },
    { number: 110, name: "An-Nasr", nameAr: "النصر", ayahs: 3 },
    { number: 111, name: "Al-Masad", nameAr: "المسد", ayahs: 5 },
    { number: 112, name: "Al-Ikhlas", nameAr: "الإخلاص", ayahs: 4 },
    { number: 113, name: "Al-Falaq", nameAr: "الفلق", ayahs: 5 },
    { number: 114, name: "An-Nas", nameAr: "الناس", ayahs: 6 }
];

// Populate surah dropdown on page load
function populateSurahDropdown() {
    const select = document.getElementById('surahSelect');
    if (!select) return;

    select.innerHTML = '';
    SURAHS.forEach(surah => {
        const option = document.createElement('option');
        option.value = surah.number;
        option.textContent = `${surah.number}. ${surah.name} (${surah.nameAr})`;
        select.appendChild(option);
    });
}

// Update ayah dropdown based on selected surah
function updateAyahDropdown(surahNumber) {
    const surah = SURAHS.find(s => s.number === parseInt(surahNumber));
    if (!surah) return;

    const select = document.getElementById('ayahSelect');
    if (!select) return;

    select.innerHTML = '';
    for (let i = 1; i <= surah.ayahs; i++) {
        const option = document.createElement('option');
        option.value = i;
        option.textContent = `Ayah ${i}`;
        select.appendChild(option);
    }
}

// Initialize on page load
if (typeof document !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => {
        populateSurahDropdown();
        updateAyahDropdown(1);  // Default to Al-Fatihah

        // Listen for surah changes
        const surahSelect = document.getElementById('surahSelect');
        if (surahSelect) {
            surahSelect.addEventListener('change', (e) => {
                updateAyahDropdown(e.target.value);
            });
        }
    });
}
