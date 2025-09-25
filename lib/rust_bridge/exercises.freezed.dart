// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'exercises.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$Exercise {
  String get nodeId => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId, String arabic, String translation)
    recall,
    required TResult Function(String nodeId, String question, String answer)
    cloze,
    required TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )
    mcqArToEn,
    required TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )
    mcqEnToAr,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId, String arabic, String translation)? recall,
    TResult? Function(String nodeId, String question, String answer)? cloze,
    TResult? Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult? Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId, String arabic, String translation)? recall,
    TResult Function(String nodeId, String question, String answer)? cloze,
    TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Exercise_Recall value) recall,
    required TResult Function(Exercise_Cloze value) cloze,
    required TResult Function(Exercise_McqArToEn value) mcqArToEn,
    required TResult Function(Exercise_McqEnToAr value) mcqEnToAr,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Exercise_Recall value)? recall,
    TResult? Function(Exercise_Cloze value)? cloze,
    TResult? Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult? Function(Exercise_McqEnToAr value)? mcqEnToAr,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Exercise_Recall value)? recall,
    TResult Function(Exercise_Cloze value)? cloze,
    TResult Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult Function(Exercise_McqEnToAr value)? mcqEnToAr,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ExerciseCopyWith<Exercise> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ExerciseCopyWith<$Res> {
  factory $ExerciseCopyWith(Exercise value, $Res Function(Exercise) then) =
      _$ExerciseCopyWithImpl<$Res, Exercise>;
  @useResult
  $Res call({String nodeId});
}

/// @nodoc
class _$ExerciseCopyWithImpl<$Res, $Val extends Exercise>
    implements $ExerciseCopyWith<$Res> {
  _$ExerciseCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null}) {
    return _then(
      _value.copyWith(
            nodeId: null == nodeId
                ? _value.nodeId
                : nodeId // ignore: cast_nullable_to_non_nullable
                      as String,
          )
          as $Val,
    );
  }
}

/// @nodoc
abstract class _$$Exercise_RecallImplCopyWith<$Res>
    implements $ExerciseCopyWith<$Res> {
  factory _$$Exercise_RecallImplCopyWith(
    _$Exercise_RecallImpl value,
    $Res Function(_$Exercise_RecallImpl) then,
  ) = __$$Exercise_RecallImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String nodeId, String arabic, String translation});
}

/// @nodoc
class __$$Exercise_RecallImplCopyWithImpl<$Res>
    extends _$ExerciseCopyWithImpl<$Res, _$Exercise_RecallImpl>
    implements _$$Exercise_RecallImplCopyWith<$Res> {
  __$$Exercise_RecallImplCopyWithImpl(
    _$Exercise_RecallImpl _value,
    $Res Function(_$Exercise_RecallImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? arabic = null,
    Object? translation = null,
  }) {
    return _then(
      _$Exercise_RecallImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        arabic: null == arabic
            ? _value.arabic
            : arabic // ignore: cast_nullable_to_non_nullable
                  as String,
        translation: null == translation
            ? _value.translation
            : translation // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$Exercise_RecallImpl extends Exercise_Recall {
  const _$Exercise_RecallImpl({
    required this.nodeId,
    required this.arabic,
    required this.translation,
  }) : super._();

  @override
  final String nodeId;
  @override
  final String arabic;
  @override
  final String translation;

  @override
  String toString() {
    return 'Exercise.recall(nodeId: $nodeId, arabic: $arabic, translation: $translation)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Exercise_RecallImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.arabic, arabic) || other.arabic == arabic) &&
            (identical(other.translation, translation) ||
                other.translation == translation));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, arabic, translation);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Exercise_RecallImplCopyWith<_$Exercise_RecallImpl> get copyWith =>
      __$$Exercise_RecallImplCopyWithImpl<_$Exercise_RecallImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId, String arabic, String translation)
    recall,
    required TResult Function(String nodeId, String question, String answer)
    cloze,
    required TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )
    mcqArToEn,
    required TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )
    mcqEnToAr,
  }) {
    return recall(nodeId, arabic, translation);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId, String arabic, String translation)? recall,
    TResult? Function(String nodeId, String question, String answer)? cloze,
    TResult? Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult? Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
  }) {
    return recall?.call(nodeId, arabic, translation);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId, String arabic, String translation)? recall,
    TResult Function(String nodeId, String question, String answer)? cloze,
    TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
    required TResult orElse(),
  }) {
    if (recall != null) {
      return recall(nodeId, arabic, translation);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Exercise_Recall value) recall,
    required TResult Function(Exercise_Cloze value) cloze,
    required TResult Function(Exercise_McqArToEn value) mcqArToEn,
    required TResult Function(Exercise_McqEnToAr value) mcqEnToAr,
  }) {
    return recall(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Exercise_Recall value)? recall,
    TResult? Function(Exercise_Cloze value)? cloze,
    TResult? Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult? Function(Exercise_McqEnToAr value)? mcqEnToAr,
  }) {
    return recall?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Exercise_Recall value)? recall,
    TResult Function(Exercise_Cloze value)? cloze,
    TResult Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult Function(Exercise_McqEnToAr value)? mcqEnToAr,
    required TResult orElse(),
  }) {
    if (recall != null) {
      return recall(this);
    }
    return orElse();
  }
}

