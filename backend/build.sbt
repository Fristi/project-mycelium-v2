scalaVersion := "3.7.4"
name         := "backend"
organization := "co.mycelium"
version      := "1.0"

libraryDependencies ++= Seq(
  "org.tpolecat"                  %% "doobie-core"           % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres"       % versions.doobie,
  "org.tpolecat"                  %% "doobie-postgres-circe" % versions.doobie,
  "org.tpolecat"                  %% "doobie-weaver"         % versions.doobie % Test,
  "org.typelevel"                 %% "cats-time"             % "0.6.0",
  "org.typelevel"                 %% "keypool"               % "0.4.10",
  "org.typelevel"                 %% "kittens"               % "3.5.0",
  "org.typelevel"                 %% "cats-tagless-core"     % "0.16.3",
  "com.github.alonsodomin.cron4s" %% "cron4s-core"           % "0.8.2",
  "org.http4s"                    %% "http4s-dsl"            % versions.http4s,
  "org.http4s"                    %% "http4s-ember-server"   % versions.http4s,
  "com.softwaremill.sttp.tapir"   %% "tapir-http4s-server"   % versions.tapir,
  "com.softwaremill.sttp.tapir"   %% "tapir-json-circe"      % versions.tapir,
  "com.softwaremill.sttp.tapir"   %% "tapir-openapi-docs"    % "1.12.6",
  "com.softwaremill.sttp.ai"      %% "fs2"                   % "0.4.3",
  "com.softwaremill.sttp.client4" %% "cats"                  % "4.0.13",
  "com.softwaremill.sttp.client4" %% "circe"                 % "4.0.13",
  "com.softwaremill.sttp.apispec" %% "openapi-circe-yaml"    % "0.11.10",
  "com.github.jwt-scala"          %% "jwt-core"              % versions.jwt_scala,
  "com.github.jwt-scala"          %% "jwt-circe"             % versions.jwt_scala,
  "is.cir"                        %% "ciris"                 % "3.11.1",
  "com.github.cb372"              %% "cats-retry"            % "3.1.3",
  "com.github.fs2-blobstore"      %% "s3"                    % "0.9.15",
  "co.fs2"                        %% "fs2-core"              % "3.12.2",
  "co.fs2"                        %% "fs2-io"                % "3.12.2",
  "com.auth0"                      % "jwks-rsa"              % "0.23.0",
  "ch.qos.logback"                 % "logback-classic"       % "1.5.21",
  "ch.qos.logback"                 % "logback-core"          % "1.5.21",
  "org.slf4j"                      % "slf4j-api"             % "2.0.17",
  "org.postgresql"                 % "postgresql"            % "42.7.8",
  "com.sksamuel.scrimage"          % "scrimage-core"         % "4.3.5"
)

val mainClass_ = "co.mycelium.Main"
val tag = sys.props.getOrElse("imageTag", "latest")


Compile / scalacOptions ++= Seq("-experimental")
Compile / run / fork := true

assembly / mainClass := Some(mainClass_)
assembly / assemblyMergeStrategy := {
  case PathList("META-INF", "native-image", xs @ _*) => MergeStrategy.first
  case "META-INF/io.netty.versions.properties"       => MergeStrategy.first
  case "module-info.class"                           => MergeStrategy.discard
  case PathList("META-INF", "services", xs @ _*)     => MergeStrategy.concat
  case PathList("META-INF", _)                       => MergeStrategy.discard
  case x                                             => MergeStrategy.first
}

mainClass := Some(mainClass_)

jibBaseImage := "gcr.io/distroless/java17-debian11"
jibImageFormat := JibImageFormat.Docker
jibPlatforms := Set(JibPlatforms.amd64, JibPlatforms.arm64)
jibVersion := tag
jibName := "mycelium-backend"
jibOrganization := "markdj"
