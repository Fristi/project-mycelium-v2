package co.mycelium

import cats.data.Kleisli
import cats.effect._
import cats.implicits._
import co.mycelium.db.Repositories
import co.mycelium.endpoints.{Avatar, Stations}
import com.comcast.ip4s._
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.{Router, Server}
import org.http4s.server.middleware.{CORS, CORSConfig, ErrorAction, ErrorHandling}
import org.http4s.server.staticcontent._
import org.http4s.{HttpApp, Request, Response}
import org.typelevel.log4cats.LoggerFactory
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

  private def errorHandling(route: Kleisli[IO, Request[IO], Response[IO]]) =
    ErrorHandling.Recover.total(
      ErrorAction.log(
        route,
        messageFailureLogAction = (t, msg) =>
          IO.println(msg) >>
            IO.println(t),
        serviceErrorLogAction = (t, msg) =>
          IO.println(msg) >>
            IO.delay(t.printStackTrace())
      )
    )

  val app: Resource[IO, Server] =
    for {
      cfg <- Resource.eval(AppConfig.config.load[IO])
      tx  <- Transactors.pg[IO](cfg.db)
      repos = Repositories.fromTransactor(tx)
      app = errorHandling(httpApp(repos))
      app_logging = org.http4s.server.middleware.Logger.httpApp(true, true)(app)
      server <- EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"8080")
        .withHttpApp(app_logging)
        .build
    } yield server
}
