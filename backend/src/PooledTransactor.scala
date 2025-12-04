
package co.mycelium

import cats.implicits.*
import cats.effect.*
import doobie.KleisliInterpreter
import doobie.util.transactor.Transactor
import doobie.util.transactor.Strategy
import scala.concurrent.ExecutionContext

import java.sql.{Connection, DriverManager}
import cats.effect.std.Semaphore
import org.typelevel.keypool.*;
import java.util.Properties
import scala.concurrent.duration.Duration
import doobie.util.log.LogHandler
import doobie.WeakAsync

object PooledTransactor {

    def connection[F[_] : Sync](driver: String, url: String): Resource[F, Connection] = 
        Resource.make(Sync[F].blocking(DriverManager.getDriver(driver).connect(url, new Properties())))(x => Sync[F].delay(x.close()))

  /**
   * Build a Pool of Connections, using the url.
   */
  def pool[F[_]: Temporal: Sync : WeakAsync](
    driver: String,
    url:    String,
    maxConnectionsInPool: Int,
    transactEC:  ExecutionContext
  ): Resource[F, Transactor[F]] = {
    def pool: Resource[F, Pool[F, Connection]] = Pool.Builder(connection(driver, url))
        .withDefaultReuseState(Reusable.Reuse)
        .withIdleTimeAllowedInPool(Duration.Inf)
        .withMaxTotal(10)
        .withOnReaperException((_: Throwable) => Sync[F].unit)
        .build

    pool.map { p => 
        Transactor(
            p,
            _.take.map(_.value),
            KleisliInterpreter[F](LogHandler.jdkLogHandler[F]).ConnectionInterpreter,
            Strategy.default
        )
    }
  }

}