package co.mycelium.db

import cats.effect.IO
import cats.effect.kernel.Resource
import ciris.Secret
import co.mycelium.{DbConfig, Transactors}
import co.mycelium.domain.{MeasurementPeriod, StationInsert, StationMeasurement, StationUpdate}
import doobie.weaver.*
import weaver.*
import doobie.*
import doobie.implicits.*
import org.typelevel.log4cats.Logger
import org.typelevel.log4cats.slf4j.Slf4jLogger

import java.time.Instant
import java.util.UUID

object DoobieStationMeasurementRepositoryTest extends IOSuite with IOChecker {

  override type Res = Transactor[IO]

  val log = Slf4jLogger.getLoggerFromName[IO]("Doobie")

  val config  = DbConfig("localhost", 5432, 1, "postgres", Secret("postgres"), "mycelium")
  val repo    = DoobieStationMeasurementRepository
  val now     = Instant.parse("2025-07-29T00:10:00Z")
  val insert  = StationInsert("00:00:00:00:00:00", "Unnamed")
  val station = insert.toStation(UUID.randomUUID(), now, "some-user-id")

  override def sharedResource: Resource[IO, Res] =
    Transactors.pg[IO](config, log).map(tx => Transactor.after.set(tx, HC.rollback))

  test("average should work") { implicit tx =>
    val timebucket = Instant.parse("2025-07-29T00:00:00Z")

    val program = for {
      id <- DoobieStationRepository.insert(station, now)
      _  <- DoobieStationMeasurementRepository.insertMany(
        id,
        List(
          StationMeasurement(now, 12.5, 20.5, 65.0, 100.0, 2.5, 3.0),
          StationMeasurement(now.plusSeconds(60), 12.6, 21.0, 64.5, 110.0, 2.6, 3.1),
          StationMeasurement(now.plusSeconds(120), 12.7, 21.5, 64.0, 120.0, 2.7, 3.2),
          StationMeasurement(now.plusSeconds(180), 12.8, 22.0, 63.5, 130.0, 2.8, 3.3),
          StationMeasurement(now.plusSeconds(240), 12.9, 22.5, 63.0, 140.0, 2.9, 3.4),
          StationMeasurement(now.plusSeconds(300), 13.0, 23.0, 62.5, 150.0, 3.0, 3.5),
          StationMeasurement(now.plusSeconds(360), 13.1, 23.5, 62.0, 160.0, 3.1, 3.6),
          StationMeasurement(now.plusSeconds(420), 13.2, 24.0, 61.5, 170.0, 3.2, 3.7),
          StationMeasurement(now.plusSeconds(480), 13.3, 24.5, 61.0, 180.0, 3.3, 3.8),
          StationMeasurement(now.plusSeconds(540), 13.4, 25.0, 60.5, 190.0, 3.4, 3.9)
        )
      )
      avg <- DoobieStationMeasurementRepository.avg(id, MeasurementPeriod.LastMonth)
    } yield {
      expect.eql(avg, List(StationMeasurement(timebucket, 12.95, 22.75, 62.75, 145.0, 2.95, 3.45)))
    }

    program.transact(tx)
  }

}
