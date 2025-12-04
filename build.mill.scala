//| mvnDeps:
//| - io.github.alexarchambault.mill::mill-native-image::0.2.4
import mill.*, mill.scalalib.*
import io.github.alexarchambault.millnativeimage.NativeImage

object versions {
  val doobie    = "1.0.0-RC11"
  val http4s    = "0.23.32"
  val tapir     = "1.11.40"
  val jwt_scala = "10.0.0"
}

object backend extends ScalaModule with NativeImage {
  def scalaVersion = "3.7.4"

  val mainClass_ = "co.mycelium.Main"

  def mainClass = Some(mainClass_)

  def nativeImageName         = "backend"
  def nativeImageMainClass    = mainClass_
  def nativeImageGraalVmJvmId = "graalvm-java23:23.0.2"
  def nativeImageClassPath    = runClasspath()
  def nativeImageOptions = Seq(
    "--no-fallback",
    "--report-unsupported-elements-at-runtime",
    "--enable-url-protocols=jdbc",
    "-H:IncludeResources=META-INF/services/.*,migrations/.*,logback.*\\.xml",
      // avoid initializing heavy JDBC/connection code at image-build-time
      "--initialize-at-run-time=com.zaxxer.hikari",
      "--initialize-at-run-time=com.zaxxer.hikari.pool",
      "--initialize-at-run-time=org.postgresql",
      "--initialize-at-run-time=org.postgresql.Driver",
      "--initialize-at-run-time=org.postgresql.jdbc",
      "--initialize-at-run-time=org.flywaydb",

      // better runtime diagnostics if something goes wrong
      "-H:+ReportExceptionStackTraces",
      "--verbose",
      "--allow-incomplete-classpath",
      "--report-unsupported-elements-at-runtime"
  )

  def mvnDeps = Seq(
    mvn"org.tpolecat::doobie-core:${versions.doobie}",
    mvn"org.tpolecat::doobie-postgres:${versions.doobie}",
    mvn"org.tpolecat::doobie-postgres-circe:${versions.doobie}",
    mvn"org.typelevel::cats-tagless-core:0.16.3",
    mvn"org.typelevel::keypool:0.4.10",
    mvn"com.github.alonsodomin.cron4s::cron4s-core:0.8.2",
    mvn"org.http4s::http4s-dsl:${versions.http4s}",
    mvn"org.http4s::http4s-ember-server:${versions.http4s}",
    mvn"com.softwaremill.sttp.tapir::tapir-http4s-server:${versions.tapir}",
    mvn"com.softwaremill.sttp.tapir::tapir-json-circe:${versions.tapir}",
    mvn"org.flywaydb:flyway-database-postgresql:11.18.0",
    mvn"ch.qos.logback:logback-classic:1.5.21",
    mvn"com.github.jwt-scala::jwt-core:${versions.jwt_scala}",
    mvn"com.github.jwt-scala::jwt-circe:${versions.jwt_scala}",
    mvn"com.auth0:jwks-rsa:0.23.0",
    mvn"is.cir::ciris:3.11.1",
    mvn"com.github.cb372::cats-retry:3.1.0",
    mvn"io.sentry:sentry-logback:8.27.1",
    mvn"org.postgresql:postgresql:42.7.8",
    mvn"com.softwaremill.sttp.tapir::tapir-openapi-docs:1.12.6",
    mvn"com.softwaremill.sttp.apispec::openapi-circe-yaml:0.11.10"
  )

  def scalacOptions = Seq("-experimental")

  // If instead of creating a Native-Image binary for current host (Eg. MacOS)
  // you want to create a Docker image with the binary for Linux in a Docker container
  // you can use the following parameters and run `DOCKER_NATIVEIMAGE=1 mill hello.nativeImage`
  def isDockerBuild           = Task.Input(Task.ctx().env.get("DOCKER_NATIVEIMAGE") != None)
  def nativeImageDockerParams = Task {
    if isDockerBuild() then
      Some(
        NativeImage.DockerParams(
          imageName = "ubuntu:22.04",
          prepareCommand = """apt-get update -q -y &&\
                             |apt-get install -q -y build-essential libz-dev locales --no-install-recommends
                             |locale-gen en_US.UTF-8
                             |export LANG=en_US.UTF-8
                             |export LANGUAGE=en_US:en
                             |export LC_ALL=en_US.UTF-8""".stripMargin,
          csUrl = s"https://github.com/coursier/coursier/releases/download/v2.1.2/cs-x86_64-pc-linux.gz",
          extraNativeImageArgs = Nil,
        )
      )
    else Option.empty[NativeImage.DockerParams]
  }
}