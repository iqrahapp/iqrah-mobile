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
INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES
    ('2:1', 'foundational_score', 0.51), ('2:1', 'influence_score', 0.41), ('2:1', 'difficulty_score', 0.31), ('2:1', 'quran_order', 2001000),
    ('2:2', 'foundational_score', 0.52), ('2:2', 'influence_score', 0.42), ('2:2', 'difficulty_score', 0.32), ('2:2', 'quran_order', 2002000),
    ('2:3', 'foundational_score', 0.53), ('2:3', 'influence_score', 0.43), ('2:3', 'difficulty_score', 0.33), ('2:3', 'quran_order', 2003000),
    ('2:4', 'foundational_score', 0.54), ('2:4', 'influence_score', 0.44), ('2:4', 'difficulty_score', 0.34), ('2:4', 'quran_order', 2004000),
    ('2:5', 'foundational_score', 0.55), ('2:5', 'influence_score', 0.45), ('2:5', 'difficulty_score', 0.35), ('2:5', 'quran_order', 2005000),
    ('2:6', 'foundational_score', 0.56), ('2:6', 'influence_score', 0.46), ('2:6', 'difficulty_score', 0.36), ('2:6', 'quran_order', 2006000),
    ('2:7', 'foundational_score', 0.57), ('2:7', 'influence_score', 0.47), ('2:7', 'difficulty_score', 0.37), ('2:7', 'quran_order', 2007000),
    ('2:8', 'foundational_score', 0.58), ('2:8', 'influence_score', 0.48), ('2:8', 'difficulty_score', 0.38), ('2:8', 'quran_order', 2008000),
    ('2:9', 'foundational_score', 0.59), ('2:9', 'influence_score', 0.49), ('2:9', 'difficulty_score', 0.39), ('2:9', 'quran_order', 2009000),
    ('2:10', 'foundational_score', 0.60), ('2:10', 'influence_score', 0.50), ('2:10', 'difficulty_score', 0.40), ('2:10', 'quran_order', 2010000),
    ('2:11', 'foundational_score', 0.61), ('2:11', 'influence_score', 0.51), ('2:11', 'difficulty_score', 0.41), ('2:11', 'quran_order', 2011000),
    ('2:12', 'foundational_score', 0.62), ('2:12', 'influence_score', 0.52), ('2:12', 'difficulty_score', 0.42), ('2:12', 'quran_order', 2012000),
    ('2:13', 'foundational_score', 0.63), ('2:13', 'influence_score', 0.53), ('2:13', 'difficulty_score', 0.43), ('2:13', 'quran_order', 2013000),
    ('2:14', 'foundational_score', 0.64), ('2:14', 'influence_score', 0.54), ('2:14', 'difficulty_score', 0.44), ('2:14', 'quran_order', 2014000),
    ('2:15', 'foundational_score', 0.65), ('2:15', 'influence_score', 0.55), ('2:15', 'difficulty_score', 0.45), ('2:15', 'quran_order', 2015000),
    ('2:16', 'foundational_score', 0.66), ('2:16', 'influence_score', 0.56), ('2:16', 'difficulty_score', 0.46), ('2:16', 'quran_order', 2016000),
    ('2:17', 'foundational_score', 0.67), ('2:17', 'influence_score', 0.57), ('2:17', 'difficulty_score', 0.47), ('2:17', 'quran_order', 2017000),
    ('2:18', 'foundational_score', 0.68), ('2:18', 'influence_score', 0.58), ('2:18', 'difficulty_score', 0.48), ('2:18', 'quran_order', 2018000),
    ('2:19', 'foundational_score', 0.69), ('2:19', 'influence_score', 0.59), ('2:19', 'difficulty_score', 0.49), ('2:19', 'quran_order', 2019000),
    ('2:20', 'foundational_score', 0.70), ('2:20', 'influence_score', 0.60), ('2:20', 'difficulty_score', 0.50), ('2:20', 'quran_order', 2020000);

-- Goal for Al-Baqarah first 20 verses
INSERT OR IGNORE INTO goals (goal_id, goal_type, goal_group, label, description) VALUES
    ('memorization:surah-2-part1', 'surah', 'memorization', 'Memorize Al-Baqarah (Part 1)', 'First 20 verses of Al-Baqarah');

-- Node-goal mappings for Al-Baqarah
INSERT OR IGNORE INTO node_goals (goal_id, node_id, priority) VALUES
    ('memorization:surah-2-part1', '2:1', 1), ('memorization:surah-2-part1', '2:2', 2), ('memorization:surah-2-part1', '2:3', 3),
    ('memorization:surah-2-part1', '2:4', 4), ('memorization:surah-2-part1', '2:5', 5), ('memorization:surah-2-part1', '2:6', 6),
    ('memorization:surah-2-part1', '2:7', 7), ('memorization:surah-2-part1', '2:8', 8), ('memorization:surah-2-part1', '2:9', 9),
    ('memorization:surah-2-part1', '2:10', 10), ('memorization:surah-2-part1', '2:11', 11), ('memorization:surah-2-part1', '2:12', 12),
    ('memorization:surah-2-part1', '2:13', 13), ('memorization:surah-2-part1', '2:14', 14), ('memorization:surah-2-part1', '2:15', 15),
    ('memorization:surah-2-part1', '2:16', 16), ('memorization:surah-2-part1', '2:17', 17), ('memorization:surah-2-part1', '2:18', 18),
    ('memorization:surah-2-part1', '2:19', 19), ('memorization:surah-2-part1', '2:20', 20);

-- Sequential prerequisites for Al-Baqarah (each verse requires previous)
INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2) VALUES
    ('2:1', '2:2', 0, 0, 0.0, 0.0), ('2:2', '2:3', 0, 0, 0.0, 0.0), ('2:3', '2:4', 0, 0, 0.0, 0.0),
    ('2:4', '2:5', 0, 0, 0.0, 0.0), ('2:5', '2:6', 0, 0, 0.0, 0.0), ('2:6', '2:7', 0, 0, 0.0, 0.0),
    ('2:7', '2:8', 0, 0, 0.0, 0.0), ('2:8', '2:9', 0, 0, 0.0, 0.0), ('2:9', '2:10', 0, 0, 0.0, 0.0),
    ('2:10', '2:11', 0, 0, 0.0, 0.0), ('2:11', '2:12', 0, 0, 0.0, 0.0), ('2:12', '2:13', 0, 0, 0.0, 0.0),
    ('2:13', '2:14', 0, 0, 0.0, 0.0), ('2:14', '2:15', 0, 0, 0.0, 0.0), ('2:15', '2:16', 0, 0, 0.0, 0.0),
    ('2:16', '2:17', 0, 0, 0.0, 0.0), ('2:17', '2:18', 0, 0, 0.0, 0.0), ('2:18', '2:19', 0, 0, 0.0, 0.0),
    ('2:19', '2:20', 0, 0, 0.0, 0.0);