abstract class Exercise_Recall extends Exercise {
  const factory Exercise_Recall({
    required final String nodeId,
    required final String arabic,
    required final String translation,
  }) = _$Exercise_RecallImpl;
  const Exercise_Recall._() : super._();

  @override
  String get nodeId;
  String get arabic;
  String get translation;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Exercise_RecallImplCopyWith<_$Exercise_RecallImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Exercise_ClozeImplCopyWith<$Res>
    implements $ExerciseCopyWith<$Res> {
  factory _$$Exercise_ClozeImplCopyWith(
    _$Exercise_ClozeImpl value,
    $Res Function(_$Exercise_ClozeImpl) then,
  ) = __$$Exercise_ClozeImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String nodeId, String question, String answer});
}

/// @nodoc
class __$$Exercise_ClozeImplCopyWithImpl<$Res>
    extends _$ExerciseCopyWithImpl<$Res, _$Exercise_ClozeImpl>
    implements _$$Exercise_ClozeImplCopyWith<$Res> {
  __$$Exercise_ClozeImplCopyWithImpl(
    _$Exercise_ClozeImpl _value,
    $Res Function(_$Exercise_ClozeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? question = null,
    Object? answer = null,
  }) {
    return _then(
      _$Exercise_ClozeImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        question: null == question
            ? _value.question
            : question // ignore: cast_nullable_to_non_nullable
                  as String,
        answer: null == answer
            ? _value.answer
            : answer // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$Exercise_ClozeImpl extends Exercise_Cloze {
  const _$Exercise_ClozeImpl({
    required this.nodeId,
    required this.question,
    required this.answer,
  }) : super._();

  @override
  final String nodeId;
  @override
  final String question;
  @override
  final String answer;

  @override
  String toString() {
    return 'Exercise.cloze(nodeId: $nodeId, question: $question, answer: $answer)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Exercise_ClozeImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.question, question) ||
                other.question == question) &&
            (identical(other.answer, answer) || other.answer == answer));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, question, answer);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Exercise_ClozeImplCopyWith<_$Exercise_ClozeImpl> get copyWith =>
      __$$Exercise_ClozeImplCopyWithImpl<_$Exercise_ClozeImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId, String arabic, String translation)
    recall,
    required TResult Function(String nodeId, String question, String answer)
    cloze,
    required TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )
    mcqArToEn,
    required TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )
    mcqEnToAr,
  }) {
    return cloze(nodeId, question, answer);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId, String arabic, String translation)? recall,
    TResult? Function(String nodeId, String question, String answer)? cloze,
    TResult? Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult? Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
  }) {
    return cloze?.call(nodeId, question, answer);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId, String arabic, String translation)? recall,
    TResult Function(String nodeId, String question, String answer)? cloze,
    TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
    required TResult orElse(),
  }) {
    if (cloze != null) {
      return cloze(nodeId, question, answer);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Exercise_Recall value) recall,
    required TResult Function(Exercise_Cloze value) cloze,
    required TResult Function(Exercise_McqArToEn value) mcqArToEn,
    required TResult Function(Exercise_McqEnToAr value) mcqEnToAr,
  }) {
    return cloze(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Exercise_Recall value)? recall,
    TResult? Function(Exercise_Cloze value)? cloze,
    TResult? Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult? Function(Exercise_McqEnToAr value)? mcqEnToAr,
  }) {
    return cloze?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Exercise_Recall value)? recall,
    TResult Function(Exercise_Cloze value)? cloze,
    TResult Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult Function(Exercise_McqEnToAr value)? mcqEnToAr,
    required TResult orElse(),
  }) {
    if (cloze != null) {
      return cloze(this);
    }
    return orElse();
  }
}

