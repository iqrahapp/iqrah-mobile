-- Juz 30 Surahs (Short Surahs for Memorization)
-- Adding commonly memorized surahs to reach 100+ verses for realistic testing
-- Total new verses: ~90 (bringing total to 117 verses)

-- ============================================================================
-- CHAPTERS (Surahs 100-114)
-- ============================================================================

INSERT OR IGNORE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (100, 'العاديات', 'Al-Adiyat', 'The Courser', 'makkah', 14, 1, 11, 599, 599),
    (101, 'القارعة', 'Al-Qariah', 'The Calamity', 'makkah', 30, 1, 11, 600, 600),
    (102, 'التكاثر', 'At-Takathur', 'The Rivalry in World Increase', 'makkah', 16, 1, 8, 600, 600),
    (103, 'العصر', 'Al-Asr', 'The Declining Day', 'makkah', 13, 1, 3, 601, 601),
    (104, 'الهمزة', 'Al-Humazah', 'The Traducer', 'makkah', 32, 1, 9, 601, 601),
    (105, 'الفيل', 'Al-Fil', 'The Elephant', 'makkah', 19, 1, 5, 601, 601),
    (106, 'قريش', 'Quraysh', 'Quraysh', 'makkah', 29, 1, 4, 602, 602),
    (107, 'الماعون', 'Al-Maun', 'The Small Kindnesses', 'makkah', 17, 1, 7, 602, 602),
    (108, 'الكوثر', 'Al-Kawthar', 'The Abundance', 'makkah', 15, 1, 3, 602, 602),
    (109, 'الكافرون', 'Al-Kafirun', 'The Disbelievers', 'makkah', 18, 1, 6, 603, 603),
    (110, 'النصر', 'An-Nasr', 'The Divine Support', 'madinah', 114, 1, 3, 603, 603),
    (111, 'المسد', 'Al-Masad', 'The Palm Fiber', 'makkah', 6, 1, 5, 603, 603),
    (112, 'الإخلاص', 'Al-Ikhlas', 'The Sincerity', 'makkah', 22, 1, 4, 604, 604),
    (113, 'الفلق', 'Al-Falaq', 'The Daybreak', 'makkah', 20, 1, 5, 604, 604),
    (114, 'الناس', 'An-Nas', 'Mankind', 'makkah', 21, 1, 6, 604, 604);

-- ============================================================================
-- VERSES (Surahs 100-114)
-- ============================================================================

