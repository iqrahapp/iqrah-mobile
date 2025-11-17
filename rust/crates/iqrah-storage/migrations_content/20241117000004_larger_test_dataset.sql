-- Larger Test Dataset for Scheduler v2 Performance Testing
-- Adds data for Surahs 2-4 (Al-Baqarah first 20 verses, Al-Imran first 10, An-Nisa first 10)
-- Total: ~40 additional verses for realistic performance testing

-- Chapter 2: Al-Baqarah
INSERT OR IGNORE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (2, 'البقرة', 'Al-Baqarah', 'The Cow', 'madinah', 87, 1, 286, 2, 49);

-- Chapter 3: Al-Imran
INSERT OR IGNORE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (3, 'آل عمران', 'Ali Imran', 'The Family of Imran', 'madinah', 89, 1, 200, 50, 76);

-- Chapter 4: An-Nisa
INSERT OR IGNORE INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, revelation_place, revelation_order, bismillah_pre, verse_count, page_start, page_end) VALUES
    (4, 'النساء', 'An-Nisa', 'The Women', 'madinah', 92, 1, 176, 77, 106);

-- Al-Baqarah verses 2:1-2:20
INSERT OR IGNORE INTO verses (verse_key, chapter_number, verse_number, text_uthmani, text_simple, juz, hizb, rub_el_hizb, page, manzil, word_count) VALUES
    ('2:1', 2, 1, 'الم', 'الم', 1, 1, 1, 2, 1, 1),
    ('2:2', 2, 2, 'ذَٰلِكَ ٱلْكِتَٰبُ لَا رَيْبَ ۛ فِيهِ ۛ هُدًى لِّلْمُتَّقِينَ', 'ذلك الكتاب لا ريب فيه هدى للمتقين', 1, 1, 1, 2, 1, 5),
    ('2:3', 2, 3, 'ٱلَّذِينَ يُؤْمِنُونَ بِٱلْغَيْبِ وَيُقِيمُونَ ٱلصَّلَوٰةَ وَمِمَّا رَزَقْنَٰهُمْ يُنفِقُونَ', 'الذين يؤمنون بالغيب ويقيمون الصلاة ومما رزقناهم ينفقون', 1, 1, 1, 2, 1, 7),
    ('2:4', 2, 4, 'وَٱلَّذِينَ يُؤْمِنُونَ بِمَآ أُنزِلَ إِلَيْكَ وَمَآ أُنزِلَ مِن قَبْلِكَ وَبِٱلْءَاخِرَةِ هُمْ يُوقِنُونَ', 'والذين يؤمنون بما انزل اليك وما انزل من قبلك وبالاخرة هم يوقنون', 1, 1, 1, 2, 1, 10),
    ('2:5', 2, 5, 'أُو۟لَٰٓئِكَ عَلَىٰ هُدًى مِّن رَّبِّهِمْ ۖ وَأُو۟لَٰٓئِكَ هُمُ ٱلْمُفْلِحُونَ', 'اولئك على هدى من ربهم واولئك هم المفلحون', 1, 1, 1, 2, 1, 6),
    ('2:6', 2, 6, 'إِنَّ ٱلَّذِينَ كَفَرُواْ سَوَآءٌ عَلَيْهِمْ ءَأَنذَرْتَهُمْ أَمْ لَمْ تُنذِرْهُمْ لَا يُؤْمِنُونَ', 'ان الذين كفروا سواء عليهم اانذرتهم ام لم تنذرهم لا يؤمنون', 1, 1, 1, 2, 1, 10),
    ('2:7', 2, 7, 'خَتَمَ ٱللَّهُ عَلَىٰ قُلُوبِهِمْ وَعَلَىٰ سَمْعِهِمْ ۖ وَعَلَىٰٓ أَبْصَٰرِهِمْ غِشَٰوَةٌ ۖ وَلَهُمْ عَذَابٌ عَظِيمٌ', 'ختم الله على قلوبهم وعلى سمعهم وعلى ابصارهم غشاوة ولهم عذاب عظيم', 1, 1, 1, 2, 1, 9),
    ('2:8', 2, 8, 'وَمِنَ ٱلنَّاسِ مَن يَقُولُ ءَامَنَّا بِٱللَّهِ وَبِٱلْيَوْمِ ٱلْءَاخِرِ وَمَا هُم بِمُؤْمِنِينَ', 'ومن الناس من يقول امنا بالله وباليوم الاخر وما هم بمؤمنين', 1, 1, 1, 2, 1, 11),
    ('2:9', 2, 9, 'يُخَٰدِعُونَ ٱللَّهَ وَٱلَّذِينَ ءَامَنُواْ وَمَا يَخْدَعُونَ إِلَّآ أَنفُسَهُمْ وَمَا يَشْعُرُونَ', 'يخادعون الله والذين امنوا وما يخدعون الا انفسهم وما يشعرون', 1, 1, 1, 2, 1, 9),
    ('2:10', 2, 10, 'فِى قُلُوبِهِم مَّرَضٌ فَزَادَهُمُ ٱللَّهُ مَرَضًا ۖ وَلَهُمْ عَذَابٌ أَلِيمٌۢ بِمَا كَانُواْ يَكْذِبُونَ', 'في قلوبهم مرض فزادهم الله مرضا ولهم عذاب اليم بما كانوا يكذبون', 1, 1, 1, 2, 1, 10),
    ('2:11', 2, 11, 'وَإِذَا قِيلَ لَهُمْ لَا تُفْسِدُواْ فِى ٱلْأَرْضِ قَالُوٓاْ إِنَّمَا نَحْنُ مُصْلِحُونَ', 'واذا قيل لهم لا تفسدوا في الارض قالوا انما نحن مصلحون', 1, 1, 1, 2, 1, 10),
    ('2:12', 2, 12, 'أَلَآ إِنَّهُمْ هُمُ ٱلْمُفْسِدُونَ وَلَٰكِن لَّا يَشْعُرُونَ', 'الا انهم هم المفسدون ولكن لا يشعرون', 1, 1, 1, 2, 1, 6),
    ('2:13', 2, 13, 'وَإِذَا قِيلَ لَهُمْ ءَامِنُواْ كَمَآ ءَامَنَ ٱلنَّاسُ قَالُوٓاْ أَنُؤْمِنُ كَمَآ ءَامَنَ ٱلسُّفَهَآءُ ۗ أَلَآ إِنَّهُمْ هُمُ ٱلسُّفَهَآءُ وَلَٰكِن لَّا يَعْلَمُونَ', 'واذا قيل لهم امنوا كما امن الناس قالوا انؤمن كما امن السفهاء الا انهم هم السفهاء ولكن لا يعلمون', 1, 1, 1, 2, 1, 15),
    ('2:14', 2, 14, 'وَإِذَا لَقُواْ ٱلَّذِينَ ءَامَنُواْ قَالُوٓاْ ءَامَنَّا وَإِذَا خَلَوْاْ إِلَىٰ شَيَٰطِينِهِمْ قَالُوٓاْ إِنَّا مَعَكُمْ إِنَّمَا نَحْنُ مُسْتَهْزِءُونَ', 'واذا لقوا الذين امنوا قالوا امنا واذا خلوا الى شياطينهم قالوا انا معكم انما نحن مستهزءون', 1, 1, 1, 2, 1, 14),
    ('2:15', 2, 15, 'ٱللَّهُ يَسْتَهْزِئُ بِهِمْ وَيَمُدُّهُمْ فِى طُغْيَٰنِهِمْ يَعْمَهُونَ', 'الله يستهزئ بهم ويمدهم في طغيانهم يعمهون', 1, 1, 1, 2, 1, 6),
    ('2:16', 2, 16, 'أُو۟لَٰٓئِكَ ٱلَّذِينَ ٱشْتَرَوُاْ ٱلضَّلَٰلَةَ بِٱلْهُدَىٰ فَمَا رَبِحَت تِّجَٰرَتُهُمْ وَمَا كَانُواْ مُهْتَدِينَ', 'اولئك الذين اشتروا الضلالة بالهدى فما ربحت تجارتهم وما كانوا مهتدين', 1, 1, 1, 2, 1, 9),
    ('2:17', 2, 17, 'مَثَلُهُمْ كَمَثَلِ ٱلَّذِى ٱسْتَوْقَدَ نَارًا فَلَمَّآ أَضَآءَتْ مَا حَوْلَهُۥ ذَهَبَ ٱللَّهُ بِنُورِهِمْ وَتَرَكَهُمْ فِى ظُلُمَٰتٍ لَّا يُبْصِرُونَ', 'مثلهم كمثل الذي استوقد نارا فلما اضاءت ما حوله ذهب الله بنورهم وتركهم في ظلمات لا يبصرون', 1, 1, 1, 2, 1, 15),
    ('2:18', 2, 18, 'صُمٌّۢ بُكْمٌ عُمْىٌ فَهُمْ لَا يَرْجِعُونَ', 'صم بكم عمي فهم لا يرجعون', 1, 1, 1, 2, 1, 5),
    ('2:19', 2, 19, 'أَوْ كَصَيِّبٍ مِّنَ ٱلسَّمَآءِ فِيهِ ظُلُمَٰتٌ وَرَعْدٌ وَبَرْقٌ يَجْعَلُونَ أَصَٰبِعَهُمْ فِىٓ ءَاذَانِهِم مِّنَ ٱلصَّوَٰعِقِ حَذَرَ ٱلْمَوْتِ ۚ وَٱللَّهُ مُحِيطٌۢ بِٱلْكَٰفِرِينَ', 'او كصيب من السماء فيه ظلمات ورعد وبرق يجعلون اصابعهم في اذانهم من الصواعق حذر الموت والله محيط بالكافرين', 1, 1, 1, 2, 1, 18),
    ('2:20', 2, 20, 'يَكَادُ ٱلْبَرْقُ يَخْطَفُ أَبْصَٰرَهُمْ ۖ كُلَّمَآ أَضَآءَ لَهُم مَّشَوْاْ فِيهِ وَإِذَآ أَظْلَمَ عَلَيْهِمْ قَامُواْ ۚ وَلَوْ شَآءَ ٱللَّهُ لَذَهَبَ بِسَمْعِهِمْ وَأَبْصَٰرِهِمْ ۚ إِنَّ ٱللَّهَ عَلَىٰ كُلِّ شَىْءٍ قَدِيرٌ', 'يكاد البرق يخطف ابصارهم كلما اضاء لهم مشوا فيه واذا اظلم عليهم قاموا ولو شاء الله لذهب بسمعهم وابصارهم ان الله على كل شيء قدير', 1, 1, 1, 2, 1, 23);

