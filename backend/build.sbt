scalaVersion := "3.7.4"
name         := "backend"
organization := "co.mycelium"
version      := "1.0"

libraryDependencies ++= Seq(
  "org.tpolecat"                  %% "doobie-core"                % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres"            % versions.doobie,
  "org.tpolecat"                  %% "doobie-hikari"              % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres-circe"      % versions.doobie,
  "org.tpolecat"                  %% "doobie-weaver"              % versions.doobie % Test,
  "org.typelevel"                 %% "cats-tagless-core"          % "0.16.3",
  "com.github.alonsodomin.cron4s" %% "cron4s-core"                % "0.8.2",
  "org.http4s"                    %% "http4s-dsl"                 % versions.http4s,
  "org.http4s"                    %% "http4s-ember-server"        % versions.http4s,
  "com.softwaremill.sttp.tapir"   %% "tapir-http4s-server"        % versions.tapir,
  "com.softwaremill.sttp.tapir"   %% "tapir-json-circe"           % versions.tapir,
  "org.flywaydb"                   % "flyway-database-postgresql" % "11.18.0",
  "ch.qos.logback"                 % "logback-classic"            % "1.5.21",
  "com.github.jwt-scala"          %% "jwt-core"                   % versions.jwt_scala,
  "com.github.jwt-scala"          %% "jwt-circe"                  % versions.jwt_scala,
  "com.auth0"                      % "jwks-rsa"                   % "0.23.0",
  "com.github.fs2-blobstore"      %% "s3"                         % "0.9.15",
  "is.cir"                        %% "ciris"                      % "3.11.1",
  "com.github.cb372"              %% "cats-retry"                 % "3.1.3",
  "io.sentry"                      % "sentry-logback"             % "8.27.1",
  "org.postgresql"                 % "postgresql"                 % "42.7.8",
  "com.softwaremill.sttp.tapir"   %% "tapir-openapi-docs"         % "1.12.6",
  "com.softwaremill.sttp.apispec" %% "openapi-circe-yaml"         % "0.11.10"
)

Compile / scalacOptions ++= Seq("-experimental")
mainClass := Some("co.mycelium.Main")

val tag = sys.props.getOrElse("imageTag", "latest")

jibBaseImage    := "gcr.io/distroless/java17-debian11"
jibImageFormat  := JibImageFormat.Docker
jibPlatforms    := Set(JibPlatforms.amd64, JibPlatforms.arm64)
jibVersion      := tag
jibName         := "mycelium-backend"
jibOrganization := "markdj"
