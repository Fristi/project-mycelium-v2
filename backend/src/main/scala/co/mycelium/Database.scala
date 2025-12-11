package co.mycelium

import cats.Applicative
import cats.effect.{Resource, Sync, Temporal}
import co.mycelium.{DbConfig, PooledTransactor}
import doobie.WeakAsync
import org.flywaydb.core.Flyway
import org.flywaydb.core.api.output.MigrateResult
import org.typelevel.log4cats.Logger
import retry.*
import retry.RetryPolicies.*

import scala.concurrent.duration.DurationInt

object Database {
  private def policy[F[_]: Applicative]: RetryPolicy[F] =
    limitRetries[F](10) join exponentialBackoff[F](200.milliseconds)

  private def handleError[F[_]](
      log: Logger[F]
  )(error: Throwable, retryDetails: RetryDetails): Resource[F, Unit] =
    Resource.eval(log.error(error)(s"Retried connecting to database: $retryDetails"))

//  private def retryingPoolTransactor[F[_], E](log: Logger[F], pt: Resource[F, PooledTransactor[F]])(given M: MonadError[F, E], S: Sleep[F]) =
//    retryingOnAllErrors[PooledTransactor[F]].apply(policy, (err, details) => handleError(log)(err, details))

  def flyway[F[_]: Sync](cfg: DbConfig): F[MigrateResult] =
    Sync[F].blocking {
      Flyway
        .configure()
        .locations("migrations")
        .dataSource(
          s"jdbc:postgresql://${cfg.host}:${cfg.port}/${cfg.database}",
          cfg.username,
          cfg.password.value
        )
        .load()
        .migrate()
    }

  def transactor[F[_]: {Temporal, WeakAsync}](
      cfg: DbConfig,
      log: Logger[F]
  ): Resource[F, PooledTransactor[F]] = {
    val pooledTransactor = PooledTransactor.pool(
      s"jdbc:postgresql://${cfg.host}:${cfg.port}/${cfg.database}",
      cfg.username,
      cfg.password.value,
      cfg.maxConnections,
      scala.concurrent.ExecutionContext.global,
      log
    )

    pooledTransactor
  }

}
