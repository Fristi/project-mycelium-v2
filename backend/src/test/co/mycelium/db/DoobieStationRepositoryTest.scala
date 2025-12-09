package co.mycelium.db

import weaver._
import doobie.weaver._
import doobie._
import doobie.implicits._
import cats.effect.kernel.Resource
import cats.effect.IO
import co.mycelium.Transactors
import co.mycelium.DbConfig
import ciris.Secret
import java.util.UUID
import co.mycelium.domain.StationInsert
import java.time.Instant
import co.mycelium.domain.StationUpdate

object DoobieStationRepositoryTest extends IOSuite with IOChecker {

  override type Res = Transactor[IO]

  val config = DbConfig("localhost", 5432, "postgres", Secret("postgres"), "mycelium")
  val repo   = DoobieStationRepository
  val now    = Instant.now()

  override def sharedResource: Resource[IO, Res] =
    Transactors.pg[IO](config).map(tx => Transactor.after.set(tx, HC.rollback))

  test("inserting the station with the same mac should return same id") { implicit tx =>
    val insert  = StationInsert("00:00:00:00:00:00", "Unnamed")
    val station = insert.toStation(UUID.randomUUID(), now, "some-user-id")

    val program = for {
      id1 <- repo.insert(station, now)
      id2 <- repo.insert(station, now)
    } yield assert(id1 == id2, "Should be the same id")

    program.transact(tx)
  }

  test("should update station") { implicit tx =>
    val insert  = StationInsert("00:00:00:00:00:01", "Test Station")
    val station = insert.toStation(UUID.randomUUID(), now, "some-user-id")

    val program = for {
      id <- repo.insert(station, now)
      _  <- repo.update(id, "some-user-id", StationUpdate(name = Some("Updated Station")), now)
      retrieved <- repo.findById(id, "some-user-id")
    } yield assert(retrieved.exists(_.name == "Updated Station"), "Station should be updated")

    program.transact(tx)
  }

}
