package co.mycelium.db

import cats.effect.IO
import cats.effect.kernel.Resource
import ciris.Secret
import co.mycelium.adapters.db.DoobieStationRepository
import co.mycelium.{Database, DbConfig}
import co.mycelium.domain.{StationInsert, StationUpdate}
import doobie.weaver.*
import weaver.*
import doobie.*
import doobie.implicits.*
import org.typelevel.log4cats.slf4j.Slf4jLogger

import java.time.Instant
import java.util.UUID

object DoobieStationRepositoryTest extends IOSuite with IOChecker {

  override type Res = Transactor[IO]

  val repo = DoobieStationRepository
  val now  = Instant.now()

  override def sharedResource: Resource[IO, Res] =
    DoobieResource.setup

  test("inserting the station with the same mac should return same id") { implicit tx =>
    val insert  = StationInsert("00:00:00:00:00:00", "Unnamed")
    val station = insert.toStation(UUID.randomUUID(), now, "some-user-id")

    val program = for {
      id1 <- repo.insert(station, now)
      id2 <- repo.insert(station, now)
    } yield expect.eql(id1, id2)

    program.transact(tx)
  }

  test("should update station") { implicit tx =>
    val insert  = StationInsert("00:00:00:00:00:01", "Test Station")
    val station = insert.toStation(UUID.randomUUID(), now, "some-user-id")

    val program = for {
      id <- repo.insert(station, now)
      _  <- repo.update(id, "some-user-id", StationUpdate(name = Some("Updated Station")), now)
      retrieved <- repo.findById(id, "some-user-id")
    } yield expect.eql(retrieved.exists(_.name == "Updated Station"), true)

    program.transact(tx)
  }

}
