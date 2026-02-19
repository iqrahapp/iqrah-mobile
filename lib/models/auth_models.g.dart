// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'auth_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$AuthResponseImpl _$$AuthResponseImplFromJson(Map<String, dynamic> json) =>
    _$AuthResponseImpl(
      accessToken: json['access_token'] as String,
      userId: json['user_id'] as String,
      expiresIn: (json['expires_in'] as num).toInt(),
    );

Map<String, dynamic> _$$AuthResponseImplToJson(_$AuthResponseImpl instance) =>
    <String, dynamic>{
      'access_token': instance.accessToken,
      'user_id': instance.userId,
      'expires_in': instance.expiresIn,
    };

_$AuthStateImpl _$$AuthStateImplFromJson(Map<String, dynamic> json) =>
    _$AuthStateImpl(
      userId: json['userId'] as String?,
      accessToken: json['accessToken'] as String?,
      tokenIssuedAt: (json['tokenIssuedAt'] as num?)?.toInt(),
      expiresIn: (json['expiresIn'] as num?)?.toInt(),
      isLoading: json['isLoading'] as bool? ?? false,
      error: json['error'] as String?,
    );

Map<String, dynamic> _$$AuthStateImplToJson(_$AuthStateImpl instance) =>
    <String, dynamic>{
      'userId': instance.userId,
      'accessToken': instance.accessToken,
      'tokenIssuedAt': instance.tokenIssuedAt,
      'expiresIn': instance.expiresIn,
      'isLoading': instance.isLoading,
      'error': instance.error,
    };
