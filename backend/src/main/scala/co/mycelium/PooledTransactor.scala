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

import fs2._
import scala.concurrent.duration.*
import cats.Functor

type PooledTransactor[F[_]] = Transactor.Aux[F, Pool[F, Connection]]

object PooledTransactor {

    def connection[F[_] : Sync](url: String, properties: Properties): Resource[F, Connection] = 
        Resource.make(Sync[F].blocking(DriverManager.getConnection(url, properties)))(x => Sync[F].delay(x.close()))

    def pool[F[_]: Temporal : WeakAsync](
        url:    String,
        username: String,
        password: String,
        maxConnectionsInPool: Int,
        transactEC:  ExecutionContext
    ): Resource[F, PooledTransactor[F]] = {

        val properties = new Properties()
        properties.setProperty("user", username)
        properties.setProperty("password", password)
        properties.setProperty("ssl", "false")

        def pool: Resource[F, Pool[F, Connection]] = Pool.Builder(connection(url, properties))
            .withDefaultReuseState(Reusable.Reuse)
            .withIdleTimeAllowedInPool(Duration.Inf)
            .withMaxTotal(maxConnectionsInPool)
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