package co.mycelium

import cats.effect.*
import doobie.{KleisliInterpreter, WeakAsync}
import doobie.util.log.*
import doobie.util.transactor.{Strategy, Transactor}
import org.typelevel.keypool.*
import org.typelevel.log4cats.Logger

import java.sql.{Connection, DriverManager}
import java.util.Properties
import scala.concurrent.ExecutionContext
import scala.concurrent.duration.Duration

type PooledTransactor[F[_]] = Transactor.Aux[F, Pool[F, Connection]]

object PooledTransactor {

  private class Slf4jLogger[F[_]](log: Logger[F]) extends LogHandler[F] {
    override def run(logEvent: LogEvent): F[Unit] = logEvent match {
      case Success(sql, params, label, exec, processing) =>
        log.debug(
          s"Successfully executed SQL: ${sql.linesIterator.dropWhile(_.trim.isEmpty).mkString(" ")}"
        )
      case ProcessingFailure(sql, params, label, exec, processing, failure) =>
        log.error(
          s"Error processing SQL: ${sql.linesIterator.dropWhile(_.trim.isEmpty).mkString(" ")} -> ${failure.getMessage}"
        )
      case ExecFailure(sql, params, label, exec, failure) =>
        log.error(
          s"Error executing SQL: ${sql.linesIterator.dropWhile(_.trim.isEmpty).mkString(" ")} -> ${failure.getMessage}"
        )
    }
  }

  def connection[F[_]: Sync](url: String, properties: Properties): Resource[F, Connection] =
    Resource.make(Sync[F].blocking(DriverManager.getConnection(url, properties)))(x =>
      Sync[F].delay(x.close())
    )

  def pool[F[_]: {Temporal, WeakAsync}](
      url: String,
      username: String,
      password: String,
      maxConnectionsInPool: Int,
      transactEC: ExecutionContext,
      logger: Logger[F]
  ): Resource[F, PooledTransactor[F]] = {

    val properties = new Properties()
    properties.setProperty("user", username)
    properties.setProperty("password", password)
    properties.setProperty("ssl", "false")

    def pool: Resource[F, Pool[F, Connection]] = Pool
      .Builder(connection(url, properties))
      .withDefaultReuseState(Reusable.Reuse)
      .withIdleTimeAllowedInPool(Duration.Inf)
      .withMaxTotal(maxConnectionsInPool)
      .withOnReaperException((_: Throwable) => Sync[F].unit)
      .build

    pool.map { p =>
      Transactor(
        p,
        _.take.map(_.value),
        KleisliInterpreter[F](new Slf4jLogger[F](logger)).ConnectionInterpreter,
        Strategy.default
      )
    }
  }

}