abstract class Exercise_Cloze extends Exercise {
  const factory Exercise_Cloze({
    required final String nodeId,
    required final String question,
    required final String answer,
  }) = _$Exercise_ClozeImpl;
  const Exercise_Cloze._() : super._();

  @override
  String get nodeId;
  String get question;
  String get answer;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Exercise_ClozeImplCopyWith<_$Exercise_ClozeImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Exercise_McqArToEnImplCopyWith<$Res>
    implements $ExerciseCopyWith<$Res> {
  factory _$$Exercise_McqArToEnImplCopyWith(
    _$Exercise_McqArToEnImpl value,
    $Res Function(_$Exercise_McqArToEnImpl) then,
  ) = __$$Exercise_McqArToEnImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String nodeId,
    String arabic,
    String verseArabic,
    int surahNumber,
    int ayahNumber,
    int wordIndex,
    List<String> choicesEn,
    int correctIndex,
  });
}

/// @nodoc
class __$$Exercise_McqArToEnImplCopyWithImpl<$Res>
    extends _$ExerciseCopyWithImpl<$Res, _$Exercise_McqArToEnImpl>
    implements _$$Exercise_McqArToEnImplCopyWith<$Res> {
  __$$Exercise_McqArToEnImplCopyWithImpl(
    _$Exercise_McqArToEnImpl _value,
    $Res Function(_$Exercise_McqArToEnImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? arabic = null,
    Object? verseArabic = null,
    Object? surahNumber = null,
    Object? ayahNumber = null,
    Object? wordIndex = null,
    Object? choicesEn = null,
    Object? correctIndex = null,
  }) {
    return _then(
      _$Exercise_McqArToEnImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        arabic: null == arabic
            ? _value.arabic
            : arabic // ignore: cast_nullable_to_non_nullable
                  as String,
        verseArabic: null == verseArabic
            ? _value.verseArabic
            : verseArabic // ignore: cast_nullable_to_non_nullable
                  as String,
        surahNumber: null == surahNumber
            ? _value.surahNumber
            : surahNumber // ignore: cast_nullable_to_non_nullable
                  as int,
        ayahNumber: null == ayahNumber
            ? _value.ayahNumber
            : ayahNumber // ignore: cast_nullable_to_non_nullable
                  as int,
        wordIndex: null == wordIndex
            ? _value.wordIndex
            : wordIndex // ignore: cast_nullable_to_non_nullable
                  as int,
        choicesEn: null == choicesEn
            ? _value._choicesEn
            : choicesEn // ignore: cast_nullable_to_non_nullable
                  as List<String>,
        correctIndex: null == correctIndex
            ? _value.correctIndex
            : correctIndex // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$Exercise_McqArToEnImpl extends Exercise_McqArToEn {
  const _$Exercise_McqArToEnImpl({
    required this.nodeId,
    required this.arabic,
    required this.verseArabic,
    required this.surahNumber,
    required this.ayahNumber,
    required this.wordIndex,
    required final List<String> choicesEn,
    required this.correctIndex,
  }) : _choicesEn = choicesEn,
       super._();

  @override
  final String nodeId;
  @override
  final String arabic;
  @override
  final String verseArabic;
  @override
  final int surahNumber;
  @override
  final int ayahNumber;
  @override
  final int wordIndex;
  final List<String> _choicesEn;
  @override
  List<String> get choicesEn {
    if (_choicesEn is EqualUnmodifiableListView) return _choicesEn;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_choicesEn);
  }

  @override
  final int correctIndex;

  @override
  String toString() {
    return 'Exercise.mcqArToEn(nodeId: $nodeId, arabic: $arabic, verseArabic: $verseArabic, surahNumber: $surahNumber, ayahNumber: $ayahNumber, wordIndex: $wordIndex, choicesEn: $choicesEn, correctIndex: $correctIndex)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Exercise_McqArToEnImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.arabic, arabic) || other.arabic == arabic) &&
            (identical(other.verseArabic, verseArabic) ||
                other.verseArabic == verseArabic) &&
            (identical(other.surahNumber, surahNumber) ||
                other.surahNumber == surahNumber) &&
            (identical(other.ayahNumber, ayahNumber) ||
                other.ayahNumber == ayahNumber) &&
            (identical(other.wordIndex, wordIndex) ||
                other.wordIndex == wordIndex) &&
            const DeepCollectionEquality().equals(
              other._choicesEn,
              _choicesEn,
            ) &&
            (identical(other.correctIndex, correctIndex) ||
                other.correctIndex == correctIndex));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    arabic,
    verseArabic,
    surahNumber,
    ayahNumber,
    wordIndex,
    const DeepCollectionEquality().hash(_choicesEn),
    correctIndex,
  );

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Exercise_McqArToEnImplCopyWith<_$Exercise_McqArToEnImpl> get copyWith =>
      __$$Exercise_McqArToEnImplCopyWithImpl<_$Exercise_McqArToEnImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId, String arabic, String translation)
    recall,
    required TResult Function(String nodeId, String question, String answer)
    cloze,
    required TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )
    mcqArToEn,
    required TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )
    mcqEnToAr,
  }) {
    return mcqArToEn(
      nodeId,
      arabic,
      verseArabic,
      surahNumber,
      ayahNumber,
      wordIndex,
      choicesEn,
      correctIndex,
    );
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId, String arabic, String translation)? recall,
    TResult? Function(String nodeId, String question, String answer)? cloze,
    TResult? Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult? Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
  }) {
    return mcqArToEn?.call(
      nodeId,
      arabic,
      verseArabic,
      surahNumber,
      ayahNumber,
      wordIndex,
      choicesEn,
      correctIndex,
    );
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId, String arabic, String translation)? recall,
    TResult Function(String nodeId, String question, String answer)? cloze,
    TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
    required TResult orElse(),
  }) {
    if (mcqArToEn != null) {
      return mcqArToEn(
        nodeId,
        arabic,
        verseArabic,
        surahNumber,
        ayahNumber,
        wordIndex,
        choicesEn,
        correctIndex,
      );
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Exercise_Recall value) recall,
    required TResult Function(Exercise_Cloze value) cloze,
    required TResult Function(Exercise_McqArToEn value) mcqArToEn,
    required TResult Function(Exercise_McqEnToAr value) mcqEnToAr,
  }) {
    return mcqArToEn(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Exercise_Recall value)? recall,
    TResult? Function(Exercise_Cloze value)? cloze,
    TResult? Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult? Function(Exercise_McqEnToAr value)? mcqEnToAr,
  }) {
    return mcqArToEn?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Exercise_Recall value)? recall,
    TResult Function(Exercise_Cloze value)? cloze,
    TResult Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult Function(Exercise_McqEnToAr value)? mcqEnToAr,
    required TResult orElse(),
  }) {
    if (mcqArToEn != null) {
      return mcqArToEn(this);
    }
    return orElse();
  }
}

