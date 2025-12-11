package co.mycelium.db

import cats.effect.{IO, Resource}
import cats.implicits.*
import ciris.Secret
import co.mycelium.{Database, DbConfig}
import doobie.{HC, Transactor}
import org.typelevel.log4cats.slf4j.Slf4jLogger

object DoobieResource {
  def setup: Resource[IO, Transactor[IO]] = {
    val log    = Slf4jLogger.getLoggerFromName[IO]("Doobie")
    val config = DbConfig("localhost", 5432, 1, "postgres", Secret("postgres"), "mycelium")
    val tx = Database.transactor[IO](config, log).map(tx => Transactor.after.set(tx, HC.rollback))
    val migrations = Resource.eval(Database.flyway[IO](config))

    migrations >> tx
  }
}
