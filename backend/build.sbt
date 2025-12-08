scalaVersion := "3.7.4"
name         := "backend"
organization := "co.mycelium"
version      := "1.0"

libraryDependencies ++= Seq(
  "org.tpolecat"                  %% "doobie-core"                % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres"            % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres-circe"      % versions.doobie,
  "org.tpolecat"                  %% "doobie-weaver"              % versions.doobie % Test,
  "org.typelevel"                 %% "keypool"                    % "0.4.10",
  "org.typelevel"                 %% "cats-tagless-core"          % "0.16.3",
  "com.github.alonsodomin.cron4s" %% "cron4s-core"                % "0.8.2",
  "org.http4s"                    %% "http4s-dsl"                 % versions.http4s,
  "org.http4s"                    %% "http4s-ember-server"        % versions.http4s,
  "com.softwaremill.sttp.tapir"   %% "tapir-http4s-server"        % versions.tapir,
  "com.softwaremill.sttp.tapir"   %% "tapir-json-circe"           % versions.tapir,
  "com.softwaremill.sttp.tapir"   %% "tapir-openapi-docs"         % "1.12.6",
  "com.softwaremill.sttp.apispec" %% "openapi-circe-yaml"         % "0.11.10",
  "ch.qos.logback"                 % "logback-classic"            % "1.5.21",
  "com.github.jwt-scala"          %% "jwt-core"                   % versions.jwt_scala,
  "com.github.jwt-scala"          %% "jwt-circe"                  % versions.jwt_scala,
  "com.auth0"                      % "jwks-rsa"                   % "0.23.0",
  "com.github.fs2-blobstore"      %% "s3"                         % "0.9.15",
  "is.cir"                        %% "ciris"                      % "3.11.1",
  "com.github.cb372"              %% "cats-retry"                 % "3.1.3",
  "io.sentry"                      % "sentry-logback"             % "8.27.1",
  "org.postgresql"                 % "postgresql"                 % "42.7.8"
)

Compile / scalacOptions ++= Seq("-experimental")

assembly  / mainClass := Some("co.mycelium.Main")

assembly / assemblyMergeStrategy := {
  case PathList("META-INF", "native-image", xs @ _*)=> MergeStrategy.first
  case "META-INF/io.netty.versions.properties" => MergeStrategy.first
  case "module-info.class" => MergeStrategy.discard
  case PathList("META-INF", xs @ _*) => MergeStrategy.discard
  case x => MergeStrategy.first
}
