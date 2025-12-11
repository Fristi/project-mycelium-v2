package co.mycelium

import ciris._
import cats.implicits._

enum ImplementationType(val value: String) {
  case Production extends ImplementationType("production")
  case Constant   extends ImplementationType("constant")
}

object ImplementationType {
  given ConfigDecoder[String, ImplementationType] =
    ConfigDecoder[String].mapOption("ImplementationType")(fromString)

  val all = List(Production, Constant)

  def fromString(str: String): Option[ImplementationType] = all.find(_.value.equalsIgnoreCase(str))
}

final case class DbConfig(
    host: String,
    port: Int,
    maxConnections: Int,
    username: String,
    password: Secret[String],
    database: String
)

object DbConfig {
  val config =
    (
      env("PG_HOST").as[String].default("localhost"),
      env("PG_PORT").as[Int].default(5432),
      env("PG_MAX_CONNECTIONS").as[Int].default(10),
      env("PG_USER").as[String].default("postgres"),
      env("PG_PASS").as[String].secret.default(Secret("postgres")),
      env("PG_DB").as[String].default("mycelium")
    ).parMapN(DbConfig.apply)
}

final case class S3BlobConfig(host: String, accessKey: String, secretKey: Secret[String])

object S3BlobConfig {
  val config =
    (
      env("S3_HOST").as[String].default("http://127.0.0.1:9000"),
      env("S3_ACCESS_KEY").as[String].default("minio"),
      env("S3_SECRET_KEY").as[String].secret.default(Secret("miniominio"))
    ).parMapN(S3BlobConfig.apply)
}

final case class PlantProfilerConfig(
    implementation: ImplementationType,
    openPlantBookKey: Secret[String]
)

object PlantProfilerConfig {
  val config =
    (
      env("PLANT_PROFILER_IMPLEMENTATION")
        .as[ImplementationType]
        .default(ImplementationType.Constant),
      env("PLANT_PROFILER_OPENPLANTBOOK_APIKEY").as[String].secret.default(Secret("secret"))
    ).parMapN(PlantProfilerConfig.apply)
}

final case class PlantClassifierConfig(
    implementation: ImplementationType,
    openAiKey: Secret[String]
)

object PlantClassifierConfig {
  val config =
    (
      env("PLANT_CLASSIFIER_IMPLEMENTATION")
        .as[ImplementationType]
        .default(ImplementationType.Constant),
      env("PLANT_CLASSIFIER_OPENAI_APIKEY").as[String].secret.default(Secret("secret"))
    ).parMapN(PlantClassifierConfig.apply)
}

final case class AppConfig(
    db: DbConfig,
    blob: S3BlobConfig,
    plantClassifier: PlantClassifierConfig,
    plantProfiler: PlantProfilerConfig
)

object AppConfig {
  val config: ConfigValue[Effect, AppConfig] =
    (
      DbConfig.config,
      S3BlobConfig.config,
      PlantClassifierConfig.config,
      PlantProfilerConfig.config
    ).parMapN(AppConfig.apply)
}
