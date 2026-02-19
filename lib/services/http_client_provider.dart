import 'dart:async';

import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:iqrah/providers/auth_provider.dart';
import 'package:iqrah/utils/app_logger.dart';
import 'package:iqrah/utils/backend_config.dart';

/// Authenticated HTTP client provider.
final httpClientProvider = Provider<Dio>((ref) {
  final dio = Dio(
    BaseOptions(
      baseUrl: backendBaseUrl,
      connectTimeout: const Duration(seconds: 30),
      receiveTimeout: const Duration(seconds: 30),
      headers: const {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
    ),
  );

  dio.interceptors.add(AuthInterceptor(ref));
  dio.interceptors.add(
    LogInterceptor(
      requestBody: true,
      responseBody: true,
      logPrint: (obj) => AppLogger.logHTTP(obj.toString()),
    ),
  );

  return dio;
});

/// Interceptor that adds JWT Bearer token to outgoing requests.
class AuthInterceptor extends Interceptor {
  final Ref ref;

  AuthInterceptor(this.ref);

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    final token = ref.read(authProvider.notifier).getToken();
    if (token != null) {
      options.headers['Authorization'] = 'Bearer $token';
      AppLogger.logHTTP('Added auth header to ${options.method} ${options.path}');
    }
    handler.next(options);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (err.response?.statusCode == 401) {
      AppLogger.logAuth('Token expired (401), signing out');
      unawaited(ref.read(authProvider.notifier).signOut());
    }
    handler.next(err);
  }
}
