// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$ExerciseDataDto {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ExerciseDataDtoCopyWith<$Res> {
  factory $ExerciseDataDtoCopyWith(
    ExerciseDataDto value,
    $Res Function(ExerciseDataDto) then,
  ) = _$ExerciseDataDtoCopyWithImpl<$Res, ExerciseDataDto>;
}

/// @nodoc
class _$ExerciseDataDtoCopyWithImpl<$Res, $Val extends ExerciseDataDto>
    implements $ExerciseDataDtoCopyWith<$Res> {
  _$ExerciseDataDtoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ExerciseDataDto_MemorizationImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_MemorizationImplCopyWith(
    _$ExerciseDataDto_MemorizationImpl value,
    $Res Function(_$ExerciseDataDto_MemorizationImpl) then,
  ) = __$$ExerciseDataDto_MemorizationImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId});
}

/// @nodoc
class __$$ExerciseDataDto_MemorizationImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_MemorizationImpl>
    implements _$$ExerciseDataDto_MemorizationImplCopyWith<$Res> {
  __$$ExerciseDataDto_MemorizationImplCopyWithImpl(
    _$ExerciseDataDto_MemorizationImpl _value,
    $Res Function(_$ExerciseDataDto_MemorizationImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null}) {
    return _then(
      _$ExerciseDataDto_MemorizationImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_MemorizationImpl extends ExerciseDataDto_Memorization {
  const _$ExerciseDataDto_MemorizationImpl({required this.nodeId}) : super._();

  @override
  final String nodeId;

  @override
  String toString() {
    return 'ExerciseDataDto.memorization(nodeId: $nodeId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_MemorizationImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_MemorizationImplCopyWith<
    _$ExerciseDataDto_MemorizationImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_MemorizationImplCopyWithImpl<
        _$ExerciseDataDto_MemorizationImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return memorization(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return memorization?.call(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (memorization != null) {
      return memorization(nodeId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return memorization(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return memorization?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (memorization != null) {
      return memorization(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_Memorization extends ExerciseDataDto {
  const factory ExerciseDataDto_Memorization({required final String nodeId}) =
      _$ExerciseDataDto_MemorizationImpl;
  const ExerciseDataDto_Memorization._() : super._();

  String get nodeId;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_MemorizationImplCopyWith<
    _$ExerciseDataDto_MemorizationImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_McqArToEnImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_McqArToEnImplCopyWith(
    _$ExerciseDataDto_McqArToEnImpl value,
    $Res Function(_$ExerciseDataDto_McqArToEnImpl) then,
  ) = __$$ExerciseDataDto_McqArToEnImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, List<String> distractorNodeIds});
}

/// @nodoc
class __$$ExerciseDataDto_McqArToEnImplCopyWithImpl<$Res>
    extends _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_McqArToEnImpl>
    implements _$$ExerciseDataDto_McqArToEnImplCopyWith<$Res> {
  __$$ExerciseDataDto_McqArToEnImplCopyWithImpl(
    _$ExerciseDataDto_McqArToEnImpl _value,
    $Res Function(_$ExerciseDataDto_McqArToEnImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? distractorNodeIds = null}) {
    return _then(
      _$ExerciseDataDto_McqArToEnImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        distractorNodeIds: null == distractorNodeIds
            ? _value._distractorNodeIds
            : distractorNodeIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_McqArToEnImpl extends ExerciseDataDto_McqArToEn {
  const _$ExerciseDataDto_McqArToEnImpl({
    required this.nodeId,
    required final List<String> distractorNodeIds,
  }) : _distractorNodeIds = distractorNodeIds,
       super._();

  @override
  final String nodeId;
  final List<String> _distractorNodeIds;
  @override
  List<String> get distractorNodeIds {
    if (_distractorNodeIds is EqualUnmodifiableListView)
      return _distractorNodeIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_distractorNodeIds);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.mcqArToEn(nodeId: $nodeId, distractorNodeIds: $distractorNodeIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_McqArToEnImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            const DeepCollectionEquality().equals(
              other._distractorNodeIds,
              _distractorNodeIds,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    const DeepCollectionEquality().hash(_distractorNodeIds),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_McqArToEnImplCopyWith<_$ExerciseDataDto_McqArToEnImpl>
  get copyWith =>
      __$$ExerciseDataDto_McqArToEnImplCopyWithImpl<
        _$ExerciseDataDto_McqArToEnImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return mcqArToEn(nodeId, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return mcqArToEn?.call(nodeId, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (mcqArToEn != null) {
      return mcqArToEn(nodeId, distractorNodeIds);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return mcqArToEn(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return mcqArToEn?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (mcqArToEn != null) {
      return mcqArToEn(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_McqArToEn extends ExerciseDataDto {
  const factory ExerciseDataDto_McqArToEn({
    required final String nodeId,
    required final List<String> distractorNodeIds,
  }) = _$ExerciseDataDto_McqArToEnImpl;
  const ExerciseDataDto_McqArToEn._() : super._();

  String get nodeId;
  List<String> get distractorNodeIds;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_McqArToEnImplCopyWith<_$ExerciseDataDto_McqArToEnImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_McqEnToArImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_McqEnToArImplCopyWith(
    _$ExerciseDataDto_McqEnToArImpl value,
    $Res Function(_$ExerciseDataDto_McqEnToArImpl) then,
  ) = __$$ExerciseDataDto_McqEnToArImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, List<String> distractorNodeIds});
}

/// @nodoc
class __$$ExerciseDataDto_McqEnToArImplCopyWithImpl<$Res>
    extends _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_McqEnToArImpl>
    implements _$$ExerciseDataDto_McqEnToArImplCopyWith<$Res> {
  __$$ExerciseDataDto_McqEnToArImplCopyWithImpl(
    _$ExerciseDataDto_McqEnToArImpl _value,
    $Res Function(_$ExerciseDataDto_McqEnToArImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? distractorNodeIds = null}) {
    return _then(
      _$ExerciseDataDto_McqEnToArImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        distractorNodeIds: null == distractorNodeIds
            ? _value._distractorNodeIds
            : distractorNodeIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_McqEnToArImpl extends ExerciseDataDto_McqEnToAr {
  const _$ExerciseDataDto_McqEnToArImpl({
    required this.nodeId,
    required final List<String> distractorNodeIds,
  }) : _distractorNodeIds = distractorNodeIds,
       super._();

  @override
  final String nodeId;
  final List<String> _distractorNodeIds;
  @override
  List<String> get distractorNodeIds {
    if (_distractorNodeIds is EqualUnmodifiableListView)
      return _distractorNodeIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_distractorNodeIds);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.mcqEnToAr(nodeId: $nodeId, distractorNodeIds: $distractorNodeIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_McqEnToArImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            const DeepCollectionEquality().equals(
              other._distractorNodeIds,
              _distractorNodeIds,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    const DeepCollectionEquality().hash(_distractorNodeIds),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_McqEnToArImplCopyWith<_$ExerciseDataDto_McqEnToArImpl>
  get copyWith =>
      __$$ExerciseDataDto_McqEnToArImplCopyWithImpl<
        _$ExerciseDataDto_McqEnToArImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return mcqEnToAr(nodeId, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return mcqEnToAr?.call(nodeId, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (mcqEnToAr != null) {
      return mcqEnToAr(nodeId, distractorNodeIds);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return mcqEnToAr(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return mcqEnToAr?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (mcqEnToAr != null) {
      return mcqEnToAr(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_McqEnToAr extends ExerciseDataDto {
  const factory ExerciseDataDto_McqEnToAr({
    required final String nodeId,
    required final List<String> distractorNodeIds,
  }) = _$ExerciseDataDto_McqEnToArImpl;
  const ExerciseDataDto_McqEnToAr._() : super._();

  String get nodeId;
  List<String> get distractorNodeIds;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_McqEnToArImplCopyWith<_$ExerciseDataDto_McqEnToArImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_TranslationImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_TranslationImplCopyWith(
    _$ExerciseDataDto_TranslationImpl value,
    $Res Function(_$ExerciseDataDto_TranslationImpl) then,
  ) = __$$ExerciseDataDto_TranslationImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId});
}

/// @nodoc
class __$$ExerciseDataDto_TranslationImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_TranslationImpl>
    implements _$$ExerciseDataDto_TranslationImplCopyWith<$Res> {
  __$$ExerciseDataDto_TranslationImplCopyWithImpl(
    _$ExerciseDataDto_TranslationImpl _value,
    $Res Function(_$ExerciseDataDto_TranslationImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null}) {
    return _then(
      _$ExerciseDataDto_TranslationImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_TranslationImpl extends ExerciseDataDto_Translation {
  const _$ExerciseDataDto_TranslationImpl({required this.nodeId}) : super._();

  @override
  final String nodeId;

  @override
  String toString() {
    return 'ExerciseDataDto.translation(nodeId: $nodeId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_TranslationImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_TranslationImplCopyWith<_$ExerciseDataDto_TranslationImpl>
  get copyWith =>
      __$$ExerciseDataDto_TranslationImplCopyWithImpl<
        _$ExerciseDataDto_TranslationImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return translation(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return translation?.call(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (translation != null) {
      return translation(nodeId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return translation(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return translation?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (translation != null) {
      return translation(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_Translation extends ExerciseDataDto {
  const factory ExerciseDataDto_Translation({required final String nodeId}) =
      _$ExerciseDataDto_TranslationImpl;
  const ExerciseDataDto_Translation._() : super._();

  String get nodeId;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_TranslationImplCopyWith<_$ExerciseDataDto_TranslationImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_ContextualTranslationImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_ContextualTranslationImplCopyWith(
    _$ExerciseDataDto_ContextualTranslationImpl value,
    $Res Function(_$ExerciseDataDto_ContextualTranslationImpl) then,
  ) = __$$ExerciseDataDto_ContextualTranslationImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, String verseKey});
}

/// @nodoc
class __$$ExerciseDataDto_ContextualTranslationImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_ContextualTranslationImpl
        >
    implements _$$ExerciseDataDto_ContextualTranslationImplCopyWith<$Res> {
  __$$ExerciseDataDto_ContextualTranslationImplCopyWithImpl(
    _$ExerciseDataDto_ContextualTranslationImpl _value,
    $Res Function(_$ExerciseDataDto_ContextualTranslationImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? verseKey = null}) {
    return _then(
      _$ExerciseDataDto_ContextualTranslationImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        verseKey: null == verseKey
            ? _value.verseKey
            : verseKey // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_ContextualTranslationImpl
    extends ExerciseDataDto_ContextualTranslation {
  const _$ExerciseDataDto_ContextualTranslationImpl({
    required this.nodeId,
    required this.verseKey,
  }) : super._();

  @override
  final String nodeId;
  @override
  final String verseKey;

  @override
  String toString() {
    return 'ExerciseDataDto.contextualTranslation(nodeId: $nodeId, verseKey: $verseKey)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_ContextualTranslationImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.verseKey, verseKey) ||
                other.verseKey == verseKey));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, verseKey);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_ContextualTranslationImplCopyWith<
    _$ExerciseDataDto_ContextualTranslationImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_ContextualTranslationImplCopyWithImpl<
        _$ExerciseDataDto_ContextualTranslationImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return contextualTranslation(nodeId, verseKey);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return contextualTranslation?.call(nodeId, verseKey);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (contextualTranslation != null) {
      return contextualTranslation(nodeId, verseKey);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return contextualTranslation(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return contextualTranslation?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (contextualTranslation != null) {
      return contextualTranslation(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_ContextualTranslation extends ExerciseDataDto {
  const factory ExerciseDataDto_ContextualTranslation({
    required final String nodeId,
    required final String verseKey,
  }) = _$ExerciseDataDto_ContextualTranslationImpl;
  const ExerciseDataDto_ContextualTranslation._() : super._();

  String get nodeId;
  String get verseKey;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_ContextualTranslationImplCopyWith<
    _$ExerciseDataDto_ContextualTranslationImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_ClozeDeletionImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_ClozeDeletionImplCopyWith(
    _$ExerciseDataDto_ClozeDeletionImpl value,
    $Res Function(_$ExerciseDataDto_ClozeDeletionImpl) then,
  ) = __$$ExerciseDataDto_ClozeDeletionImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, int blankPosition});
}

/// @nodoc
class __$$ExerciseDataDto_ClozeDeletionImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_ClozeDeletionImpl>
    implements _$$ExerciseDataDto_ClozeDeletionImplCopyWith<$Res> {
  __$$ExerciseDataDto_ClozeDeletionImplCopyWithImpl(
    _$ExerciseDataDto_ClozeDeletionImpl _value,
    $Res Function(_$ExerciseDataDto_ClozeDeletionImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? blankPosition = null}) {
    return _then(
      _$ExerciseDataDto_ClozeDeletionImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        blankPosition: null == blankPosition
            ? _value.blankPosition
            : blankPosition // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_ClozeDeletionImpl
    extends ExerciseDataDto_ClozeDeletion {
  const _$ExerciseDataDto_ClozeDeletionImpl({
    required this.nodeId,
    required this.blankPosition,
  }) : super._();

  @override
  final String nodeId;
  @override
  final int blankPosition;

  @override
  String toString() {
    return 'ExerciseDataDto.clozeDeletion(nodeId: $nodeId, blankPosition: $blankPosition)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_ClozeDeletionImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.blankPosition, blankPosition) ||
                other.blankPosition == blankPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, blankPosition);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_ClozeDeletionImplCopyWith<
    _$ExerciseDataDto_ClozeDeletionImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_ClozeDeletionImplCopyWithImpl<
        _$ExerciseDataDto_ClozeDeletionImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return clozeDeletion(nodeId, blankPosition);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return clozeDeletion?.call(nodeId, blankPosition);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (clozeDeletion != null) {
      return clozeDeletion(nodeId, blankPosition);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return clozeDeletion(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return clozeDeletion?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (clozeDeletion != null) {
      return clozeDeletion(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_ClozeDeletion extends ExerciseDataDto {
  const factory ExerciseDataDto_ClozeDeletion({
    required final String nodeId,
    required final int blankPosition,
  }) = _$ExerciseDataDto_ClozeDeletionImpl;
  const ExerciseDataDto_ClozeDeletion._() : super._();

  String get nodeId;
  int get blankPosition;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_ClozeDeletionImplCopyWith<
    _$ExerciseDataDto_ClozeDeletionImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_FirstLetterHintImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_FirstLetterHintImplCopyWith(
    _$ExerciseDataDto_FirstLetterHintImpl value,
    $Res Function(_$ExerciseDataDto_FirstLetterHintImpl) then,
  ) = __$$ExerciseDataDto_FirstLetterHintImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, int wordPosition});
}

/// @nodoc
class __$$ExerciseDataDto_FirstLetterHintImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_FirstLetterHintImpl
        >
    implements _$$ExerciseDataDto_FirstLetterHintImplCopyWith<$Res> {
  __$$ExerciseDataDto_FirstLetterHintImplCopyWithImpl(
    _$ExerciseDataDto_FirstLetterHintImpl _value,
    $Res Function(_$ExerciseDataDto_FirstLetterHintImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? wordPosition = null}) {
    return _then(
      _$ExerciseDataDto_FirstLetterHintImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        wordPosition: null == wordPosition
            ? _value.wordPosition
            : wordPosition // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_FirstLetterHintImpl
    extends ExerciseDataDto_FirstLetterHint {
  const _$ExerciseDataDto_FirstLetterHintImpl({
    required this.nodeId,
    required this.wordPosition,
  }) : super._();

  @override
  final String nodeId;
  @override
  final int wordPosition;

  @override
  String toString() {
    return 'ExerciseDataDto.firstLetterHint(nodeId: $nodeId, wordPosition: $wordPosition)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_FirstLetterHintImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.wordPosition, wordPosition) ||
                other.wordPosition == wordPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, wordPosition);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_FirstLetterHintImplCopyWith<
    _$ExerciseDataDto_FirstLetterHintImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_FirstLetterHintImplCopyWithImpl<
        _$ExerciseDataDto_FirstLetterHintImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return firstLetterHint(nodeId, wordPosition);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return firstLetterHint?.call(nodeId, wordPosition);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (firstLetterHint != null) {
      return firstLetterHint(nodeId, wordPosition);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return firstLetterHint(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return firstLetterHint?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (firstLetterHint != null) {
      return firstLetterHint(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_FirstLetterHint extends ExerciseDataDto {
  const factory ExerciseDataDto_FirstLetterHint({
    required final String nodeId,
    required final int wordPosition,
  }) = _$ExerciseDataDto_FirstLetterHintImpl;
  const ExerciseDataDto_FirstLetterHint._() : super._();

  String get nodeId;
  int get wordPosition;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_FirstLetterHintImplCopyWith<
    _$ExerciseDataDto_FirstLetterHintImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_MissingWordMcqImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_MissingWordMcqImplCopyWith(
    _$ExerciseDataDto_MissingWordMcqImpl value,
    $Res Function(_$ExerciseDataDto_MissingWordMcqImpl) then,
  ) = __$$ExerciseDataDto_MissingWordMcqImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, int blankPosition, List<String> distractorNodeIds});
}

/// @nodoc
class __$$ExerciseDataDto_MissingWordMcqImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_MissingWordMcqImpl
        >
    implements _$$ExerciseDataDto_MissingWordMcqImplCopyWith<$Res> {
  __$$ExerciseDataDto_MissingWordMcqImplCopyWithImpl(
    _$ExerciseDataDto_MissingWordMcqImpl _value,
    $Res Function(_$ExerciseDataDto_MissingWordMcqImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? blankPosition = null,
    Object? distractorNodeIds = null,
  }) {
    return _then(
      _$ExerciseDataDto_MissingWordMcqImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        blankPosition: null == blankPosition
            ? _value.blankPosition
            : blankPosition // ignore: cast_nullable_to_non_nullable
                  as int,
        distractorNodeIds: null == distractorNodeIds
            ? _value._distractorNodeIds
            : distractorNodeIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_MissingWordMcqImpl
    extends ExerciseDataDto_MissingWordMcq {
  const _$ExerciseDataDto_MissingWordMcqImpl({
    required this.nodeId,
    required this.blankPosition,
    required final List<String> distractorNodeIds,
  }) : _distractorNodeIds = distractorNodeIds,
       super._();

  @override
  final String nodeId;
  @override
  final int blankPosition;
  final List<String> _distractorNodeIds;
  @override
  List<String> get distractorNodeIds {
    if (_distractorNodeIds is EqualUnmodifiableListView)
      return _distractorNodeIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_distractorNodeIds);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.missingWordMcq(nodeId: $nodeId, blankPosition: $blankPosition, distractorNodeIds: $distractorNodeIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_MissingWordMcqImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.blankPosition, blankPosition) ||
                other.blankPosition == blankPosition) &&
            const DeepCollectionEquality().equals(
              other._distractorNodeIds,
              _distractorNodeIds,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    blankPosition,
    const DeepCollectionEquality().hash(_distractorNodeIds),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_MissingWordMcqImplCopyWith<
    _$ExerciseDataDto_MissingWordMcqImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_MissingWordMcqImplCopyWithImpl<
        _$ExerciseDataDto_MissingWordMcqImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return missingWordMcq(nodeId, blankPosition, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return missingWordMcq?.call(nodeId, blankPosition, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (missingWordMcq != null) {
      return missingWordMcq(nodeId, blankPosition, distractorNodeIds);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return missingWordMcq(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return missingWordMcq?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (missingWordMcq != null) {
      return missingWordMcq(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_MissingWordMcq extends ExerciseDataDto {
  const factory ExerciseDataDto_MissingWordMcq({
    required final String nodeId,
    required final int blankPosition,
    required final List<String> distractorNodeIds,
  }) = _$ExerciseDataDto_MissingWordMcqImpl;
  const ExerciseDataDto_MissingWordMcq._() : super._();

  String get nodeId;
  int get blankPosition;
  List<String> get distractorNodeIds;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_MissingWordMcqImplCopyWith<
    _$ExerciseDataDto_MissingWordMcqImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_NextWordMcqImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_NextWordMcqImplCopyWith(
    _$ExerciseDataDto_NextWordMcqImpl value,
    $Res Function(_$ExerciseDataDto_NextWordMcqImpl) then,
  ) = __$$ExerciseDataDto_NextWordMcqImplCopyWithImpl<$Res>;
  @useResult
  $Res call({
    String nodeId,
    int contextPosition,
    List<String> distractorNodeIds,
  });
}

/// @nodoc
class __$$ExerciseDataDto_NextWordMcqImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_NextWordMcqImpl>
    implements _$$ExerciseDataDto_NextWordMcqImplCopyWith<$Res> {
  __$$ExerciseDataDto_NextWordMcqImplCopyWithImpl(
    _$ExerciseDataDto_NextWordMcqImpl _value,
    $Res Function(_$ExerciseDataDto_NextWordMcqImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? contextPosition = null,
    Object? distractorNodeIds = null,
  }) {
    return _then(
      _$ExerciseDataDto_NextWordMcqImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        contextPosition: null == contextPosition
            ? _value.contextPosition
            : contextPosition // ignore: cast_nullable_to_non_nullable
                  as int,
        distractorNodeIds: null == distractorNodeIds
            ? _value._distractorNodeIds
            : distractorNodeIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_NextWordMcqImpl extends ExerciseDataDto_NextWordMcq {
  const _$ExerciseDataDto_NextWordMcqImpl({
    required this.nodeId,
    required this.contextPosition,
    required final List<String> distractorNodeIds,
  }) : _distractorNodeIds = distractorNodeIds,
       super._();

  @override
  final String nodeId;
  @override
  final int contextPosition;
  final List<String> _distractorNodeIds;
  @override
  List<String> get distractorNodeIds {
    if (_distractorNodeIds is EqualUnmodifiableListView)
      return _distractorNodeIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_distractorNodeIds);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.nextWordMcq(nodeId: $nodeId, contextPosition: $contextPosition, distractorNodeIds: $distractorNodeIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_NextWordMcqImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.contextPosition, contextPosition) ||
                other.contextPosition == contextPosition) &&
            const DeepCollectionEquality().equals(
              other._distractorNodeIds,
              _distractorNodeIds,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    contextPosition,
    const DeepCollectionEquality().hash(_distractorNodeIds),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_NextWordMcqImplCopyWith<_$ExerciseDataDto_NextWordMcqImpl>
  get copyWith =>
      __$$ExerciseDataDto_NextWordMcqImplCopyWithImpl<
        _$ExerciseDataDto_NextWordMcqImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return nextWordMcq(nodeId, contextPosition, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return nextWordMcq?.call(nodeId, contextPosition, distractorNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (nextWordMcq != null) {
      return nextWordMcq(nodeId, contextPosition, distractorNodeIds);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return nextWordMcq(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return nextWordMcq?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (nextWordMcq != null) {
      return nextWordMcq(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_NextWordMcq extends ExerciseDataDto {
  const factory ExerciseDataDto_NextWordMcq({
    required final String nodeId,
    required final int contextPosition,
    required final List<String> distractorNodeIds,
  }) = _$ExerciseDataDto_NextWordMcqImpl;
  const ExerciseDataDto_NextWordMcq._() : super._();

  String get nodeId;
  int get contextPosition;
  List<String> get distractorNodeIds;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_NextWordMcqImplCopyWith<_$ExerciseDataDto_NextWordMcqImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_FullVerseInputImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_FullVerseInputImplCopyWith(
    _$ExerciseDataDto_FullVerseInputImpl value,
    $Res Function(_$ExerciseDataDto_FullVerseInputImpl) then,
  ) = __$$ExerciseDataDto_FullVerseInputImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId});
}

/// @nodoc
class __$$ExerciseDataDto_FullVerseInputImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_FullVerseInputImpl
        >
    implements _$$ExerciseDataDto_FullVerseInputImplCopyWith<$Res> {
  __$$ExerciseDataDto_FullVerseInputImplCopyWithImpl(
    _$ExerciseDataDto_FullVerseInputImpl _value,
    $Res Function(_$ExerciseDataDto_FullVerseInputImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null}) {
    return _then(
      _$ExerciseDataDto_FullVerseInputImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_FullVerseInputImpl
    extends ExerciseDataDto_FullVerseInput {
  const _$ExerciseDataDto_FullVerseInputImpl({required this.nodeId})
    : super._();

  @override
  final String nodeId;

  @override
  String toString() {
    return 'ExerciseDataDto.fullVerseInput(nodeId: $nodeId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_FullVerseInputImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_FullVerseInputImplCopyWith<
    _$ExerciseDataDto_FullVerseInputImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_FullVerseInputImplCopyWithImpl<
        _$ExerciseDataDto_FullVerseInputImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return fullVerseInput(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return fullVerseInput?.call(nodeId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (fullVerseInput != null) {
      return fullVerseInput(nodeId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return fullVerseInput(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return fullVerseInput?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (fullVerseInput != null) {
      return fullVerseInput(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_FullVerseInput extends ExerciseDataDto {
  const factory ExerciseDataDto_FullVerseInput({required final String nodeId}) =
      _$ExerciseDataDto_FullVerseInputImpl;
  const ExerciseDataDto_FullVerseInput._() : super._();

  String get nodeId;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_FullVerseInputImplCopyWith<
    _$ExerciseDataDto_FullVerseInputImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_AyahChainImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_AyahChainImplCopyWith(
    _$ExerciseDataDto_AyahChainImpl value,
    $Res Function(_$ExerciseDataDto_AyahChainImpl) then,
  ) = __$$ExerciseDataDto_AyahChainImplCopyWithImpl<$Res>;
  @useResult
  $Res call({
    String nodeId,
    List<String> verseKeys,
    BigInt currentIndex,
    BigInt completedCount,
  });
}

/// @nodoc
class __$$ExerciseDataDto_AyahChainImplCopyWithImpl<$Res>
    extends _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_AyahChainImpl>
    implements _$$ExerciseDataDto_AyahChainImplCopyWith<$Res> {
  __$$ExerciseDataDto_AyahChainImplCopyWithImpl(
    _$ExerciseDataDto_AyahChainImpl _value,
    $Res Function(_$ExerciseDataDto_AyahChainImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? verseKeys = null,
    Object? currentIndex = null,
    Object? completedCount = null,
  }) {
    return _then(
      _$ExerciseDataDto_AyahChainImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        verseKeys: null == verseKeys
            ? _value._verseKeys
            : verseKeys // ignore: cast_nullable_to_non_nullable
                  as List<String>,
        currentIndex: null == currentIndex
            ? _value.currentIndex
            : currentIndex // ignore: cast_nullable_to_non_nullable
                  as BigInt,
        completedCount: null == completedCount
            ? _value.completedCount
            : completedCount // ignore: cast_nullable_to_non_nullable
                  as BigInt,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_AyahChainImpl extends ExerciseDataDto_AyahChain {
  const _$ExerciseDataDto_AyahChainImpl({
    required this.nodeId,
    required final List<String> verseKeys,
    required this.currentIndex,
    required this.completedCount,
  }) : _verseKeys = verseKeys,
       super._();

  @override
  final String nodeId;
  final List<String> _verseKeys;
  @override
  List<String> get verseKeys {
    if (_verseKeys is EqualUnmodifiableListView) return _verseKeys;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_verseKeys);
  }

  @override
  final BigInt currentIndex;
  @override
  final BigInt completedCount;

  @override
  String toString() {
    return 'ExerciseDataDto.ayahChain(nodeId: $nodeId, verseKeys: $verseKeys, currentIndex: $currentIndex, completedCount: $completedCount)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_AyahChainImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            const DeepCollectionEquality().equals(
              other._verseKeys,
              _verseKeys,
            ) &&
            (identical(other.currentIndex, currentIndex) ||
                other.currentIndex == currentIndex) &&
            (identical(other.completedCount, completedCount) ||
                other.completedCount == completedCount));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    const DeepCollectionEquality().hash(_verseKeys),
    currentIndex,
    completedCount,
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_AyahChainImplCopyWith<_$ExerciseDataDto_AyahChainImpl>
  get copyWith =>
      __$$ExerciseDataDto_AyahChainImplCopyWithImpl<
        _$ExerciseDataDto_AyahChainImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return ayahChain(nodeId, verseKeys, currentIndex, completedCount);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return ayahChain?.call(nodeId, verseKeys, currentIndex, completedCount);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (ayahChain != null) {
      return ayahChain(nodeId, verseKeys, currentIndex, completedCount);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return ayahChain(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return ayahChain?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (ayahChain != null) {
      return ayahChain(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_AyahChain extends ExerciseDataDto {
  const factory ExerciseDataDto_AyahChain({
    required final String nodeId,
    required final List<String> verseKeys,
    required final BigInt currentIndex,
    required final BigInt completedCount,
  }) = _$ExerciseDataDto_AyahChainImpl;
  const ExerciseDataDto_AyahChain._() : super._();

  String get nodeId;
  List<String> get verseKeys;
  BigInt get currentIndex;
  BigInt get completedCount;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_AyahChainImplCopyWith<_$ExerciseDataDto_AyahChainImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_FindMistakeImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_FindMistakeImplCopyWith(
    _$ExerciseDataDto_FindMistakeImpl value,
    $Res Function(_$ExerciseDataDto_FindMistakeImpl) then,
  ) = __$$ExerciseDataDto_FindMistakeImplCopyWithImpl<$Res>;
  @useResult
  $Res call({
    String nodeId,
    int mistakePosition,
    String correctWordNodeId,
    String incorrectWordNodeId,
  });
}

/// @nodoc
class __$$ExerciseDataDto_FindMistakeImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_FindMistakeImpl>
    implements _$$ExerciseDataDto_FindMistakeImplCopyWith<$Res> {
  __$$ExerciseDataDto_FindMistakeImplCopyWithImpl(
    _$ExerciseDataDto_FindMistakeImpl _value,
    $Res Function(_$ExerciseDataDto_FindMistakeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? mistakePosition = null,
    Object? correctWordNodeId = null,
    Object? incorrectWordNodeId = null,
  }) {
    return _then(
      _$ExerciseDataDto_FindMistakeImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        mistakePosition: null == mistakePosition
            ? _value.mistakePosition
            : mistakePosition // ignore: cast_nullable_to_non_nullable
                  as int,
        correctWordNodeId: null == correctWordNodeId
            ? _value.correctWordNodeId
            : correctWordNodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        incorrectWordNodeId: null == incorrectWordNodeId
            ? _value.incorrectWordNodeId
            : incorrectWordNodeId // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_FindMistakeImpl extends ExerciseDataDto_FindMistake {
  const _$ExerciseDataDto_FindMistakeImpl({
    required this.nodeId,
    required this.mistakePosition,
    required this.correctWordNodeId,
    required this.incorrectWordNodeId,
  }) : super._();

  @override
  final String nodeId;
  @override
  final int mistakePosition;
  @override
  final String correctWordNodeId;
  @override
  final String incorrectWordNodeId;

  @override
  String toString() {
    return 'ExerciseDataDto.findMistake(nodeId: $nodeId, mistakePosition: $mistakePosition, correctWordNodeId: $correctWordNodeId, incorrectWordNodeId: $incorrectWordNodeId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_FindMistakeImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.mistakePosition, mistakePosition) ||
                other.mistakePosition == mistakePosition) &&
            (identical(other.correctWordNodeId, correctWordNodeId) ||
                other.correctWordNodeId == correctWordNodeId) &&
            (identical(other.incorrectWordNodeId, incorrectWordNodeId) ||
                other.incorrectWordNodeId == incorrectWordNodeId));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    mistakePosition,
    correctWordNodeId,
    incorrectWordNodeId,
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_FindMistakeImplCopyWith<_$ExerciseDataDto_FindMistakeImpl>
  get copyWith =>
      __$$ExerciseDataDto_FindMistakeImplCopyWithImpl<
        _$ExerciseDataDto_FindMistakeImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return findMistake(
      nodeId,
      mistakePosition,
      correctWordNodeId,
      incorrectWordNodeId,
    );
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return findMistake?.call(
      nodeId,
      mistakePosition,
      correctWordNodeId,
      incorrectWordNodeId,
    );
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (findMistake != null) {
      return findMistake(
        nodeId,
        mistakePosition,
        correctWordNodeId,
        incorrectWordNodeId,
      );
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return findMistake(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return findMistake?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (findMistake != null) {
      return findMistake(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_FindMistake extends ExerciseDataDto {
  const factory ExerciseDataDto_FindMistake({
    required final String nodeId,
    required final int mistakePosition,
    required final String correctWordNodeId,
    required final String incorrectWordNodeId,
  }) = _$ExerciseDataDto_FindMistakeImpl;
  const ExerciseDataDto_FindMistake._() : super._();

  String get nodeId;
  int get mistakePosition;
  String get correctWordNodeId;
  String get incorrectWordNodeId;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_FindMistakeImplCopyWith<_$ExerciseDataDto_FindMistakeImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_AyahSequenceImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_AyahSequenceImplCopyWith(
    _$ExerciseDataDto_AyahSequenceImpl value,
    $Res Function(_$ExerciseDataDto_AyahSequenceImpl) then,
  ) = __$$ExerciseDataDto_AyahSequenceImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, List<String> correctSequence});
}

/// @nodoc
class __$$ExerciseDataDto_AyahSequenceImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_AyahSequenceImpl>
    implements _$$ExerciseDataDto_AyahSequenceImplCopyWith<$Res> {
  __$$ExerciseDataDto_AyahSequenceImplCopyWithImpl(
    _$ExerciseDataDto_AyahSequenceImpl _value,
    $Res Function(_$ExerciseDataDto_AyahSequenceImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? correctSequence = null}) {
    return _then(
      _$ExerciseDataDto_AyahSequenceImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        correctSequence: null == correctSequence
            ? _value._correctSequence
            : correctSequence // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_AyahSequenceImpl extends ExerciseDataDto_AyahSequence {
  const _$ExerciseDataDto_AyahSequenceImpl({
    required this.nodeId,
    required final List<String> correctSequence,
  }) : _correctSequence = correctSequence,
       super._();

  @override
  final String nodeId;
  final List<String> _correctSequence;
  @override
  List<String> get correctSequence {
    if (_correctSequence is EqualUnmodifiableListView) return _correctSequence;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_correctSequence);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.ayahSequence(nodeId: $nodeId, correctSequence: $correctSequence)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_AyahSequenceImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            const DeepCollectionEquality().equals(
              other._correctSequence,
              _correctSequence,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    const DeepCollectionEquality().hash(_correctSequence),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_AyahSequenceImplCopyWith<
    _$ExerciseDataDto_AyahSequenceImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_AyahSequenceImplCopyWithImpl<
        _$ExerciseDataDto_AyahSequenceImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return ayahSequence(nodeId, correctSequence);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return ayahSequence?.call(nodeId, correctSequence);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (ayahSequence != null) {
      return ayahSequence(nodeId, correctSequence);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return ayahSequence(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return ayahSequence?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (ayahSequence != null) {
      return ayahSequence(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_AyahSequence extends ExerciseDataDto {
  const factory ExerciseDataDto_AyahSequence({
    required final String nodeId,
    required final List<String> correctSequence,
  }) = _$ExerciseDataDto_AyahSequenceImpl;
  const ExerciseDataDto_AyahSequence._() : super._();

  String get nodeId;
  List<String> get correctSequence;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_AyahSequenceImplCopyWith<
    _$ExerciseDataDto_AyahSequenceImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_IdentifyRootImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_IdentifyRootImplCopyWith(
    _$ExerciseDataDto_IdentifyRootImpl value,
    $Res Function(_$ExerciseDataDto_IdentifyRootImpl) then,
  ) = __$$ExerciseDataDto_IdentifyRootImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, String root});
}

/// @nodoc
class __$$ExerciseDataDto_IdentifyRootImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_IdentifyRootImpl>
    implements _$$ExerciseDataDto_IdentifyRootImplCopyWith<$Res> {
  __$$ExerciseDataDto_IdentifyRootImplCopyWithImpl(
    _$ExerciseDataDto_IdentifyRootImpl _value,
    $Res Function(_$ExerciseDataDto_IdentifyRootImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? root = null}) {
    return _then(
      _$ExerciseDataDto_IdentifyRootImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        root: null == root
            ? _value.root
            : root // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_IdentifyRootImpl extends ExerciseDataDto_IdentifyRoot {
  const _$ExerciseDataDto_IdentifyRootImpl({
    required this.nodeId,
    required this.root,
  }) : super._();

  @override
  final String nodeId;
  @override
  final String root;

  @override
  String toString() {
    return 'ExerciseDataDto.identifyRoot(nodeId: $nodeId, root: $root)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_IdentifyRootImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.root, root) || other.root == root));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, root);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_IdentifyRootImplCopyWith<
    _$ExerciseDataDto_IdentifyRootImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_IdentifyRootImplCopyWithImpl<
        _$ExerciseDataDto_IdentifyRootImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return identifyRoot(nodeId, root);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return identifyRoot?.call(nodeId, root);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (identifyRoot != null) {
      return identifyRoot(nodeId, root);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return identifyRoot(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return identifyRoot?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (identifyRoot != null) {
      return identifyRoot(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_IdentifyRoot extends ExerciseDataDto {
  const factory ExerciseDataDto_IdentifyRoot({
    required final String nodeId,
    required final String root,
  }) = _$ExerciseDataDto_IdentifyRootImpl;
  const ExerciseDataDto_IdentifyRoot._() : super._();

  String get nodeId;
  String get root;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_IdentifyRootImplCopyWith<
    _$ExerciseDataDto_IdentifyRootImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_ReverseClozeImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_ReverseClozeImplCopyWith(
    _$ExerciseDataDto_ReverseClozeImpl value,
    $Res Function(_$ExerciseDataDto_ReverseClozeImpl) then,
  ) = __$$ExerciseDataDto_ReverseClozeImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, int blankPosition});
}

/// @nodoc
class __$$ExerciseDataDto_ReverseClozeImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_ReverseClozeImpl>
    implements _$$ExerciseDataDto_ReverseClozeImplCopyWith<$Res> {
  __$$ExerciseDataDto_ReverseClozeImplCopyWithImpl(
    _$ExerciseDataDto_ReverseClozeImpl _value,
    $Res Function(_$ExerciseDataDto_ReverseClozeImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? blankPosition = null}) {
    return _then(
      _$ExerciseDataDto_ReverseClozeImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        blankPosition: null == blankPosition
            ? _value.blankPosition
            : blankPosition // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_ReverseClozeImpl extends ExerciseDataDto_ReverseCloze {
  const _$ExerciseDataDto_ReverseClozeImpl({
    required this.nodeId,
    required this.blankPosition,
  }) : super._();

  @override
  final String nodeId;
  @override
  final int blankPosition;

  @override
  String toString() {
    return 'ExerciseDataDto.reverseCloze(nodeId: $nodeId, blankPosition: $blankPosition)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_ReverseClozeImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.blankPosition, blankPosition) ||
                other.blankPosition == blankPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, blankPosition);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_ReverseClozeImplCopyWith<
    _$ExerciseDataDto_ReverseClozeImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_ReverseClozeImplCopyWithImpl<
        _$ExerciseDataDto_ReverseClozeImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return reverseCloze(nodeId, blankPosition);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return reverseCloze?.call(nodeId, blankPosition);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (reverseCloze != null) {
      return reverseCloze(nodeId, blankPosition);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return reverseCloze(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return reverseCloze?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (reverseCloze != null) {
      return reverseCloze(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_ReverseCloze extends ExerciseDataDto {
  const factory ExerciseDataDto_ReverseCloze({
    required final String nodeId,
    required final int blankPosition,
  }) = _$ExerciseDataDto_ReverseClozeImpl;
  const ExerciseDataDto_ReverseCloze._() : super._();

  String get nodeId;
  int get blankPosition;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_ReverseClozeImplCopyWith<
    _$ExerciseDataDto_ReverseClozeImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_TranslatePhraseImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_TranslatePhraseImplCopyWith(
    _$ExerciseDataDto_TranslatePhraseImpl value,
    $Res Function(_$ExerciseDataDto_TranslatePhraseImpl) then,
  ) = __$$ExerciseDataDto_TranslatePhraseImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, int translatorId});
}

/// @nodoc
class __$$ExerciseDataDto_TranslatePhraseImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_TranslatePhraseImpl
        >
    implements _$$ExerciseDataDto_TranslatePhraseImplCopyWith<$Res> {
  __$$ExerciseDataDto_TranslatePhraseImplCopyWithImpl(
    _$ExerciseDataDto_TranslatePhraseImpl _value,
    $Res Function(_$ExerciseDataDto_TranslatePhraseImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? nodeId = null, Object? translatorId = null}) {
    return _then(
      _$ExerciseDataDto_TranslatePhraseImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        translatorId: null == translatorId
            ? _value.translatorId
            : translatorId // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_TranslatePhraseImpl
    extends ExerciseDataDto_TranslatePhrase {
  const _$ExerciseDataDto_TranslatePhraseImpl({
    required this.nodeId,
    required this.translatorId,
  }) : super._();

  @override
  final String nodeId;
  @override
  final int translatorId;

  @override
  String toString() {
    return 'ExerciseDataDto.translatePhrase(nodeId: $nodeId, translatorId: $translatorId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_TranslatePhraseImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.translatorId, translatorId) ||
                other.translatorId == translatorId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, nodeId, translatorId);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_TranslatePhraseImplCopyWith<
    _$ExerciseDataDto_TranslatePhraseImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_TranslatePhraseImplCopyWithImpl<
        _$ExerciseDataDto_TranslatePhraseImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return translatePhrase(nodeId, translatorId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return translatePhrase?.call(nodeId, translatorId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (translatePhrase != null) {
      return translatePhrase(nodeId, translatorId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return translatePhrase(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return translatePhrase?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (translatePhrase != null) {
      return translatePhrase(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_TranslatePhrase extends ExerciseDataDto {
  const factory ExerciseDataDto_TranslatePhrase({
    required final String nodeId,
    required final int translatorId,
  }) = _$ExerciseDataDto_TranslatePhraseImpl;
  const ExerciseDataDto_TranslatePhrase._() : super._();

  String get nodeId;
  int get translatorId;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_TranslatePhraseImplCopyWith<
    _$ExerciseDataDto_TranslatePhraseImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_PosTaggingImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_PosTaggingImplCopyWith(
    _$ExerciseDataDto_PosTaggingImpl value,
    $Res Function(_$ExerciseDataDto_PosTaggingImpl) then,
  ) = __$$ExerciseDataDto_PosTaggingImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String nodeId, String correctPos, List<String> options});
}

/// @nodoc
class __$$ExerciseDataDto_PosTaggingImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_PosTaggingImpl>
    implements _$$ExerciseDataDto_PosTaggingImplCopyWith<$Res> {
  __$$ExerciseDataDto_PosTaggingImplCopyWithImpl(
    _$ExerciseDataDto_PosTaggingImpl _value,
    $Res Function(_$ExerciseDataDto_PosTaggingImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? correctPos = null,
    Object? options = null,
  }) {
    return _then(
      _$ExerciseDataDto_PosTaggingImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        correctPos: null == correctPos
            ? _value.correctPos
            : correctPos // ignore: cast_nullable_to_non_nullable
                  as String,
        options: null == options
            ? _value._options
            : options // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_PosTaggingImpl extends ExerciseDataDto_PosTagging {
  const _$ExerciseDataDto_PosTaggingImpl({
    required this.nodeId,
    required this.correctPos,
    required final List<String> options,
  }) : _options = options,
       super._();

  @override
  final String nodeId;
  @override
  final String correctPos;
  final List<String> _options;
  @override
  List<String> get options {
    if (_options is EqualUnmodifiableListView) return _options;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_options);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.posTagging(nodeId: $nodeId, correctPos: $correctPos, options: $options)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_PosTaggingImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            (identical(other.correctPos, correctPos) ||
                other.correctPos == correctPos) &&
            const DeepCollectionEquality().equals(other._options, _options));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    correctPos,
    const DeepCollectionEquality().hash(_options),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_PosTaggingImplCopyWith<_$ExerciseDataDto_PosTaggingImpl>
  get copyWith =>
      __$$ExerciseDataDto_PosTaggingImplCopyWithImpl<
        _$ExerciseDataDto_PosTaggingImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return posTagging(nodeId, correctPos, options);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return posTagging?.call(nodeId, correctPos, options);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (posTagging != null) {
      return posTagging(nodeId, correctPos, options);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return posTagging(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return posTagging?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (posTagging != null) {
      return posTagging(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_PosTagging extends ExerciseDataDto {
  const factory ExerciseDataDto_PosTagging({
    required final String nodeId,
    required final String correctPos,
    required final List<String> options,
  }) = _$ExerciseDataDto_PosTaggingImpl;
  const ExerciseDataDto_PosTagging._() : super._();

  String get nodeId;
  String get correctPos;
  List<String> get options;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_PosTaggingImplCopyWith<_$ExerciseDataDto_PosTaggingImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_CrossVerseConnectionImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_CrossVerseConnectionImplCopyWith(
    _$ExerciseDataDto_CrossVerseConnectionImpl value,
    $Res Function(_$ExerciseDataDto_CrossVerseConnectionImpl) then,
  ) = __$$ExerciseDataDto_CrossVerseConnectionImplCopyWithImpl<$Res>;
  @useResult
  $Res call({
    String nodeId,
    List<String> relatedVerseIds,
    String connectionTheme,
  });
}

/// @nodoc
class __$$ExerciseDataDto_CrossVerseConnectionImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<
          $Res,
          _$ExerciseDataDto_CrossVerseConnectionImpl
        >
    implements _$$ExerciseDataDto_CrossVerseConnectionImplCopyWith<$Res> {
  __$$ExerciseDataDto_CrossVerseConnectionImplCopyWithImpl(
    _$ExerciseDataDto_CrossVerseConnectionImpl _value,
    $Res Function(_$ExerciseDataDto_CrossVerseConnectionImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? nodeId = null,
    Object? relatedVerseIds = null,
    Object? connectionTheme = null,
  }) {
    return _then(
      _$ExerciseDataDto_CrossVerseConnectionImpl(
        nodeId: null == nodeId
            ? _value.nodeId
            : nodeId // ignore: cast_nullable_to_non_nullable
                  as String,
        relatedVerseIds: null == relatedVerseIds
            ? _value._relatedVerseIds
            : relatedVerseIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
        connectionTheme: null == connectionTheme
            ? _value.connectionTheme
            : connectionTheme // ignore: cast_nullable_to_non_nullable
                  as String,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_CrossVerseConnectionImpl
    extends ExerciseDataDto_CrossVerseConnection {
  const _$ExerciseDataDto_CrossVerseConnectionImpl({
    required this.nodeId,
    required final List<String> relatedVerseIds,
    required this.connectionTheme,
  }) : _relatedVerseIds = relatedVerseIds,
       super._();

  @override
  final String nodeId;
  final List<String> _relatedVerseIds;
  @override
  List<String> get relatedVerseIds {
    if (_relatedVerseIds is EqualUnmodifiableListView) return _relatedVerseIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_relatedVerseIds);
  }

  @override
  final String connectionTheme;

  @override
  String toString() {
    return 'ExerciseDataDto.crossVerseConnection(nodeId: $nodeId, relatedVerseIds: $relatedVerseIds, connectionTheme: $connectionTheme)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_CrossVerseConnectionImpl &&
            (identical(other.nodeId, nodeId) || other.nodeId == nodeId) &&
            const DeepCollectionEquality().equals(
              other._relatedVerseIds,
              _relatedVerseIds,
            ) &&
            (identical(other.connectionTheme, connectionTheme) ||
                other.connectionTheme == connectionTheme));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    nodeId,
    const DeepCollectionEquality().hash(_relatedVerseIds),
    connectionTheme,
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_CrossVerseConnectionImplCopyWith<
    _$ExerciseDataDto_CrossVerseConnectionImpl
  >
  get copyWith =>
      __$$ExerciseDataDto_CrossVerseConnectionImplCopyWithImpl<
        _$ExerciseDataDto_CrossVerseConnectionImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return crossVerseConnection(nodeId, relatedVerseIds, connectionTheme);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return crossVerseConnection?.call(nodeId, relatedVerseIds, connectionTheme);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (crossVerseConnection != null) {
      return crossVerseConnection(nodeId, relatedVerseIds, connectionTheme);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return crossVerseConnection(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return crossVerseConnection?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (crossVerseConnection != null) {
      return crossVerseConnection(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_CrossVerseConnection extends ExerciseDataDto {
  const factory ExerciseDataDto_CrossVerseConnection({
    required final String nodeId,
    required final List<String> relatedVerseIds,
    required final String connectionTheme,
  }) = _$ExerciseDataDto_CrossVerseConnectionImpl;
  const ExerciseDataDto_CrossVerseConnection._() : super._();

  String get nodeId;
  List<String> get relatedVerseIds;
  String get connectionTheme;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_CrossVerseConnectionImplCopyWith<
    _$ExerciseDataDto_CrossVerseConnectionImpl
  >
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ExerciseDataDto_EchoRecallImplCopyWith<$Res> {
  factory _$$ExerciseDataDto_EchoRecallImplCopyWith(
    _$ExerciseDataDto_EchoRecallImpl value,
    $Res Function(_$ExerciseDataDto_EchoRecallImpl) then,
  ) = __$$ExerciseDataDto_EchoRecallImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String userId, List<String> ayahNodeIds});
}

/// @nodoc
class __$$ExerciseDataDto_EchoRecallImplCopyWithImpl<$Res>
    extends
        _$ExerciseDataDtoCopyWithImpl<$Res, _$ExerciseDataDto_EchoRecallImpl>
    implements _$$ExerciseDataDto_EchoRecallImplCopyWith<$Res> {
  __$$ExerciseDataDto_EchoRecallImplCopyWithImpl(
    _$ExerciseDataDto_EchoRecallImpl _value,
    $Res Function(_$ExerciseDataDto_EchoRecallImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? userId = null, Object? ayahNodeIds = null}) {
    return _then(
      _$ExerciseDataDto_EchoRecallImpl(
        userId: null == userId
            ? _value.userId
            : userId // ignore: cast_nullable_to_non_nullable
                  as String,
        ayahNodeIds: null == ayahNodeIds
            ? _value._ayahNodeIds
            : ayahNodeIds // ignore: cast_nullable_to_non_nullable
                  as List<String>,
      ),
    );
  }
}

/// @nodoc

class _$ExerciseDataDto_EchoRecallImpl extends ExerciseDataDto_EchoRecall {
  const _$ExerciseDataDto_EchoRecallImpl({
    required this.userId,
    required final List<String> ayahNodeIds,
  }) : _ayahNodeIds = ayahNodeIds,
       super._();

  /// User ID for session tracking
  @override
  final String userId;

  /// List of ayah node IDs to practice (e.g., ["VERSE:1:1", "VERSE:1:2"])
  final List<String> _ayahNodeIds;

  /// List of ayah node IDs to practice (e.g., ["VERSE:1:1", "VERSE:1:2"])
  @override
  List<String> get ayahNodeIds {
    if (_ayahNodeIds is EqualUnmodifiableListView) return _ayahNodeIds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_ayahNodeIds);
  }

  @override
  String toString() {
    return 'ExerciseDataDto.echoRecall(userId: $userId, ayahNodeIds: $ayahNodeIds)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ExerciseDataDto_EchoRecallImpl &&
            (identical(other.userId, userId) || other.userId == userId) &&
            const DeepCollectionEquality().equals(
              other._ayahNodeIds,
              _ayahNodeIds,
            ));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    userId,
    const DeepCollectionEquality().hash(_ayahNodeIds),
  );

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ExerciseDataDto_EchoRecallImplCopyWith<_$ExerciseDataDto_EchoRecallImpl>
  get copyWith =>
      __$$ExerciseDataDto_EchoRecallImplCopyWithImpl<
        _$ExerciseDataDto_EchoRecallImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String nodeId) memorization,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqArToEn,
    required TResult Function(String nodeId, List<String> distractorNodeIds)
    mcqEnToAr,
    required TResult Function(String nodeId) translation,
    required TResult Function(String nodeId, String verseKey)
    contextualTranslation,
    required TResult Function(String nodeId, int blankPosition) clozeDeletion,
    required TResult Function(String nodeId, int wordPosition) firstLetterHint,
    required TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )
    missingWordMcq,
    required TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )
    nextWordMcq,
    required TResult Function(String nodeId) fullVerseInput,
    required TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )
    ayahChain,
    required TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )
    findMistake,
    required TResult Function(String nodeId, List<String> correctSequence)
    ayahSequence,
    required TResult Function(String nodeId, String root) identifyRoot,
    required TResult Function(String nodeId, int blankPosition) reverseCloze,
    required TResult Function(String nodeId, int translatorId) translatePhrase,
    required TResult Function(
      String nodeId,
      String correctPos,
      List<String> options,
    )
    posTagging,
    required TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )
    crossVerseConnection,
    required TResult Function(String userId, List<String> ayahNodeIds)
    echoRecall,
  }) {
    return echoRecall(userId, ayahNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String nodeId)? memorization,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult? Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult? Function(String nodeId)? translation,
    TResult? Function(String nodeId, String verseKey)? contextualTranslation,
    TResult? Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult? Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult? Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult? Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult? Function(String nodeId)? fullVerseInput,
    TResult? Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult? Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult? Function(String nodeId, List<String> correctSequence)?
    ayahSequence,
    TResult? Function(String nodeId, String root)? identifyRoot,
    TResult? Function(String nodeId, int blankPosition)? reverseCloze,
    TResult? Function(String nodeId, int translatorId)? translatePhrase,
    TResult? Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult? Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult? Function(String userId, List<String> ayahNodeIds)? echoRecall,
  }) {
    return echoRecall?.call(userId, ayahNodeIds);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String nodeId)? memorization,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqArToEn,
    TResult Function(String nodeId, List<String> distractorNodeIds)? mcqEnToAr,
    TResult Function(String nodeId)? translation,
    TResult Function(String nodeId, String verseKey)? contextualTranslation,
    TResult Function(String nodeId, int blankPosition)? clozeDeletion,
    TResult Function(String nodeId, int wordPosition)? firstLetterHint,
    TResult Function(
      String nodeId,
      int blankPosition,
      List<String> distractorNodeIds,
    )?
    missingWordMcq,
    TResult Function(
      String nodeId,
      int contextPosition,
      List<String> distractorNodeIds,
    )?
    nextWordMcq,
    TResult Function(String nodeId)? fullVerseInput,
    TResult Function(
      String nodeId,
      List<String> verseKeys,
      BigInt currentIndex,
      BigInt completedCount,
    )?
    ayahChain,
    TResult Function(
      String nodeId,
      int mistakePosition,
      String correctWordNodeId,
      String incorrectWordNodeId,
    )?
    findMistake,
    TResult Function(String nodeId, List<String> correctSequence)? ayahSequence,
    TResult Function(String nodeId, String root)? identifyRoot,
    TResult Function(String nodeId, int blankPosition)? reverseCloze,
    TResult Function(String nodeId, int translatorId)? translatePhrase,
    TResult Function(String nodeId, String correctPos, List<String> options)?
    posTagging,
    TResult Function(
      String nodeId,
      List<String> relatedVerseIds,
      String connectionTheme,
    )?
    crossVerseConnection,
    TResult Function(String userId, List<String> ayahNodeIds)? echoRecall,
    required TResult orElse(),
  }) {
    if (echoRecall != null) {
      return echoRecall(userId, ayahNodeIds);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ExerciseDataDto_Memorization value) memorization,
    required TResult Function(ExerciseDataDto_McqArToEn value) mcqArToEn,
    required TResult Function(ExerciseDataDto_McqEnToAr value) mcqEnToAr,
    required TResult Function(ExerciseDataDto_Translation value) translation,
    required TResult Function(ExerciseDataDto_ContextualTranslation value)
    contextualTranslation,
    required TResult Function(ExerciseDataDto_ClozeDeletion value)
    clozeDeletion,
    required TResult Function(ExerciseDataDto_FirstLetterHint value)
    firstLetterHint,
    required TResult Function(ExerciseDataDto_MissingWordMcq value)
    missingWordMcq,
    required TResult Function(ExerciseDataDto_NextWordMcq value) nextWordMcq,
    required TResult Function(ExerciseDataDto_FullVerseInput value)
    fullVerseInput,
    required TResult Function(ExerciseDataDto_AyahChain value) ayahChain,
    required TResult Function(ExerciseDataDto_FindMistake value) findMistake,
    required TResult Function(ExerciseDataDto_AyahSequence value) ayahSequence,
    required TResult Function(ExerciseDataDto_IdentifyRoot value) identifyRoot,
    required TResult Function(ExerciseDataDto_ReverseCloze value) reverseCloze,
    required TResult Function(ExerciseDataDto_TranslatePhrase value)
    translatePhrase,
    required TResult Function(ExerciseDataDto_PosTagging value) posTagging,
    required TResult Function(ExerciseDataDto_CrossVerseConnection value)
    crossVerseConnection,
    required TResult Function(ExerciseDataDto_EchoRecall value) echoRecall,
  }) {
    return echoRecall(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ExerciseDataDto_Memorization value)? memorization,
    TResult? Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult? Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult? Function(ExerciseDataDto_Translation value)? translation,
    TResult? Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult? Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult? Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult? Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult? Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult? Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult? Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult? Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult? Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult? Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult? Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult? Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult? Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult? Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult? Function(ExerciseDataDto_EchoRecall value)? echoRecall,
  }) {
    return echoRecall?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ExerciseDataDto_Memorization value)? memorization,
    TResult Function(ExerciseDataDto_McqArToEn value)? mcqArToEn,
    TResult Function(ExerciseDataDto_McqEnToAr value)? mcqEnToAr,
    TResult Function(ExerciseDataDto_Translation value)? translation,
    TResult Function(ExerciseDataDto_ContextualTranslation value)?
    contextualTranslation,
    TResult Function(ExerciseDataDto_ClozeDeletion value)? clozeDeletion,
    TResult Function(ExerciseDataDto_FirstLetterHint value)? firstLetterHint,
    TResult Function(ExerciseDataDto_MissingWordMcq value)? missingWordMcq,
    TResult Function(ExerciseDataDto_NextWordMcq value)? nextWordMcq,
    TResult Function(ExerciseDataDto_FullVerseInput value)? fullVerseInput,
    TResult Function(ExerciseDataDto_AyahChain value)? ayahChain,
    TResult Function(ExerciseDataDto_FindMistake value)? findMistake,
    TResult Function(ExerciseDataDto_AyahSequence value)? ayahSequence,
    TResult Function(ExerciseDataDto_IdentifyRoot value)? identifyRoot,
    TResult Function(ExerciseDataDto_ReverseCloze value)? reverseCloze,
    TResult Function(ExerciseDataDto_TranslatePhrase value)? translatePhrase,
    TResult Function(ExerciseDataDto_PosTagging value)? posTagging,
    TResult Function(ExerciseDataDto_CrossVerseConnection value)?
    crossVerseConnection,
    TResult Function(ExerciseDataDto_EchoRecall value)? echoRecall,
    required TResult orElse(),
  }) {
    if (echoRecall != null) {
      return echoRecall(this);
    }
    return orElse();
  }
}

abstract class ExerciseDataDto_EchoRecall extends ExerciseDataDto {
  const factory ExerciseDataDto_EchoRecall({
    required final String userId,
    required final List<String> ayahNodeIds,
  }) = _$ExerciseDataDto_EchoRecallImpl;
  const ExerciseDataDto_EchoRecall._() : super._();

  /// User ID for session tracking
  String get userId;

  /// List of ayah node IDs to practice (e.g., ["VERSE:1:1", "VERSE:1:2"])
  List<String> get ayahNodeIds;

  /// Create a copy of ExerciseDataDto
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ExerciseDataDto_EchoRecallImplCopyWith<_$ExerciseDataDto_EchoRecallImpl>
  get copyWith => throw _privateConstructorUsedError;
}