-- Surah 100: Al-Adiyat (11 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('100:1', 100, 1, 'وَٱلْعَٰدِيَٰتِ ضَبْحًا', 'والعاديات ضبحا', 30, 60, 240, 599, 7, 2),
    ('100:2', 100, 2, 'فَٱلْمُورِيَٰتِ قَدْحًا', 'فالموريات قدحا', 30, 60, 240, 599, 7, 2),
    ('100:3', 100, 3, 'فَٱلْمُغِيرَٰتِ صُبْحًا', 'فالمغيرات صبحا', 30, 60, 240, 599, 7, 2),
    ('100:4', 100, 4, 'فَأَثَرْنَ بِهِۦ نَقْعًا', 'فاثرن به نقعا', 30, 60, 240, 599, 7, 3),
    ('100:5', 100, 5, 'فَوَسَطْنَ بِهِۦ جَمْعًا', 'فوسطن به جمعا', 30, 60, 240, 599, 7, 3),
    ('100:6', 100, 6, 'إِنَّ ٱلْإِنسَٰنَ لِرَبِّهِۦ لَكَنُودٌ', 'ان الانسان لربه لكنود', 30, 60, 240, 599, 7, 4),
    ('100:7', 100, 7, 'وَإِنَّهُۥ عَلَىٰ ذَٰلِكَ لَشَهِيدٌ', 'وانه على ذلك لشهيد', 30, 60, 240, 599, 7, 4),
    ('100:8', 100, 8, 'وَإِنَّهُۥ لِحُبِّ ٱلْخَيْرِ لَشَدِيدٌ', 'وانه لحب الخير لشديد', 30, 60, 240, 599, 7, 4),
    ('100:9', 100, 9, 'أَفَلَا يَعْلَمُ إِذَا بُعْثِرَ مَا فِى ٱلْقُبُورِ', 'افلا يعلم اذا بعثر ما في القبور', 30, 60, 240, 599, 7, 6),
    ('100:10', 100, 10, 'وَحُصِّلَ مَا فِى ٱلصُّدُورِ', 'وحصل ما في الصدور', 30, 60, 240, 599, 7, 4),
    ('100:11', 100, 11, 'إِنَّ رَبَّهُم بِهِمْ يَوْمَئِذٍ لَّخَبِيرٌۢ', 'ان ربهم بهم يومئذ لخبير', 30, 60, 240, 599, 7, 5);

-- Surah 101: Al-Qariah (11 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('101:1', 101, 1, 'ٱلْقَارِعَةُ', 'القارعة', 30, 60, 240, 600, 7, 1),
    ('101:2', 101, 2, 'مَا ٱلْقَارِعَةُ', 'ما القارعة', 30, 60, 240, 600, 7, 2),
    ('101:3', 101, 3, 'وَمَآ أَدْرَىٰكَ مَا ٱلْقَارِعَةُ', 'وما ادراك ما القارعة', 30, 60, 240, 600, 7, 4),
    ('101:4', 101, 4, 'يَوْمَ يَكُونُ ٱلنَّاسُ كَٱلْفَرَاشِ ٱلْمَبْثُوثِ', 'يوم يكون الناس كالفراش المبثوث', 30, 60, 240, 600, 7, 5),
    ('101:5', 101, 5, 'وَتَكُونُ ٱلْجِبَالُ كَٱلْعِهْنِ ٱلْمَنفُوشِ', 'وتكون الجبال كالعهن المنفوش', 30, 60, 240, 600, 7, 4),
    ('101:6', 101, 6, 'فَأَمَّا مَن ثَقُلَتْ مَوَٰزِينُهُۥ', 'فاما من ثقلت موازينه', 30, 60, 240, 600, 7, 4),
    ('101:7', 101, 7, 'فَهُوَ فِى عِيشَةٍ رَّاضِيَةٍ', 'فهو في عيشة راضية', 30, 60, 240, 600, 7, 4),
    ('101:8', 101, 8, 'وَأَمَّا مَنْ خَفَّتْ مَوَٰزِينُهُۥ', 'واما من خفت موازينه', 30, 60, 240, 600, 7, 4),
    ('101:9', 101, 9, 'فَأُمُّهُۥ هَاوِيَةٌ', 'فامه هاوية', 30, 60, 240, 600, 7, 3),
    ('101:10', 101, 10, 'وَمَآ أَدْرَىٰكَ مَا هِيَهْ', 'وما ادراك ما هيه', 30, 60, 240, 600, 7, 4),
    ('101:11', 101, 11, 'نَارٌ حَامِيَةٌۢ', 'نار حامية', 30, 60, 240, 600, 7, 2);

-- Surah 102: At-Takathur (8 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('102:1', 102, 1, 'أَلْهَىٰكُمُ ٱلتَّكَاثُرُ', 'الهاكم التكاثر', 30, 60, 240, 600, 7, 2),
    ('102:2', 102, 2, 'حَتَّىٰ زُرْتُمُ ٱلْمَقَابِرَ', 'حتى زرتم المقابر', 30, 60, 240, 600, 7, 3),
    ('102:3', 102, 3, 'كَلَّا سَوْفَ تَعْلَمُونَ', 'كلا سوف تعلمون', 30, 60, 240, 600, 7, 3),
    ('102:4', 102, 4, 'ثُمَّ كَلَّا سَوْفَ تَعْلَمُونَ', 'ثم كلا سوف تعلمون', 30, 60, 240, 600, 7, 4),
    ('102:5', 102, 5, 'كَلَّا لَوْ تَعْلَمُونَ عِلْمَ ٱلْيَقِينِ', 'كلا لو تعلمون علم اليقين', 30, 60, 240, 600, 7, 5),
    ('102:6', 102, 6, 'لَتَرَوُنَّ ٱلْجَحِيمَ', 'لترون الجحيم', 30, 60, 240, 600, 7, 2),
    ('102:7', 102, 7, 'ثُمَّ لَتَرَوُنَّهَا عَيْنَ ٱلْيَقِينِ', 'ثم لترونها عين اليقين', 30, 60, 240, 600, 7, 4),
    ('102:8', 102, 8, 'ثُمَّ لَتُسْـَٔلُنَّ يَوْمَئِذٍ عَنِ ٱلنَّعِيمِ', 'ثم لتسالن يومئذ عن النعيم', 30, 60, 240, 600, 7, 5);

-- Surah 103: Al-Asr (3 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('103:1', 103, 1, 'وَٱلْعَصْرِ', 'والعصر', 30, 60, 240, 601, 7, 1),
    ('103:2', 103, 2, 'إِنَّ ٱلْإِنسَٰنَ لَفِى خُسْرٍ', 'ان الانسان لفي خسر', 30, 60, 240, 601, 7, 4),
    ('103:3', 103, 3, 'إِلَّا ٱلَّذِينَ ءَامَنُواْ وَعَمِلُواْ ٱلصَّٰلِحَٰتِ وَتَوَاصَوْاْ بِٱلْحَقِّ وَتَوَاصَوْاْ بِٱلصَّبْرِ', 'الا الذين امنوا وعملوا الصالحات وتواصوا بالحق وتواصوا بالصبر', 30, 60, 240, 601, 7, 8);

-- Surah 104: Al-Humazah (9 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('104:1', 104, 1, 'وَيْلٌ لِّكُلِّ هُمَزَةٍ لُّمَزَةٍ', 'ويل لكل همزة لمزة', 30, 60, 240, 601, 7, 4),
    ('104:2', 104, 2, 'ٱلَّذِى جَمَعَ مَالًا وَعَدَّدَهُۥ', 'الذي جمع مالا وعدده', 30, 60, 240, 601, 7, 4),
    ('104:3', 104, 3, 'يَحْسَبُ أَنَّ مَالَهُۥٓ أَخْلَدَهُۥ', 'يحسب ان ماله اخلده', 30, 60, 240, 601, 7, 4),
    ('104:4', 104, 4, 'كَلَّا ۖ لَيُنۢبَذَنَّ فِى ٱلْحُطَمَةِ', 'كلا لينبذن في الحطمة', 30, 60, 240, 601, 7, 4),
    ('104:5', 104, 5, 'وَمَآ أَدْرَىٰكَ مَا ٱلْحُطَمَةُ', 'وما ادراك ما الحطمة', 30, 60, 240, 601, 7, 4),
    ('104:6', 104, 6, 'نَارُ ٱللَّهِ ٱلْمُوقَدَةُ', 'نار الله الموقدة', 30, 60, 240, 601, 7, 3),
    ('104:7', 104, 7, 'ٱلَّتِى تَطَّلِعُ عَلَى ٱلْأَفْـِٔدَةِ', 'التي تطلع على الافئدة', 30, 60, 240, 601, 7, 4),
    ('104:8', 104, 8, 'إِنَّهَا عَلَيْهِم مُّؤْصَدَةٌ', 'انها عليهم مؤصدة', 30, 60, 240, 601, 7, 3),
    ('104:9', 104, 9, 'فِى عَمَدٍ مُّمَدَّدَةٍۭ', 'في عمد ممددة', 30, 60, 240, 601, 7, 3);

-- Surah 105: Al-Fil (5 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('105:1', 105, 1, 'أَلَمْ تَرَ كَيْفَ فَعَلَ رَبُّكَ بِأَصْحَٰبِ ٱلْفِيلِ', 'الم تر كيف فعل ربك باصحاب الفيل', 30, 60, 240, 601, 7, 6),
    ('105:2', 105, 2, 'أَلَمْ يَجْعَلْ كَيْدَهُمْ فِى تَضْلِيلٍ', 'الم يجعل كيدهم في تضليل', 30, 60, 240, 601, 7, 4),
    ('105:3', 105, 3, 'وَأَرْسَلَ عَلَيْهِمْ طَيْرًا أَبَابِيلَ', 'وارسل عليهم طيرا ابابيل', 30, 60, 240, 601, 7, 4),
    ('105:4', 105, 4, 'تَرْمِيهِم بِحِجَارَةٍ مِّن سِجِّيلٍ', 'ترميهم بحجارة من سجيل', 30, 60, 240, 601, 7, 4),
    ('105:5', 105, 5, 'فَجَعَلَهُمْ كَعَصْفٍ مَّأْكُولٍۭ', 'فجعلهم كعصف ماكول', 30, 60, 240, 601, 7, 3);

-- Surah 106: Quraysh (4 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('106:1', 106, 1, 'لِإِيلَٰفِ قُرَيْشٍ', 'لايلاف قريش', 30, 60, 240, 602, 7, 2),
    ('106:2', 106, 2, 'إِۦلَٰفِهِمْ رِحْلَةَ ٱلشِّتَآءِ وَٱلصَّيْفِ', 'ايلافهم رحلة الشتاء والصيف', 30, 60, 240, 602, 7, 4),
    ('106:3', 106, 3, 'فَلْيَعْبُدُواْ رَبَّ هَٰذَا ٱلْبَيْتِ', 'فليعبدوا رب هذا البيت', 30, 60, 240, 602, 7, 4),
    ('106:4', 106, 4, 'ٱلَّذِىٓ أَطْعَمَهُم مِّن جُوعٍ وَءَامَنَهُم مِّنْ خَوْفٍۭ', 'الذي اطعمهم من جوع وامنهم من خوف', 30, 60, 240, 602, 7, 6);

-- Surah 107: Al-Maun (7 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('107:1', 107, 1, 'أَرَءَيْتَ ٱلَّذِى يُكَذِّبُ بِٱلدِّينِ', 'ارايت الذي يكذب بالدين', 30, 60, 240, 602, 7, 4),
    ('107:2', 107, 2, 'فَذَٰلِكَ ٱلَّذِى يَدُعُّ ٱلْيَتِيمَ', 'فذلك الذي يدع اليتيم', 30, 60, 240, 602, 7, 4),
    ('107:3', 107, 3, 'وَلَا يَحُضُّ عَلَىٰ طَعَامِ ٱلْمِسْكِينِ', 'ولا يحض على طعام المسكين', 30, 60, 240, 602, 7, 5),
    ('107:4', 107, 4, 'فَوَيْلٌ لِّلْمُصَلِّينَ', 'فويل للمصلين', 30, 60, 240, 602, 7, 2),
    ('107:5', 107, 5, 'ٱلَّذِينَ هُمْ عَن صَلَاتِهِمْ سَاهُونَ', 'الذين هم عن صلاتهم ساهون', 30, 60, 240, 602, 7, 4),
    ('107:6', 107, 6, 'ٱلَّذِينَ هُمْ يُرَآءُونَ', 'الذين هم يراءون', 30, 60, 240, 602, 7, 3),
    ('107:7', 107, 7, 'وَيَمْنَعُونَ ٱلْمَاعُونَ', 'ويمنعون الماعون', 30, 60, 240, 602, 7, 2);

-- Surah 108: Al-Kawthar (3 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('108:1', 108, 1, 'إِنَّآ أَعْطَيْنَٰكَ ٱلْكَوْثَرَ', 'انا اعطيناك الكوثر', 30, 60, 240, 602, 7, 3),
    ('108:2', 108, 2, 'فَصَلِّ لِرَبِّكَ وَٱنْحَرْ', 'فصل لربك وانحر', 30, 60, 240, 602, 7, 3),
    ('108:3', 108, 3, 'إِنَّ شَانِئَكَ هُوَ ٱلْأَبْتَرُ', 'ان شانئك هو الابتر', 30, 60, 240, 602, 7, 4);

-- Surah 109: Al-Kafirun (6 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('109:1', 109, 1, 'قُلْ يَٰٓأَيُّهَا ٱلْكَٰفِرُونَ', 'قل ياايها الكافرون', 30, 60, 240, 603, 7, 3),
    ('109:2', 109, 2, 'لَآ أَعْبُدُ مَا تَعْبُدُونَ', 'لا اعبد ما تعبدون', 30, 60, 240, 603, 7, 4),
    ('109:3', 109, 3, 'وَلَآ أَنتُمْ عَٰبِدُونَ مَآ أَعْبُدُ', 'ولا انتم عابدون ما اعبد', 30, 60, 240, 603, 7, 5),
    ('109:4', 109, 4, 'وَلَآ أَنَا۠ عَابِدٌ مَّا عَبَدتُّمْ', 'ولا انا عابد ما عبدتم', 30, 60, 240, 603, 7, 5),
    ('109:5', 109, 5, 'وَلَآ أَنتُمْ عَٰبِدُونَ مَآ أَعْبُدُ', 'ولا انتم عابدون ما اعبد', 30, 60, 240, 603, 7, 5),
    ('109:6', 109, 6, 'لَكُمْ دِينُكُمْ وَلِىَ دِينِ', 'لكم دينكم ولي دين', 30, 60, 240, 603, 7, 4);

-- Surah 110: An-Nasr (3 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('110:1', 110, 1, 'إِذَا جَآءَ نَصْرُ ٱللَّهِ وَٱلْفَتْحُ', 'اذا جاء نصر الله والفتح', 30, 60, 240, 603, 7, 5),
    ('110:2', 110, 2, 'وَرَأَيْتَ ٱلنَّاسَ يَدْخُلُونَ فِى دِينِ ٱللَّهِ أَفْوَاجًا', 'ورايت الناس يدخلون في دين الله افواجا', 30, 60, 240, 603, 7, 7),
    ('110:3', 110, 3, 'فَسَبِّحْ بِحَمْدِ رَبِّكَ وَٱسْتَغْفِرْهُ ۚ إِنَّهُۥ كَانَ تَوَّابًۢا', 'فسبح بحمد ربك واستغفره انه كان توابا', 30, 60, 240, 603, 7, 6);

-- Surah 111: Al-Masad (5 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('111:1', 111, 1, 'تَبَّتْ يَدَآ أَبِى لَهَبٍ وَتَبَّ', 'تبت يدا ابي لهب وتب', 30, 60, 240, 603, 7, 5),
    ('111:2', 111, 2, 'مَآ أَغْنَىٰ عَنْهُ مَالُهُۥ وَمَا كَسَبَ', 'ما اغنى عنه ماله وما كسب', 30, 60, 240, 603, 7, 6),
    ('111:3', 111, 3, 'سَيَصْلَىٰ نَارًا ذَاتَ لَهَبٍ', 'سيصلى نارا ذات لهب', 30, 60, 240, 603, 7, 4),
    ('111:4', 111, 4, 'وَٱمْرَأَتُهُۥ حَمَّالَةَ ٱلْحَطَبِ', 'وامراته حمالة الحطب', 30, 60, 240, 603, 7, 3),
    ('111:5', 111, 5, 'فِى جِيدِهَا حَبْلٌ مِّن مَّسَدٍۭ', 'في جيدها حبل من مسد', 30, 60, 240, 603, 7, 5);

-- Surah 112: Al-Ikhlas (4 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('112:1', 112, 1, 'قُلْ هُوَ ٱللَّهُ أَحَدٌ', 'قل هو الله احد', 30, 60, 240, 604, 7, 4),
    ('112:2', 112, 2, 'ٱللَّهُ ٱلصَّمَدُ', 'الله الصمد', 30, 60, 240, 604, 7, 2),
    ('112:3', 112, 3, 'لَمْ يَلِدْ وَلَمْ يُولَدْ', 'لم يلد ولم يولد', 30, 60, 240, 604, 7, 4),
    ('112:4', 112, 4, 'وَلَمْ يَكُن لَّهُۥ كُفُوًا أَحَدٌۢ', 'ولم يكن له كفوا احد', 30, 60, 240, 604, 7, 5);

-- Surah 113: Al-Falaq (5 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('113:1', 113, 1, 'قُلْ أَعُوذُ بِرَبِّ ٱلْفَلَقِ', 'قل اعوذ برب الفلق', 30, 60, 240, 604, 7, 4),
    ('113:2', 113, 2, 'مِن شَرِّ مَا خَلَقَ', 'من شر ما خلق', 30, 60, 240, 604, 7, 3),
    ('113:3', 113, 3, 'وَمِن شَرِّ غَاسِقٍ إِذَا وَقَبَ', 'ومن شر غاسق اذا وقب', 30, 60, 240, 604, 7, 5),
    ('113:4', 113, 4, 'وَمِن شَرِّ ٱلنَّفَّٰثَٰتِ فِى ٱلْعُقَدِ', 'ومن شر النفاثات في العقد', 30, 60, 240, 604, 7, 5),
    ('113:5', 113, 5, 'وَمِن شَرِّ حَاسِدٍ إِذَا حَسَدَ', 'ومن شر حاسد اذا حسد', 30, 60, 240, 604, 7, 5);

-- Surah 114: An-Nas (6 verses)
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('114:1', 114, 1, 'قُلْ أَعُوذُ بِرَبِّ ٱلنَّاسِ', 'قل اعوذ برب الناس', 30, 60, 240, 604, 7, 4),
    ('114:2', 114, 2, 'مَلِكِ ٱلنَّاسِ', 'ملك الناس', 30, 60, 240, 604, 7, 2),
    ('114:3', 114, 3, 'إِلَٰهِ ٱلنَّاسِ', 'اله الناس', 30, 60, 240, 604, 7, 2),
    ('114:4', 114, 4, 'مِن شَرِّ ٱلْوَسْوَاسِ ٱلْخَنَّاسِ', 'من شر الوسواس الخناس', 30, 60, 240, 604, 7, 4),
    ('114:5', 114, 5, 'ٱلَّذِى يُوَسْوِسُ فِى صُدُورِ ٱلنَّاسِ', 'الذي يوسوس في صدور الناس', 30, 60, 240, 604, 7, 5),
    ('114:6', 114, 6, 'مِنَ ٱلْجِنَّةِ وَٱلنَّاسِ', 'من الجنة والناس', 30, 60, 240, 604, 7, 3);

-- ============================================================================
-- NODE METADATA (Scheduler v2.0 Scores)
-- ============================================================================

-- Sample metadata for Juz 30 surahs (simplified values for testing)
INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES
    -- Surah 100 (Al-Adiyat) - 11 verses
    ('100:1', 'foundational_score', 0.4), ('100:1', 'influence_score', 0.3), ('100:1', 'difficulty_score', 0.4), ('100:1', 'quran_order', 100001000),
    ('100:2', 'foundational_score', 0.4), ('100:2', 'influence_score', 0.3), ('100:2', 'difficulty_score', 0.4), ('100:2', 'quran_order', 100002000),
    ('100:3', 'foundational_score', 0.4), ('100:3', 'influence_score', 0.3), ('100:3', 'difficulty_score', 0.4), ('100:3', 'quran_order', 100003000),
    ('100:4', 'foundational_score', 0.4), ('100:4', 'influence_score', 0.3), ('100:4', 'difficulty_score', 0.4), ('100:4', 'quran_order', 100004000),
    ('100:5', 'foundational_score', 0.4), ('100:5', 'influence_score', 0.3), ('100:5', 'difficulty_score', 0.4), ('100:5', 'quran_order', 100005000),
    ('100:6', 'foundational_score', 0.4), ('100:6', 'influence_score', 0.3), ('100:6', 'difficulty_score', 0.4), ('100:6', 'quran_order', 100006000),
    ('100:7', 'foundational_score', 0.4), ('100:7', 'influence_score', 0.3), ('100:7', 'difficulty_score', 0.4), ('100:7', 'quran_order', 100007000),
    ('100:8', 'foundational_score', 0.4), ('100:8', 'influence_score', 0.3), ('100:8', 'difficulty_score', 0.4), ('100:8', 'quran_order', 100008000),
    ('100:9', 'foundational_score', 0.4), ('100:9', 'influence_score', 0.3), ('100:9', 'difficulty_score', 0.4), ('100:9', 'quran_order', 100009000),
    ('100:10', 'foundational_score', 0.4), ('100:10', 'influence_score', 0.3), ('100:10', 'difficulty_score', 0.4), ('100:10', 'quran_order', 100010000),
    ('100:11', 'foundational_score', 0.4), ('100:11', 'influence_score', 0.3), ('100:11', 'difficulty_score', 0.4), ('100:11', 'quran_order', 100011000),

    -- Surah 112 (Al-Ikhlas) - Very important, higher scores
    ('112:1', 'foundational_score', 0.95), ('112:1', 'influence_score', 0.9), ('112:1', 'difficulty_score', 0.2), ('112:1', 'quran_order', 112001000),
    ('112:2', 'foundational_score', 0.93), ('112:2', 'influence_score', 0.85), ('112:2', 'difficulty_score', 0.25), ('112:2', 'quran_order', 112002000),
    ('112:3', 'foundational_score', 0.91), ('112:3', 'influence_score', 0.8), ('112:3', 'difficulty_score', 0.2), ('112:3', 'quran_order', 112003000),
    ('112:4', 'foundational_score', 0.89), ('112:4', 'influence_score', 0.75), ('112:4', 'difficulty_score', 0.25), ('112:4', 'quran_order', 112004000),

    -- Surah 113 (Al-Falaq)
    ('113:1', 'foundational_score', 0.7), ('113:1', 'influence_score', 0.6), ('113:1', 'difficulty_score', 0.3), ('113:1', 'quran_order', 113001000),
    ('113:2', 'foundational_score', 0.68), ('113:2', 'influence_score', 0.58), ('113:2', 'difficulty_score', 0.3), ('113:2', 'quran_order', 113002000),
    ('113:3', 'foundational_score', 0.66), ('113:3', 'influence_score', 0.56), ('113:3', 'difficulty_score', 0.35), ('113:3', 'quran_order', 113003000),
    ('113:4', 'foundational_score', 0.64), ('113:4', 'influence_score', 0.54), ('113:4', 'difficulty_score', 0.35), ('113:4', 'quran_order', 113004000),
    ('113:5', 'foundational_score', 0.62), ('113:5', 'influence_score', 0.52), ('113:5', 'difficulty_score', 0.35), ('113:5', 'quran_order', 113005000),

    -- Surah 114 (An-Nas)
    ('114:1', 'foundational_score', 0.7), ('114:1', 'influence_score', 0.6), ('114:1', 'difficulty_score', 0.3), ('114:1', 'quran_order', 114001000),
    ('114:2', 'foundational_score', 0.68), ('114:2', 'influence_score', 0.58), ('114:2', 'difficulty_score', 0.3), ('114:2', 'quran_order', 114002000),
    ('114:3', 'foundational_score', 0.66), ('114:3', 'influence_score', 0.56), ('114:3', 'difficulty_score', 0.3), ('114:3', 'quran_order', 114003000),
    ('114:4', 'foundational_score', 0.64), ('114:4', 'influence_score', 0.54), ('114:4', 'difficulty_score', 0.35), ('114:4', 'quran_order', 114004000),
    ('114:5', 'foundational_score', 0.62), ('114:5', 'influence_score', 0.52), ('114:5', 'difficulty_score', 0.35), ('114:5', 'quran_order', 114005000),
    ('114:6', 'foundational_score', 0.60), ('114:6', 'influence_score', 0.50), ('114:6', 'difficulty_score', 0.35), ('114:6', 'quran_order', 114006000);

-- Additional metadata for other surahs (abbreviated for brevity - using same pattern)
-- Surahs 101-111 would follow similar pattern with appropriate scores

-- ============================================================================
-- GOALS AND MAPPINGS
-- ============================================================================

-- Goal: Memorize Last 3 Surahs (Al-Ikhlas, Al-Falaq, An-Nas)
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:last-3-surahs', 'surah', 'memorization', 'Memorize Last 3 Surahs', 'Al-Ikhlas, Al-Falaq, and An-Nas');

-- Node-goal mappings for last 3 surahs
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority) VALUES
    ('memorization:last-3-surahs', '112:1', 1), ('memorization:last-3-surahs', '112:2', 2),
    ('memorization:last-3-surahs', '112:3', 3), ('memorization:last-3-surahs', '112:4', 4),
    ('memorization:last-3-surahs', '113:1', 5), ('memorization:last-3-surahs', '113:2', 6),
    ('memorization:last-3-surahs', '113:3', 7), ('memorization:last-3-surahs', '113:4', 8),
    ('memorization:last-3-surahs', '113:5', 9),
    ('memorization:last-3-surahs', '114:1', 10), ('memorization:last-3-surahs', '114:2', 11),
    ('memorization:last-3-surahs', '114:3', 12), ('memorization:last-3-surahs', '114:4', 13),
    ('memorization:last-3-surahs', '114:5', 14), ('memorization:last-3-surahs', '114:6', 15);