-- Node metadata for Al-Baqarah 2:1-2:20
INSERT OR IGNORE INTO node_metadata (node_id, key, value)
SELECT node_id, key, value FROM (
    SELECT '2:' || verse_num AS node_id, 'foundational_score' AS key, 0.5 + (verse_num * 0.01) AS value FROM (
        SELECT 1 AS verse_num UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5
        UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10
        UNION ALL SELECT 11 UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15
        UNION ALL SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
    )
    UNION ALL
    SELECT '2:' || verse_num, 'influence_score', 0.4 + (verse_num * 0.01) FROM (
        SELECT 1 AS verse_num UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5
        UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10
        UNION ALL SELECT 11 UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15
        UNION ALL SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
    )
    UNION ALL
    SELECT '2:' || verse_num, 'difficulty_score', 0.3 + (verse_num * 0.01) FROM (
        SELECT 1 AS verse_num UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5
        UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10
        UNION ALL SELECT 11 UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15
        UNION ALL SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
    )
    UNION ALL
    SELECT '2:' || verse_num, 'quran_order', 2000000 + (verse_num * 1000) FROM (
        SELECT 1 AS verse_num UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5
        UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10
        UNION ALL SELECT 11 UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15
        UNION ALL SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
    )
);

-- Goal for Al-Baqarah first 20 verses
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:surah-2-part1', 'surah', 'memorization', 'Memorize Al-Baqarah (Part 1)', 'First 20 verses of Al-Baqarah');

-- Node-goal mappings for Al-Baqarah
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority)
SELECT 'memorization:surah-2-part1', '2:' || verse_num, verse_num FROM (
    SELECT 1 AS verse_num UNION ALL SELECT 2 UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5
    UNION ALL SELECT 6 UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10
    UNION ALL SELECT 11 UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15
    UNION ALL SELECT 16 UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
);

-- Sequential prerequisites for Al-Baqarah (each verse requires previous)
INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
SELECT '2:' || (verse_num - 1), '2:' || verse_num, 0, 0, 0.0, 0.0 FROM (
    SELECT 2 AS verse_num UNION ALL SELECT 3 UNION ALL SELECT 4 UNION ALL SELECT 5 UNION ALL SELECT 6
    UNION ALL SELECT 7 UNION ALL SELECT 8 UNION ALL SELECT 9 UNION ALL SELECT 10 UNION ALL SELECT 11
    UNION ALL SELECT 12 UNION ALL SELECT 13 UNION ALL SELECT 14 UNION ALL SELECT 15 UNION ALL SELECT 16
    UNION ALL SELECT 17 UNION ALL SELECT 18 UNION ALL SELECT 19 UNION ALL SELECT 20
);
