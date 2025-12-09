package co.mycelium

import cats.Applicative
import cats.effect.{Async, IO, Resource}
import retry._
import retry.RetryPolicies._

import javax.sql.DataSource
import scala.concurrent.duration.DurationInt
import doobie.Transactor
import java.sql.Connection
import org.typelevel.keypool.Pool
import cats.effect.Sync
import java.io.PrintWriter
import cats.effect.Temporal
import doobie.WeakAsync
import co.mycelium.PooledTransactor.pool
import java.util.logging.Logger
import java.sql.SQLException
import co.mycelium.DbConfig
import co.mycelium.PooledTransactor

object Transactors {
  def pg[F[_]: Temporal : WeakAsync](cfg: DbConfig): Resource[F, PooledTransactor[F]] = {
    type ResourceM[A] = Resource[F, A]

    def policy[F[_]: Applicative] =
      limitRetries[F](10) join exponentialBackoff[F](200.milliseconds)

    def handleError(error: Throwable, retryDetails: RetryDetails): Resource[F, Unit] = 
      Resource.eval(Sync[F].blocking(error.printStackTrace()))

    val pooledTransactor = PooledTransactor.pool(
        s"jdbc:postgresql://${cfg.host}:${cfg.port}/${cfg.database}", 
        cfg.username,
        cfg.password.value,
        10, 
        scala.concurrent.ExecutionContext.global
      )

    // retryingOnAllErrors.apply[ResourceM, Throwable](policy, handleError)()
    pooledTransactor
  }

}
