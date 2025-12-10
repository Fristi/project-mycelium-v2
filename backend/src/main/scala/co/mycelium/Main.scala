package co.mycelium

import cats.data.Kleisli
import cats.effect.*
import cats.implicits.*
import co.mycelium.db.Repositories
import co.mycelium.endpoints.{Avatar, Stations}
import com.comcast.ip4s.*
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.{Router, Server}
import org.http4s.server.middleware.{CORS, CORSConfig, ErrorAction, ErrorHandling}
import org.http4s.server.staticcontent.*
import org.http4s.{HttpApp, Request, Response}
import org.typelevel.log4cats.{Logger, LoggerFactory}
import org.typelevel.log4cats.slf4j.Slf4jFactory

import java.net.URI
import java.time.Duration
import org.http4s.Method
import sttp.tapir.server.interceptor.cors
import org.http4s.server.middleware.CORSPolicy
import co.mycelium.AppConfig
import co.mycelium.Transactors

object Main extends IOApp {

  implicit val loggerFactory: LoggerFactory[IO] = Slf4jFactory.create[IO]

  override def run(args: List[String]): IO[ExitCode] =
    app.use(_ => IO.never).as(ExitCode.Success)

  def httpApp(repositories: Repositories[IO]): HttpApp[IO] = {

    val server = Router(
      "api"    -> Stations.routes(repositories),
      "avatar" -> Avatar.routes
    )
    val files  = fileService[IO](FileService.Config("."))
    val routes = (server <+> files).orNotFound

    routes
  }

  private def errorHandling(log: Logger[IO])(route: Kleisli[IO, Request[IO], Response[IO]]) =
    ErrorHandling.Recover.total(
      ErrorAction.log(route, (t, msg) => log.warn(t)(msg), (t, msg) => log.error(t)(msg))
    )

  val app: Resource[IO, Server] =
    for {
      cfg <- Resource.eval(AppConfig.config.load[IO])
      tx  <- Transactors.pg[IO](cfg.db, loggerFactory.getLoggerFromName("Doobie"))
      repos        = Repositories.fromTransactor(tx)
      errorLogging = loggerFactory.getLoggerFromName("Http4s")
      app          = errorHandling(errorLogging)(httpApp(repos))
      server <- EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"8080")
        .withHttpApp(app)
        .build
    } yield server
}
