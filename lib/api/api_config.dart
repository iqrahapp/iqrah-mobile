import 'package:openapi_generator_annotations/openapi_generator_annotations.dart';

@Openapi(
  inputSpec: RemoteSpec(
    path: 'https://raw.githubusercontent.com/iqrahapp/iqrah-backend/main/openapi.json',
  ),
  generatorName: Generator.dart,
  outputDirectory: 'lib/api/generated',
  skipSpecValidation: true,
  fetchDependencies: false,
  runSourceGenOnOutput: false,
  forceAlwaysRun: false,
  additionalProperties: AdditionalProperties(
    pubName: 'iqrah_api_client',
    pubAuthor: 'Iqrah',
  ),
)
class IqrahApiClientConfig {}