abstract class Exercise_McqArToEn extends Exercise {
  const factory Exercise_McqArToEn({
    required final String nodeId,
    required final String arabic,
    required final String verseArabic,
    required final int surahNumber,
    required final int ayahNumber,
    required final int wordIndex,
    required final List<String> choicesEn,
    required final int correctIndex,
  }) = _$Exercise_McqArToEnImpl;
  const Exercise_McqArToEn._() : super._();

  @override
  String get nodeId;
  String get arabic;
  String get verseArabic;
  int get surahNumber;
  int get ayahNumber;
  int get wordIndex;
  List<String> get choicesEn;
  int get correctIndex;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Exercise_McqArToEnImplCopyWith<_$Exercise_McqArToEnImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$Exercise_McqEnToArImplCopyWith<$Res>
    implements $ExerciseCopyWith<$Res> {
  factory _$$Exercise_McqEnToArImplCopyWith(
    _$Exercise_McqEnToArImpl value,
    $Res Function(_$Exercise_McqEnToArImpl) then,
  ) = __$$Exercise_McqEnToArImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({
    String nodeId,
    String english,
    String verseArabic,
    int surahNumber,
    int ayahNumber,
    int wordIndex,
    List<String> choicesAr,
    int correctIndex,
  });
}

/// @nodoc
class __$$Exercise_McqEnToArImplCopyWithImpl<$Res>
    extends _$ExerciseCopyWithImpl<$Res, _$Exercise_McqEnToArImpl>
    implements _$$Exercise_McqEnToArImplCopyWith<$Res> {
  __$$Exercise_McqEnToArImplCopyWithImpl(
    _$Exercise_McqEnToArImpl _value,
    $Res Function(_$Exercise_McqEnToArImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? english = null,
    Object? verseArabic = null,
    Object? surahNumber = null,
    Object? ayahNumber = null,
    Object? wordIndex = null,
    Object? choicesAr = null,
    Object? correctIndex = null,
  }) {
    return _then(
      _$Exercise_McqEnToArImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        english: null == english
            ? _value.english
            : english // ignore: cast_nullable_to_non_nullable
                  as String,
        verseArabic: null == verseArabic
            ? _value.verseArabic
            : verseArabic // ignore: cast_nullable_to_non_nullable
                  as String,
        surahNumber: null == surahNumber
            ? _value.surahNumber
            : surahNumber // ignore: cast_nullable_to_non_nullable
                  as int,
        ayahNumber: null == ayahNumber
            ? _value.ayahNumber
            : ayahNumber // ignore: cast_nullable_to_non_nullable
                  as int,
        wordIndex: null == wordIndex
            ? _value.wordIndex
            : wordIndex // ignore: cast_nullable_to_non_nullable
                  as int,
        choicesAr: null == choicesAr
            ? _value._choicesAr
            : choicesAr // ignore: cast_nullable_to_non_nullable
                  as List<String>,
        correctIndex: null == correctIndex
            ? _value.correctIndex
            : correctIndex // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$Exercise_McqEnToArImpl extends Exercise_McqEnToAr {
  const _$Exercise_McqEnToArImpl({
    required this.nodeId,
    required this.english,
    required this.verseArabic,
    required this.surahNumber,
    required this.ayahNumber,
    required this.wordIndex,
    required final List<String> choicesAr,
    required this.correctIndex,
  }) : _choicesAr = choicesAr,
       super._();

  @override
  final String nodeId;
  @override
  final String english;
  @override
  final String verseArabic;
  @override
  final int surahNumber;
  @override
  final int ayahNumber;
  @override
  final int wordIndex;
  final List<String> _choicesAr;
  @override
  List<String> get choicesAr {
    if (_choicesAr is EqualUnmodifiableListView) return _choicesAr;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_choicesAr);
  }

  @override
  final int correctIndex;

  @override
  String toString() {
    return 'Exercise.mcqEnToAr(nodeId: $nodeId, english: $english, verseArabic: $verseArabic, surahNumber: $surahNumber, ayahNumber: $ayahNumber, wordIndex: $wordIndex, choicesAr: $choicesAr, correctIndex: $correctIndex)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$Exercise_McqEnToArImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.english, english) || other.english == english) &&
            (identical(other.verseArabic, verseArabic) ||
                other.verseArabic == verseArabic) &&
            (identical(other.surahNumber, surahNumber) ||
                other.surahNumber == surahNumber) &&
            (identical(other.ayahNumber, ayahNumber) ||
                other.ayahNumber == ayahNumber) &&
            (identical(other.wordIndex, wordIndex) ||
                other.wordIndex == wordIndex) &&
            const DeepCollectionEquality().equals(
              other._choicesAr,
              _choicesAr,
            ) &&
            (identical(other.correctIndex, correctIndex) ||
                other.correctIndex == correctIndex));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    english,
    verseArabic,
    surahNumber,
    ayahNumber,
    wordIndex,
    const DeepCollectionEquality().hash(_choicesAr),
    correctIndex,
  );

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$Exercise_McqEnToArImplCopyWith<_$Exercise_McqEnToArImpl> get copyWith =>
      __$$Exercise_McqEnToArImplCopyWithImpl<_$Exercise_McqEnToArImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId, String arabic, String translation)
    recall,
    required TResult Function(String nodeId, String question, String answer)
    cloze,
    required TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )
    mcqArToEn,
    required TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )
    mcqEnToAr,
  }) {
    return mcqEnToAr(
      nodeId,
      english,
      verseArabic,
      surahNumber,
      ayahNumber,
      wordIndex,
      choicesAr,
      correctIndex,
    );
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId, String arabic, String translation)? recall,
    TResult? Function(String nodeId, String question, String answer)? cloze,
    TResult? Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult? Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
  }) {
    return mcqEnToAr?.call(
      nodeId,
      english,
      verseArabic,
      surahNumber,
      ayahNumber,
      wordIndex,
      choicesAr,
      correctIndex,
    );
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId, String arabic, String translation)? recall,
    TResult Function(String nodeId, String question, String answer)? cloze,
    TResult Function(
      String nodeId,
      String arabic,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesEn,
      int correctIndex,
    )?
    mcqArToEn,
    TResult Function(
      String nodeId,
      String english,
      String verseArabic,
      int surahNumber,
      int ayahNumber,
      int wordIndex,
      List<String> choicesAr,
      int correctIndex,
    )?
    mcqEnToAr,
    required TResult orElse(),
  }) {
    if (mcqEnToAr != null) {
      return mcqEnToAr(
        nodeId,
        english,
        verseArabic,
        surahNumber,
        ayahNumber,
        wordIndex,
        choicesAr,
        correctIndex,
      );
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(Exercise_Recall value) recall,
    required TResult Function(Exercise_Cloze value) cloze,
    required TResult Function(Exercise_McqArToEn value) mcqArToEn,
    required TResult Function(Exercise_McqEnToAr value) mcqEnToAr,
  }) {
    return mcqEnToAr(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(Exercise_Recall value)? recall,
    TResult? Function(Exercise_Cloze value)? cloze,
    TResult? Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult? Function(Exercise_McqEnToAr value)? mcqEnToAr,
  }) {
    return mcqEnToAr?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(Exercise_Recall value)? recall,
    TResult Function(Exercise_Cloze value)? cloze,
    TResult Function(Exercise_McqArToEn value)? mcqArToEn,
    TResult Function(Exercise_McqEnToAr value)? mcqEnToAr,
    required TResult orElse(),
  }) {
    if (mcqEnToAr != null) {
      return mcqEnToAr(this);
    }
    return orElse();
  }
}

abstract class Exercise_McqEnToAr extends Exercise {
  const factory Exercise_McqEnToAr({
    required final String nodeId,
    required final String english,
    required final String verseArabic,
    required final int surahNumber,
    required final int ayahNumber,
    required final int wordIndex,
    required final List<String> choicesAr,
    required final int correctIndex,
  }) = _$Exercise_McqEnToArImpl;
  const Exercise_McqEnToAr._() : super._();

  @override
  String get nodeId;
  String get english;
  String get verseArabic;
  int get surahNumber;
  int get ayahNumber;
  int get wordIndex;
  List<String> get choicesAr;
  int get correctIndex;

  /// Create a copy of Exercise
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$Exercise_McqEnToArImplCopyWith<_$Exercise_McqEnToArImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
